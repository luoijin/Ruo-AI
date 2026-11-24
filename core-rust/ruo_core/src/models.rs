use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct LocalMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String, // "user" | "assistant"
    pub content_md: String,
    pub status: String, // "queued" | "streaming" | "sent" | "failed"
    pub citations_json: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub active_language: String,
    pub study_mode_default: bool,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct OutboundRequest {
    pub client_msg_id: String,
    pub conversation_id: String,
    pub content: String,
    pub parent_message_id: Option<String>,
    pub attachment_ids: Vec<String>,
    pub intent: Option<String>,
    pub study_mode: bool,
}
