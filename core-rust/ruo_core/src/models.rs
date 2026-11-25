use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum MessageRole {
    User,
    Assistant,
}

impl MessageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

impl TryFrom<&str> for MessageRole {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum LocalMessageStatus {
    Queued,
    Sending,
    Streaming,
    Sent,
    Failed,
    Cancelled,
}

impl LocalMessageStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Sending => "sending",
            Self::Streaming => "streaming",
            Self::Sent => "sent",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for LocalMessageStatus {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "queued" => Ok(Self::Queued),
            "sending" => Ok(Self::Sending),
            "streaming" => Ok(Self::Streaming),
            "sent" | "complete" => Ok(Self::Sent),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct LocalMessage {
    pub id: String,
    #[uniffi(default)]
    pub server_id: Option<String>,
    pub conversation_id: String,
    #[uniffi(default)]
    pub client_msg_id: Option<String>,
    pub role: MessageRole,
    pub content_md: String,
    pub status: LocalMessageStatus,
    #[uniffi(default)]
    pub citations_json: Option<String>,
    #[uniffi(default)]
    pub parent_message_id: Option<String>,
    pub created_at_ms: i64,
    #[uniffi(default)]
    pub created_at_us: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub active_language: String,
    pub study_mode_default: bool,
    #[uniffi(default)]
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    #[uniffi(default)]
    pub updated_at_us: i64,
    #[uniffi(default)]
    pub deleted_at_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct OutboundRequest {
    pub client_msg_id: String,
    pub conversation_id: String,
    pub content: String,
    #[uniffi(default)]
    pub parent_message_id: Option<String>,
    #[uniffi(default)]
    pub attachment_ids: Vec<String>,
    #[uniffi(default)]
    pub intent: Option<String>,
    #[uniffi(default)]
    pub study_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum OutboxState {
    Queued,
    Sending,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct OutboxStatus {
    pub client_msg_id: String,
    pub conversation_id: String,
    pub state: OutboxState,
    pub attempts: u32,
    pub retry_after_sec: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum LocalEntity {
    Conversation,
    Message,
    Draft,
    Outbox,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct LocalChange {
    pub entity: LocalEntity,
    pub id: String,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct HistorySyncConfig {
    #[uniffi(default = 50)]
    pub conversation_page_size: u32,
    #[uniffi(default = 50)]
    pub message_page_size: u32,
    #[uniffi(default = 10)]
    pub offline_recent_conversations: u32,
    #[uniffi(default = 50)]
    pub offline_recent_messages_per_conversation: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct StreamMeta {
    pub user_server_id: String,
    pub assistant_server_id: String,
    #[uniffi(default)]
    pub detected_language: Option<String>,
    #[uniffi(default)]
    pub study_step: Option<i32>,
    #[uniffi(default)]
    pub fallback: bool,
    #[uniffi(default)]
    pub replayed: bool,
}
