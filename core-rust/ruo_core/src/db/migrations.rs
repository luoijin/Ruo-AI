use crate::error::CoreError;
use rusqlite::Connection;

const SCHEMA_VERSION: i32 = 3;

pub fn run_migrations(conn: &mut Connection) -> Result<(), CoreError> {
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to read database schema".to_string(),
        })?;

    if version > SCHEMA_VERSION {
        return Err(CoreError::SchemaTooNew {
            details: "Database schema is newer than this app".to_string(),
        });
    }

    if version < 1 {
        let tx = conn.transaction().map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to begin database migration".to_string(),
        })?;

        tx.execute_batch(
            r#"
            CREATE TABLE local_conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                active_language TEXT NOT NULL DEFAULT 'en',
                study_mode_default INTEGER NOT NULL DEFAULT 0,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );

            CREATE TABLE local_messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content_md TEXT NOT NULL,
                status TEXT NOT NULL,
                citations_json TEXT,
                created_at_ms INTEGER NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES local_conversations(id) ON DELETE CASCADE
            );

            CREATE TABLE local_drafts (
                conversation_id TEXT PRIMARY KEY,
                draft_text TEXT NOT NULL,
                updated_at_ms INTEGER NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES local_conversations(id) ON DELETE CASCADE
            );

            CREATE TABLE local_outbox (
                client_msg_id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                content_md TEXT NOT NULL,
                status TEXT NOT NULL,
                attempts INTEGER NOT NULL DEFAULT 0,
                last_error TEXT,
                created_at_ms INTEGER NOT NULL
            );

            PRAGMA user_version = 1;
            "#,
        )
        .map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to create the local database schema".to_string(),
        })?;

        tx.commit().map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to commit database migration".to_string(),
        })?;
    }

    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to read database schema".to_string(),
        })?;
    if version < 2 {
        let tx = conn.transaction().map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to begin database migration".to_string(),
        })?;
        tx.execute_batch(
            r#"
            ALTER TABLE local_conversations ADD COLUMN deleted_at_ms INTEGER;
            ALTER TABLE local_messages ADD COLUMN server_id TEXT;
            ALTER TABLE local_messages ADD COLUMN client_msg_id TEXT;
            ALTER TABLE local_messages ADD COLUMN parent_message_id TEXT;

            UPDATE local_messages
            SET client_msg_id = CASE
                WHEN role = 'user' THEN id
                WHEN role = 'assistant' AND id LIKE 'asst_%' THEN substr(id, 6)
                ELSE NULL
            END;

            CREATE UNIQUE INDEX local_messages_server_id
                ON local_messages(server_id) WHERE server_id IS NOT NULL;
            CREATE UNIQUE INDEX local_messages_client_role
                ON local_messages(client_msg_id, role)
                WHERE client_msg_id IS NOT NULL;
            CREATE INDEX local_messages_conversation_created
                ON local_messages(conversation_id, created_at_ms DESC, id DESC);
            CREATE INDEX local_conversations_updated
                ON local_conversations(updated_at_ms DESC, id DESC);
            CREATE INDEX local_outbox_conversation_status
                ON local_outbox(conversation_id, status, created_at_ms);

            PRAGMA user_version = 2;
            "#,
        )
        .map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to upgrade the local database schema".to_string(),
        })?;
        tx.commit().map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to commit database migration".to_string(),
        })?;
    }

    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to read database schema".to_string(),
        })?;
    if version < 3 {
        let tx = conn.transaction().map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to begin database migration".to_string(),
        })?;
        tx.execute_batch(
            r#"
            ALTER TABLE local_conversations ADD COLUMN updated_at_us INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE local_messages ADD COLUMN created_at_us INTEGER NOT NULL DEFAULT 0;

            UPDATE local_conversations SET updated_at_us = updated_at_ms * 1000;
            UPDATE local_messages SET created_at_us = created_at_ms * 1000;

            CREATE INDEX local_conversations_updated_us
                ON local_conversations(updated_at_us DESC, id DESC);
            CREATE INDEX local_messages_conversation_created_us
                ON local_messages(conversation_id, created_at_us DESC, id DESC);

            PRAGMA user_version = 3;
            "#,
        )
        .map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to upgrade the local database schema".to_string(),
        })?;
        tx.commit().map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to commit database migration".to_string(),
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn migrations_apply_cleanly_and_idempotently() {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();
        run_migrations(&mut conn).unwrap();
        let version: i32 = conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn rejects_a_newer_schema_without_modifying_it() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA user_version = 99;").unwrap();
        assert!(matches!(
            run_migrations(&mut conn),
            Err(CoreError::SchemaTooNew { .. })
        ));
        let version: i32 = conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 99);
    }
}
