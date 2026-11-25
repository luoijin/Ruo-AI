#[cfg(all(feature = "sqlcipher", feature = "plain-sqlite-tests"))]
compile_error!("Choose either `sqlcipher` or `plain-sqlite-tests`, not both.");

#[cfg(not(any(feature = "sqlcipher", feature = "plain-sqlite-tests")))]
compile_error!("Enable `sqlcipher` or the host-only `plain-sqlite-tests` feature.");

#[cfg(all(feature = "plain-sqlite-tests", target_os = "android"))]
compile_error!("`plain-sqlite-tests` is forbidden on Android targets.");

#[cfg(all(feature = "plain-sqlite-tests", not(debug_assertions)))]
compile_error!("`plain-sqlite-tests` is forbidden in release builds.");

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::broadcast;

pub mod db;
pub mod error;
pub mod models;
pub mod net;
pub mod outbox;
pub mod validate;

use db::DbHandle;
pub use error::CoreError;
pub use models::{ConversationSummary, LocalMessage, OutboundRequest};
use net::{ParsedEvent, SseParser};

uniffi::setup_scaffolding!();

#[uniffi::export(foreign)]
pub trait StreamListener: Send + Sync {
    fn on_meta(&self, message_id: String, detected_language: Option<String>, fallback: bool);
    fn on_delta(&self, message_id: String, text: String);
    fn on_citation(&self, message_id: String, attachment_id: String, snippet: Option<String>);
    fn on_complete(&self, message_id: String);
    fn on_error(&self, message_id: String, code: String, retryable: bool);
}

#[uniffi::export(foreign)]
pub trait DraftReadListener: Send + Sync {
    fn on_draft_read(&self, conversation_id: String, text: String);
}

#[derive(uniffi::Object)]
pub struct RuoCore {
    db: Arc<DbHandle>,
    access_token: RwLock<Option<String>>,
    base_url: RwLock<String>,
    cancel_senders: Mutex<HashMap<String, broadcast::Sender<()>>>,
}

#[uniffi::export]
impl RuoCore {
    #[uniffi::constructor]
    pub fn open(db_path: String, mut key: Vec<u8>) -> Result<Arc<Self>, CoreError> {
        let db_result = DbHandle::open(&db_path, &key);
        key.fill(0);
        let db = db_result?;
        Ok(Arc::new(Self {
            db: Arc::new(db),
            access_token: RwLock::new(None),
            base_url: RwLock::new("http://10.0.2.2:8000".to_string()),
            cancel_senders: Mutex::new(HashMap::new()),
        }))
    }

    pub fn set_session(&self, access_token: String, base_url: String) {
        let mut tok = self.access_token.write().unwrap();
        *tok = Some(access_token);
        let mut url = self.base_url.write().unwrap();
        *url = base_url;
    }

    pub fn create_conversation(
        &self,
        id: String,
        title: String,
        active_language: String,
        study_mode_default: bool,
    ) -> Result<(), CoreError> {
        self.db
            .create_conversation(&id, &title, &active_language, study_mode_default)
    }

    pub fn save_draft(&self, conversation_id: String, text: String) -> Result<(), CoreError> {
        self.db.save_draft(&conversation_id, &text)
    }

    pub fn get_draft(&self, conversation_id: String) -> Result<Option<String>, CoreError> {
        self.db.get_draft(&conversation_id)
    }

    pub fn read_draft_with_listener(
        &self,
        conversation_id: String,
        listener: Arc<dyn DraftReadListener>,
    ) -> Result<(), CoreError> {
        let text =
            self.db
                .get_draft(&conversation_id)?
                .ok_or_else(|| CoreError::DatabaseError {
                    details: "Draft not found".to_string(),
                    retryable: false,
                })?;
        listener.on_draft_read(conversation_id, text);
        Ok(())
    }

    pub fn list_conversations(
        &self,
        query: Option<String>,
    ) -> Result<Vec<ConversationSummary>, CoreError> {
        self.db.list_conversations(query.as_deref())
    }

    pub fn queue_message(&self, req: OutboundRequest) -> Result<LocalMessage, CoreError> {
        validate::validate_client_msg_id(&req.client_msg_id)?;
        validate::validate_message_content(&req.content)?;

        let local_msg = LocalMessage {
            id: req.client_msg_id.clone(),
            conversation_id: req.conversation_id.clone(),
            role: "user".to_string(),
            content_md: req.content,
            status: "queued".to_string(),
            citations_json: None,
            created_at_ms: chrono::Utc::now().timestamp_millis(),
        };

        self.db.insert_message(&local_msg)?;
        Ok(local_msg)
    }

    pub fn cancel_stream(&self, message_id: String) -> bool {
        let mut map = self.cancel_senders.lock().unwrap();
        if let Some(sender) = map.remove(&message_id) {
            let _ = sender.send(());
            true
        } else {
            false
        }
    }

    pub fn stream_message(
        &self,
        req: OutboundRequest,
        listener: Arc<dyn StreamListener>,
    ) -> Result<(), CoreError> {
        validate::validate_client_msg_id(&req.client_msg_id)?;
        validate::validate_message_content(&req.content)?;

        let (cancel_tx, mut cancel_rx) = broadcast::channel::<()>(1);
        {
            let mut map = self.cancel_senders.lock().unwrap();
            map.insert(req.client_msg_id.clone(), cancel_tx);
        }

        let msg_id = req.client_msg_id.clone();
        let conv_id = req.conversation_id.clone();
        let token_opt = self.access_token.read().unwrap().clone();
        let base_url = self.base_url.read().unwrap().clone();
        let db = Arc::clone(&self.db);

        // Spawn on Tokio runtime for SSE streaming
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let url = format!("{}/v1/conversations/{}/messages", base_url, conv_id);

            let mut request_builder = client.post(&url);
            if let Some(token) = token_opt {
                request_builder = request_builder.bearer_auth(token);
            }

            let body_json = serde_json::json!({
                "client_msg_id": msg_id,
                "content": req.content,
                "parent_message_id": req.parent_message_id,
                "attachment_ids": req.attachment_ids,
                "intent": req.intent,
                "study_mode": req.study_mode,
            });

            let response = match request_builder.json(&body_json).send().await {
                Ok(resp) => resp,
                Err(_e) => {
                    listener.on_error(msg_id, "NETWORK_ERROR".to_string(), true);
                    return;
                }
            };

            let mut stream = response.bytes_stream();
            let mut parser = SseParser::new();
            let mut full_response = String::new();

            use futures_util::StreamExt;

            loop {
                tokio::select! {
                    _ = cancel_rx.recv() => {
                        listener.on_error(msg_id, "CANCELLED".to_string(), false);
                        return;
                    }
                    chunk_opt = stream.next() => {
                        match chunk_opt {
                            Some(Ok(bytes)) => {
                                let chunk_str = String::from_utf8_lossy(&bytes);
                                let events = parser.feed(&chunk_str);
                                for ev in events {
                                    match ev {
                                        ParsedEvent::Meta { message_id, detected_language, fallback, .. } => {
                                            listener.on_meta(message_id, detected_language, fallback);
                                        }
                                        ParsedEvent::Delta { text } => {
                                            full_response.push_str(&text);
                                            listener.on_delta(msg_id.clone(), text);
                                        }
                                        ParsedEvent::Citation { attachment_id, snippet, .. } => {
                                            listener.on_citation(msg_id.clone(), attachment_id, snippet);
                                        }
                                        ParsedEvent::Done { .. } => {
                                            let asst_msg = LocalMessage {
                                                id: format!("asst_{}", msg_id),
                                                conversation_id: conv_id.clone(),
                                                role: "assistant".to_string(),
                                                content_md: full_response.clone(),
                                                status: "sent".to_string(),
                                                citations_json: None,
                                                created_at_ms: chrono::Utc::now().timestamp_millis(),
                                            };
                                            let _ = db.insert_message(&asst_msg);
                                            listener.on_complete(msg_id.clone());
                                            return;
                                        }
                                        ParsedEvent::Error { code, message: _, retry_after_sec: _ } => {
                                            listener.on_error(msg_id.clone(), code, true);
                                            return;
                                        }
                                        ParsedEvent::Unknown { .. } => {}
                                    }
                                }
                            }
                            Some(Err(_)) => {
                                listener.on_error(msg_id, "STREAM_READ_ERROR".to_string(), true);
                                return;
                            }
                            None => {
                                listener.on_complete(msg_id);
                                return;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_open_drafts_and_cancel() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("test_ruo.db");
        let key = vec![0x42; 32];

        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), key).unwrap();

        // Create conversation first (satisfies foreign key)
        core.create_conversation(
            "conv-1".to_string(),
            "Math Review".to_string(),
            "en".to_string(),
            false,
        )
        .unwrap();

        // Test draft save & retrieve
        core.save_draft("conv-1".to_string(), "Isang test draft".to_string())
            .unwrap();
        let draft = core.get_draft("conv-1".to_string()).unwrap();
        assert_eq!(draft.as_deref(), Some("Isang test draft"));

        // Test message queue
        let req = OutboundRequest {
            client_msg_id: "client-id-001".to_string(),
            conversation_id: "conv-1".to_string(),
            content: "Paano mag-factor ng quadratic equation?".to_string(),
            parent_message_id: None,
            attachment_ids: vec![],
            intent: None,
            study_mode: false,
        };
        let queued = core.queue_message(req).unwrap();
        assert_eq!(queued.status, "queued");

        // Test cancel registration & trigger
        {
            let (tx, _) = broadcast::channel(1);
            let mut map = core.cancel_senders.lock().unwrap();
            map.insert("client-id-001".to_string(), tx);
        }
        let cancelled = core.cancel_stream("client-id-001".to_string());
        assert!(cancelled);
        let cancelled_again = core.cancel_stream("client-id-001".to_string());
        assert!(!cancelled_again);
    }
}
