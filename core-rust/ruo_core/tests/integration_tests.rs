use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ruo_core::db::DbHandle;
use ruo_core::error::CoreError;
use ruo_core::models::{
    LocalMessage, LocalMessageStatus, MessageRole, OutboundRequest, StreamMeta,
};
use ruo_core::outbox::{OutboxItem, OutboxStatus};
use ruo_core::{RuoCore, StreamListener};

#[test]
fn test_outbox_state_transitions() {
    let mut item = OutboxItem::new(
        "msg-01".to_string(),
        "conv-01".to_string(),
        "Content".to_string(),
    );
    assert_eq!(item.status, OutboxStatus::Pending);
    assert_eq!(item.attempts, 0);

    item.mark_sending();
    assert_eq!(item.status, OutboxStatus::Sending);
    assert_eq!(item.attempts, 1);

    item.mark_failed("Network unreachable".to_string());
    assert_eq!(item.status, OutboxStatus::Failed);
    assert_eq!(item.last_error.as_deref(), Some("Network unreachable"));

    item.mark_sending();
    assert_eq!(item.attempts, 2);

    item.mark_sent();
    assert_eq!(item.status, OutboxStatus::Sent);
    assert!(item.last_error.is_none());
}

#[test]
fn test_db_repositories_conversations_and_messages() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("repo_test.db");
    let key = vec![0x42; 32];
    let db = DbHandle::open(db_path.to_str().unwrap(), &key).unwrap();
    db.create_conversation("c1", "Biology", "en", false)
        .unwrap();
    db.create_conversation("c2", "Calculus Integrals", "tl-PH", true)
        .unwrap();

    let list_all = db.list_conversations(None, 50, None, None, false).unwrap();
    assert_eq!(list_all.len(), 2);
    let list_filtered = db
        .list_conversations(Some("Calculus"), 50, None, None, false)
        .unwrap();
    assert_eq!(list_filtered.len(), 1);
    assert_eq!(list_filtered[0].title, "Calculus Integrals");

    let message = LocalMessage {
        id: "m1".to_string(),
        server_id: None,
        conversation_id: "c1".to_string(),
        client_msg_id: Some("m1".to_string()),
        role: MessageRole::User,
        content_md: "What is mitosis?".to_string(),
        status: LocalMessageStatus::Sent,
        citations_json: None,
        parent_message_id: None,
        created_at_ms: 1000,
        created_at_us: 1_000_000,
    };
    db.insert_message(&message).unwrap();
    let mut updated = message.clone();
    updated.status = LocalMessageStatus::Failed;
    db.insert_message(&updated).unwrap();
    assert_eq!(
        db.list_messages("c1", 10, None, None).unwrap()[0].status,
        LocalMessageStatus::Failed
    );
}

struct TestStreamListener {
    received_deltas: Mutex<Vec<String>>,
    completed: AtomicBool,
    error_code: Mutex<Option<String>>,
    metadata: Mutex<Option<StreamMeta>>,
}

impl StreamListener for TestStreamListener {
    fn on_meta(&self, _: String, meta: StreamMeta) {
        if let Ok(mut value) = self.metadata.lock() {
            *value = Some(meta);
        }
    }

    fn on_delta(&self, _: String, text: String) {
        if let Ok(mut values) = self.received_deltas.lock() {
            values.push(text);
        }
    }

    fn on_done(&self, _: String, _: String, _: bool) {
        self.completed.store(true, Ordering::SeqCst);
    }

    fn on_error(&self, _: String, error: CoreError) {
        if let Ok(mut value) = self.error_code.lock() {
            *value = Some(error.code().to_string());
        }
    }
}

#[tokio::test]
async fn test_sse_mock_stream_and_cancel_mid_flight() {
    use warp::Filter;

    let sse_route = warp::path!("v1" / "conversations" / String / "messages")
        .and(warp::post())
        .map(|_conversation_id| {
            let stream = async_stream::stream! {
                yield Ok::<_, warp::Error>(warp::sse::Event::default().event("meta").data(
                    r#"{"client_msg_id":"stream-to-cancel","user_message_id":"user-server-id","assistant_message_id":"assistant-server-id","fallback":false}"#,
                ));
                for index in 1..=10 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    yield Ok::<_, warp::Error>(warp::sse::Event::default().event("delta").data(
                        format!(r#"{{"text":"Chunk {index} "}}"#),
                    ));
                }
                yield Ok::<_, warp::Error>(warp::sse::Event::default().event("done").data(
                    r#"{"assistant_message_id":"assistant-server-id","replayed":false}"#,
                ));
            };
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        });

    let (address, server) = warp::serve(sse_route).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::spawn(server);

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("mock_test.db");
    let key = vec![0x42; 32];
    {
        let db = DbHandle::open(db_path.to_str().unwrap(), &key).unwrap();
        db.create_conversation("c-mock", "Mock", "en", false)
            .unwrap();
    }
    let core = RuoCore::open(db_path.to_str().unwrap().to_string(), key).unwrap();
    core.set_session("mock-token".to_string(), format!("http://{address}"), true)
        .unwrap();

    let listener = Arc::new(TestStreamListener {
        received_deltas: Mutex::new(Vec::new()),
        completed: AtomicBool::new(false),
        error_code: Mutex::new(None),
        metadata: Mutex::new(None),
    });
    core.stream_message(
        OutboundRequest {
            client_msg_id: "stream-to-cancel".to_string(),
            conversation_id: "c-mock".to_string(),
            content: "Please start streaming".to_string(),
            parent_message_id: None,
            attachment_ids: vec![],
            intent: None,
            study_mode: false,
        },
        listener.clone(),
    )
    .unwrap();

    tokio::time::sleep(Duration::from_millis(250)).await;
    assert!(core.cancel_stream("stream-to-cancel".to_string()));
    for _ in 0..20 {
        if listener.error_code.lock().unwrap().as_deref() == Some("CANCELLED") {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    assert_eq!(
        listener.error_code.lock().unwrap().as_deref(),
        Some("CANCELLED")
    );
    assert!(!listener.completed.load(Ordering::SeqCst));
    assert!(listener.metadata.lock().unwrap().is_some());
    assert!(!listener.received_deltas.lock().unwrap().is_empty());
    assert!(!core.cancel_stream("stream-to-cancel".to_string()));
}
