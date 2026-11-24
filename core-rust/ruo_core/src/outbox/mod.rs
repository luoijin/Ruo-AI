use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutboxStatus {
    Pending,
    Sending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxItem {
    pub client_msg_id: String,
    pub conversation_id: String,
    pub content_md: String,
    pub status: OutboxStatus,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub created_at_ms: i64,
}

impl OutboxItem {
    pub fn new(client_msg_id: String, conversation_id: String, content_md: String) -> Self {
        Self {
            client_msg_id,
            conversation_id,
            content_md,
            status: OutboxStatus::Pending,
            attempts: 0,
            last_error: None,
            created_at_ms: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn mark_sending(&mut self) {
        self.status = OutboxStatus::Sending;
        self.attempts += 1;
    }

    pub fn mark_failed(&mut self, err: String) {
        self.status = OutboxStatus::Failed;
        self.last_error = Some(err);
    }

    pub fn mark_sent(&mut self) {
        self.status = OutboxStatus::Sent;
        self.last_error = None;
    }
}
