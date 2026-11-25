use thiserror::Error;

#[derive(Error, Debug, uniffi::Error)]
pub enum CoreError {
    #[error("Authentication is required")]
    Unauthenticated { details: String },

    #[error("Quota exceeded")]
    QuotaExceeded {
        details: String,
        retry_after_sec: Option<u64>,
    },

    #[error("Request is rate limited")]
    RateLimited {
        details: String,
        retry_after_sec: Option<u64>,
    },

    #[error("Network request failed")]
    Network {
        details: String,
        retryable: bool,
        retry_after_sec: Option<u64>,
    },

    #[error("Operation cancelled")]
    Cancelled { details: String },

    #[error("Input validation failed")]
    Validation { details: String },

    #[error("Resource was not found")]
    NotFound { details: String },

    #[error("Operation conflicts with current state")]
    Conflict { details: String },

    #[error("Database key is unavailable")]
    KeyUnavailable { details: String },

    #[error("Database key is invalid")]
    InvalidKey { details: String },

    #[error("Database key is incorrect")]
    WrongKey { details: String },

    #[error("Database is corrupt")]
    DatabaseCorrupt { details: String },

    #[error("Database schema is newer than this core supports")]
    SchemaTooNew { details: String },

    #[error("Internal operation failed")]
    Internal {
        details: String,
        retryable: bool,
        retry_after_sec: Option<u64>,
    },
}

impl CoreError {
    pub fn from_code(code: &str, retryable: bool, retry_after_sec: Option<u64>) -> Self {
        const DETAILS: &str = "The server rejected the request";
        match code {
            "UNAUTHENTICATED" | "INVALID_TOKEN" => Self::Unauthenticated {
                details: "Sign-in is required".to_string(),
            },
            "QUOTA_EXCEEDED" => Self::QuotaExceeded {
                details: DETAILS.to_string(),
                retry_after_sec,
            },
            "RATE_LIMITED" | "RATE_LIMIT_EXCEEDED" => Self::RateLimited {
                details: DETAILS.to_string(),
                retry_after_sec,
            },
            "NOT_FOUND" | "CONVERSATION_NOT_FOUND" => Self::NotFound {
                details: DETAILS.to_string(),
            },
            "CONFLICT"
            | "CONVERSATION_ID_CONFLICT"
            | "IDEMPOTENCY_CONFLICT"
            | "STREAM_IN_PROGRESS" => Self::Conflict {
                details: DETAILS.to_string(),
            },
            "VALIDATION" | "INVALID_CURSOR" => Self::Validation {
                details: DETAILS.to_string(),
            },
            "CANCELLED" => Self::Cancelled {
                details: "The operation was cancelled".to_string(),
            },
            _ => Self::Internal {
                details: DETAILS.to_string(),
                retryable,
                retry_after_sec,
            },
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Unauthenticated { .. } => "UNAUTHENTICATED",
            Self::QuotaExceeded { .. } => "QUOTA_EXCEEDED",
            Self::RateLimited { .. } => "RATE_LIMITED",
            Self::Network { .. } => "NETWORK",
            Self::Cancelled { .. } => "CANCELLED",
            Self::Validation { .. } => "VALIDATION",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict { .. } => "CONFLICT",
            Self::KeyUnavailable { .. } => "KEY_UNAVAILABLE",
            Self::InvalidKey { .. } => "INVALID_KEY",
            Self::WrongKey { .. } => "WRONG_KEY",
            Self::DatabaseCorrupt { .. } => "DATABASE_CORRUPT",
            Self::SchemaTooNew { .. } => "SCHEMA_TOO_NEW",
            Self::Internal { .. } => "INTERNAL",
        }
    }

    pub fn retryable(&self) -> bool {
        match self {
            Self::RateLimited { .. } => true,
            Self::Network { retryable, .. } | Self::Internal { retryable, .. } => *retryable,
            Self::Unauthenticated { .. }
            | Self::QuotaExceeded { .. }
            | Self::Cancelled { .. }
            | Self::Validation { .. }
            | Self::NotFound { .. }
            | Self::Conflict { .. }
            | Self::KeyUnavailable { .. }
            | Self::InvalidKey { .. }
            | Self::WrongKey { .. }
            | Self::DatabaseCorrupt { .. }
            | Self::SchemaTooNew { .. } => false,
        }
    }

    pub fn retry_after_sec(&self) -> Option<u64> {
        match self {
            Self::QuotaExceeded {
                retry_after_sec, ..
            }
            | Self::RateLimited {
                retry_after_sec, ..
            }
            | Self::Network {
                retry_after_sec, ..
            }
            | Self::Internal {
                retry_after_sec, ..
            } => *retry_after_sec,
            _ => None,
        }
    }
}
