use thiserror::Error;

#[derive(Error, Debug, uniffi::Error)]
pub enum CoreError {
    #[error("Database error: {message}")]
    DatabaseError { message: String, retryable: bool },

    #[error("Network error: {message}")]
    NetworkError { message: String, retryable: bool },

    #[error("Stream parse error: {message}")]
    ParseError { message: String, retryable: bool },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Stream cancelled: {message}")]
    Cancelled { message: String },
}
