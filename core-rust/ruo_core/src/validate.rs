use crate::error::CoreError;

pub fn validate_message_content(content: &str) -> Result<(), CoreError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(CoreError::ValidationError {
            message: "Message content cannot be empty".to_string(),
        });
    }
    if trimmed.len() > 32_000 {
        return Err(CoreError::ValidationError {
            message: "Message content exceeds maximum allowed length of 32,000 characters"
                .to_string(),
        });
    }
    Ok(())
}

pub fn validate_client_msg_id(client_msg_id: &str) -> Result<(), CoreError> {
    if client_msg_id.trim().is_empty() {
        return Err(CoreError::ValidationError {
            message: "client_msg_id cannot be empty".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validations() {
        assert!(validate_message_content("").is_err());
        assert!(validate_message_content("   ").is_err());
        assert!(validate_message_content("Kumusta!").is_ok());
        assert!(validate_client_msg_id("").is_err());
        assert!(validate_client_msg_id("uuid-1234").is_ok());
    }
}
