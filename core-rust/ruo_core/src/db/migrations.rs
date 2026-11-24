use crate::error::CoreError;
use rusqlite::Connection;

pub fn run_migrations(conn: &mut Connection) -> Result<(), CoreError> {
    // Enable PRAGMA user_version to track migrations
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| CoreError::DatabaseError {
            message: e.to_string(),
            retryable: false,
        })?;

    if version < 1 {
        let tx = conn.transaction().map_err(|e| CoreError::DatabaseError {
            message: e.to_string(),
            retryable: false,
        })?;

        tx.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS local_conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                active_language TEXT NOT NULL DEFAULT 'en',
                study_mode_default INTEGER NOT NULL DEFAULT 0,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS local_messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content_md TEXT NOT NULL,
                status TEXT NOT NULL,
                citations_json TEXT,
                created_at_ms INTEGER NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES local_conversations(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS local_drafts (
                conversation_id TEXT PRIMARY KEY,
                draft_text TEXT NOT NULL,
                updated_at_ms INTEGER NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES local_conversations(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS local_outbox (
                client_msg_id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                content_md TEXT NOT NULL,
                status TEXT NOT NULL, -- pending, sending, failed
                attempts INTEGER NOT NULL DEFAULT 0,
                last_error TEXT,
                created_at_ms INTEGER NOT NULL
            );

            PRAGMA user_version = 1;
            "#,
        )
        .map_err(|e| CoreError::DatabaseError {
            message: e.to_string(),
            retryable: false,
        })?;

        tx.commit().map_err(|e| CoreError::DatabaseError {
            message: e.to_string(),
            retryable: false,
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_apply_cleanly_and_idempotently() {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();

        let v: i32 = conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(v, 1);

        // Running again shouldn't fail
        run_migrations(&mut conn).unwrap();
    }
}
