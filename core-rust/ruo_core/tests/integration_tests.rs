use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use ruo_core::db::DbHandle;
use ruo_core::models::{LocalMessage, OutboundRequest};
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
    let key = b"key_bytes_1234567890_abcdefghij".to_vec();

    let db = DbHandle::open(db_path.to_str().unwrap(), &key).unwrap();

    // Create 2 conversations
    db.create_conversation("c1", "Biology", "en", false)
        .unwrap();
    db.create_conversation("c2", "Calculus Integrals", "tl-PH", true)
        .unwrap();

    // Search
    let list_all = db.list_conversations(None).unwrap();
    assert_eq!(list_all.len(), 2);

    let list_filtered = db.list_conversations(Some("Calculus")).unwrap();
    assert_eq!(list_filtered.len(), 1);
    assert_eq!(list_filtered[0].title, "Calculus Integrals");

    // Insert messages
    let msg1 = LocalMessage {
        id: "m1".to_string(),
        conversation_id: "c1".to_string(),
        role: "user".to_string(),
        content_md: "What is mitosis?".to_string(),
        status: "sent".to_string(),
        citations_json: None,
        created_at_ms: 1000,
    };
    db.insert_message(&msg1).unwrap();

    // Message upsert/update
    let mut msg1_updated = msg1.clone();
    msg1_updated.status = "delivered".to_string();
    db.insert_message(&msg1_updated).unwrap();
}

struct TestStreamListener {
    pub received_deltas: Arc<Mutex<Vec<String>>>,
    pub completed: Arc<AtomicBool>,
    pub error_code: Arc<Mutex<Option<String>>>,
}

impl StreamListener for TestStreamListener {
    fn on_meta(&self, _message_id: String, _detected_language: Option<String>, _fallback: bool) {}
    fn on_delta(&self, _message_id: String, text: String) {
        let deltas = Arc::clone(&self.received_deltas);
        tokio::spawn(async move {
            deltas.lock().await.push(text);
        });
    }
    fn on_citation(&self, _message_id: String, _attachment_id: String, _snippet: Option<String>) {}
    fn on_complete(&self, _message_id: String) {
        self.completed.store(true, Ordering::SeqCst);
    }
    fn on_error(&self, _message_id: String, code: String, _retryable: bool) {
        let err = Arc::clone(&self.error_code);
        tokio::spawn(async move {
            *err.lock().await = Some(code);
        });
    }
}

#[tokio::test]
async fn test_sse_mock_stream_and_cancel_mid_flight() {
    use warp::Filter;

    // Start a mock SSE server
    let sse_route = warp::path!("v1" / "conversations" / String / "messages")
        .and(warp::post())
        .map(|_conv_id| {
            let stream = async_stream::stream! {
                yield Ok::<_, warp::Error>(warp::sse::Event::default().event("meta").data(r#"{"message_id":"msg-mock","fallback":false}"#));
                for i in 1..=10 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    yield Ok::<_, warp::Error>(warp::sse::Event::default().event("delta").data(format!(r#"{{"text":"Chunk {i} "}}"#)));
                }
                yield Ok::<_, warp::Error>(warp::sse::Event::default().event("done").data(r#"{"finish_reason":"stop"}"#));
            };
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        });

    let (addr, server) = warp::serve(sse_route).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::spawn(server);

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("mock_test.db");
    let key = b"key_bytes_1234567890_abcdefghij".to_vec();

    let core = RuoCore::open(db_path.to_str().unwrap().to_string(), key).unwrap();
    core.create_conversation(
        "c-mock".to_string(),
        "Mock".to_string(),
        "en".to_string(),
        false,
    )
    .unwrap();
    core.set_session("mock-token".to_string(), format!("http://{}", addr));

    let deltas = Arc::new(Mutex::new(Vec::new()));
    let completed = Arc::new(AtomicBool::new(false));
    let error_code = Arc::new(Mutex::new(None));

    let listener = Arc::new(TestStreamListener {
        received_deltas: Arc::clone(&deltas),
        completed: Arc::clone(&completed),
        error_code: Arc::clone(&error_code),
    });

    let req = OutboundRequest {
        client_msg_id: "stream-to-cancel".to_string(),
        conversation_id: "c-mock".to_string(),
        content: "Please start streaming".to_string(),
        parent_message_id: None,
        attachment_ids: vec![],
        intent: None,
        study_mode: false,
    };

    core.stream_message(req, listener).unwrap();

    // Let 1 or 2 chunks arrive
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Trigger cancel mid-stream
    let cancelled = core.cancel_stream("stream-to-cancel".to_string());
    assert!(cancelled);

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Should have reported CANCELLED error
    let err = error_code.lock().await;
    assert_eq!(err.as_deref(), Some("CANCELLED"));
    assert!(!completed.load(Ordering::SeqCst));
}
