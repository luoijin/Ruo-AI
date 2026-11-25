use thiserror::Error;

#[derive(Error, Debug, uniffi::Error)]
pub enum CoreError {
    #[error("Database error: {details}")]
    DatabaseError { details: String, retryable: bool },

    #[error("Network error: {details}")]
    NetworkError { details: String, retryable: bool },

    #[error("Stream parse error: {details}")]
    ParseError { details: String, retryable: bool },

    #[error("Validation error: {details}")]
    ValidationError { details: String },

    #[error("Stream cancelled: {details}")]
    Cancelled { details: String },
}
