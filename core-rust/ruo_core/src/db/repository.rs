use rusqlite::{params, Connection, OptionalExtension, Transaction};
use std::path::Path;
use std::sync::Mutex;

use crate::error::CoreError;
use crate::models::{
    ConversationSummary, LocalMessage, LocalMessageStatus, MessageRole, OutboundRequest,
    OutboxState, OutboxStatus,
};

pub struct DbHandle {
    conn: Mutex<Connection>,
}

impl DbHandle {
    pub fn open(path: &str, key: &[u8]) -> Result<Self, CoreError> {
        if key.len() != 32 {
            return Err(CoreError::InvalidKey {
                details: "Database key must contain 32 bytes".to_string(),
            });
        }

        let existed = Path::new(path).exists();
        let mut conn = Connection::open(path).map_err(|_| CoreError::DatabaseCorrupt {
            details: "Unable to open the local database".to_string(),
        })?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|_| CoreError::DatabaseCorrupt {
                details: "Unable to configure the local database".to_string(),
            })?;

        #[cfg(feature = "sqlcipher")]
        {
            let key_hex = key
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            conn.execute_batch(&format!("PRAGMA key = \"x'{key_hex}'\";"))
                .map_err(|_| CoreError::WrongKey {
                    details: "Database key could not be applied".to_string(),
                })?;
        }
        let probe = conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        });
        if probe.is_err() {
            return Err(if existed {
                CoreError::WrongKey {
                    details: "Database could not be opened with this key".to_string(),
                }
            } else {
                CoreError::DatabaseCorrupt {
                    details: "Unable to initialize the local database".to_string(),
                }
            });
        }

        crate::db::migrations::run_migrations(&mut conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn create_conversation(
        &self,
        id: &str,
        title: &str,
        active_language: &str,
        study_mode_default: bool,
    ) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        let now = chrono::Utc::now();
        let now_ms = now.timestamp_millis();
        let now_us = now.timestamp_micros();
        conn.execute(
            r#"
            INSERT INTO local_conversations
                (id, title, active_language, study_mode_default, created_at_ms, updated_at_ms, updated_at_us)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                active_language = excluded.active_language,
                study_mode_default = excluded.study_mode_default,
                updated_at_ms = excluded.updated_at_ms,
                updated_at_us = excluded.updated_at_us,
                deleted_at_ms = NULL;
            "#,
            params![
                id,
                title,
                active_language,
                study_mode_default as i32,
                now_ms,
                now_ms,
                now_us
            ],
        )
        .map_err(|_| internal_db_error())?;
        Ok(())
    }

    pub fn upsert_conversation(&self, conversation: &ConversationSummary) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        conn.execute(
            r#"
            INSERT INTO local_conversations
                (id, title, active_language, study_mode_default, created_at_ms, updated_at_ms, deleted_at_ms, updated_at_us)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                active_language = excluded.active_language,
                study_mode_default = excluded.study_mode_default,
                updated_at_ms = excluded.updated_at_ms,
                updated_at_us = excluded.updated_at_us,
                deleted_at_ms = excluded.deleted_at_ms;
            "#,
            params![
                conversation.id,
                conversation.title,
                conversation.active_language,
                conversation.study_mode_default as i32,
                if conversation.created_at_ms == 0 {
                    conversation.updated_at_ms
                } else {
                    conversation.created_at_ms
                },
                conversation.updated_at_ms,
                conversation.deleted_at_ms,
                if conversation.updated_at_us == 0 {
                    conversation.updated_at_ms * 1000
                } else {
                    conversation.updated_at_us
                }
            ],
        )
        .map_err(|_| internal_db_error())?;
        Ok(())
    }

    pub fn save_draft(&self, conversation_id: &str, text: &str) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        let now_ms = chrono::Utc::now().timestamp_millis();
        conn.execute(
            r#"
            INSERT INTO local_drafts (conversation_id, draft_text, updated_at_ms)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(conversation_id) DO UPDATE SET
                draft_text = excluded.draft_text,
                updated_at_ms = excluded.updated_at_ms;
            "#,
            params![conversation_id, text, now_ms],
        )
        .map_err(|_| internal_db_error())?;
        Ok(())
    }

    pub fn delete_conversation_local(&self, id: &str) -> Result<(), CoreError> {
        let now = chrono::Utc::now();
        let now_ms = now.timestamp_millis();
        let now_us = now.timestamp_micros();
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        conn.execute(
            "UPDATE local_conversations SET deleted_at_ms = ?1, updated_at_ms = ?1, updated_at_us = ?2 WHERE id = ?3",
            params![now_ms, now_us, id],
        )
        .map_err(|_| internal_db_error())?;
        Ok(())
    }

    pub fn get_draft(&self, conversation_id: &str) -> Result<Option<String>, CoreError> {
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        conn.query_row(
            "SELECT draft_text FROM local_drafts WHERE conversation_id = ?1",
            params![conversation_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| internal_db_error())
    }

    pub fn queue_message(&self, request: &OutboundRequest) -> Result<LocalMessage, CoreError> {
        let mut conn = self.conn.lock().map_err(|_| internal_db_error())?;
        let tx = conn.transaction().map_err(|_| internal_db_error())?;
        ensure_conversation(&tx, &request.conversation_id)?;
        let now = chrono::Utc::now();
        let now_ms = now.timestamp_millis();
        let now_us = now.timestamp_micros();
        let message = LocalMessage {
            id: request.client_msg_id.clone(),
            server_id: None,
            conversation_id: request.conversation_id.clone(),
            client_msg_id: Some(request.client_msg_id.clone()),
            role: MessageRole::User,
            content_md: request.content.clone(),
            status: LocalMessageStatus::Queued,
            citations_json: None,
            parent_message_id: request.parent_message_id.clone(),
            created_at_ms: now_ms,
            created_at_us: now_us,
        };
        insert_message(&tx, &message)?;
        tx.execute(
            r#"
            INSERT INTO local_outbox
                (client_msg_id, conversation_id, content_md, status, attempts, last_error, created_at_ms)
            VALUES (?1, ?2, ?3, 'pending', 0, NULL, ?4)
            ON CONFLICT(client_msg_id) DO UPDATE SET
                content_md = excluded.content_md,
                status = 'pending',
                last_error = NULL;
            "#,
            params![
                request.client_msg_id,
                request.conversation_id,
                request.content,
                now_ms
            ],
        )
        .map_err(|_| internal_db_error())?;
        tx.commit().map_err(|_| internal_db_error())?;
        Ok(message)
    }

    pub fn begin_stream(&self, request: &OutboundRequest) -> Result<(), CoreError> {
        let mut conn = self.conn.lock().map_err(|_| internal_db_error())?;
        let tx = conn.transaction().map_err(|_| internal_db_error())?;
        ensure_conversation(&tx, &request.conversation_id)?;
        let now = chrono::Utc::now();
        let now_ms = now.timestamp_millis();
        let now_us = now.timestamp_micros();
        let user = LocalMessage {
            id: request.client_msg_id.clone(),
            server_id: None,
            conversation_id: request.conversation_id.clone(),
            client_msg_id: Some(request.client_msg_id.clone()),
            role: MessageRole::User,
            content_md: request.content.clone(),
            status: LocalMessageStatus::Sending,
            citations_json: None,
            parent_message_id: request.parent_message_id.clone(),
            created_at_ms: now_ms,
            created_at_us: now_us,
        };
        insert_message(&tx, &user)?;
        let assistant = LocalMessage {
            id: format!("assistant:{}", request.client_msg_id),
            server_id: None,
            conversation_id: request.conversation_id.clone(),
            client_msg_id: Some(request.client_msg_id.clone()),
            role: MessageRole::Assistant,
            content_md: String::new(),
            status: LocalMessageStatus::Streaming,
            citations_json: None,
            parent_message_id: None,
            created_at_ms: now_ms,
            created_at_us: now_us,
        };
        insert_message(&tx, &assistant)?;
        tx.execute(
            r#"
            INSERT INTO local_outbox
                (client_msg_id, conversation_id, content_md, status, attempts, last_error, created_at_ms)
            VALUES (?1, ?2, ?3, 'sending', 1, NULL, ?4)
            ON CONFLICT(client_msg_id) DO UPDATE SET
                status = 'sending',
                attempts = attempts + 1,
                last_error = NULL;
            "#,
            params![
                request.client_msg_id,
                request.conversation_id,
                request.content,
                now_ms
            ],
        )
        .map_err(|_| internal_db_error())?;
        tx.commit().map_err(|_| internal_db_error())
    }

    pub fn update_assistant_delta(
        &self,
        client_msg_id: &str,
        content: &str,
    ) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        conn.execute(
            "UPDATE local_messages SET content_md = ?1, status = 'streaming' WHERE client_msg_id = ?2 AND role = 'assistant'",
            params![content, client_msg_id],
        )
        .map_err(|_| internal_db_error())?;
        Ok(())
    }

    pub fn set_server_message_ids(
        &self,
        client_msg_id: &str,
        user_server_id: &str,
        assistant_server_id: &str,
    ) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        conn.execute(
            "UPDATE local_messages SET server_id = ?1 WHERE client_msg_id = ?2 AND role = 'user'",
            params![user_server_id, client_msg_id],
        )
        .map_err(|_| internal_db_error())?;
        conn.execute(
            "UPDATE local_messages SET server_id = ?1 WHERE client_msg_id = ?2 AND role = 'assistant'",
            params![assistant_server_id, client_msg_id],
        )
        .map_err(|_| internal_db_error())?;
        Ok(())
    }

    pub fn finish_stream(
        &self,
        client_msg_id: &str,
        assistant_server_id: &str,
        content: &str,
    ) -> Result<(), CoreError> {
        let mut conn = self.conn.lock().map_err(|_| internal_db_error())?;
        let tx = conn.transaction().map_err(|_| internal_db_error())?;
        tx.execute(
            "UPDATE local_messages SET server_id = ?1, content_md = ?2, status = 'sent' WHERE client_msg_id = ?3 AND role = 'assistant'",
            params![assistant_server_id, content, client_msg_id],
        )
        .map_err(|_| internal_db_error())?;
        tx.execute(
            "UPDATE local_messages SET status = 'sent' WHERE client_msg_id = ?1 AND role = 'user'",
            params![client_msg_id],
        )
        .map_err(|_| internal_db_error())?;
        tx.execute(
            "UPDATE local_outbox SET status = 'sent', last_error = NULL WHERE client_msg_id = ?1",
            params![client_msg_id],
        )
        .map_err(|_| internal_db_error())?;
        tx.commit().map_err(|_| internal_db_error())
    }

    pub fn fail_stream(&self, client_msg_id: &str, cancelled: bool) -> Result<(), CoreError> {
        let status = if cancelled { "cancelled" } else { "failed" };
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        conn.execute(
            "UPDATE local_messages SET status = ?1 WHERE client_msg_id = ?2",
            params![status, client_msg_id],
        )
        .map_err(|_| internal_db_error())?;
        conn.execute(
            "UPDATE local_outbox SET status = 'failed' WHERE client_msg_id = ?1",
            params![client_msg_id],
        )
        .map_err(|_| internal_db_error())?;
        Ok(())
    }

    pub fn insert_message(&self, message: &LocalMessage) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        insert_message(&conn, message)
    }

    pub fn upsert_message(&self, message: &LocalMessage) -> Result<(), CoreError> {
        self.insert_message(message)
    }

    pub fn list_conversations(
        &self,
        query: Option<&str>,
        limit: u32,
        before_updated_at_us: Option<i64>,
        before_id: Option<&str>,
        include_deleted: bool,
    ) -> Result<Vec<ConversationSummary>, CoreError> {
        if before_updated_at_us.is_some() != before_id.is_some() {
            return Err(CoreError::Validation {
                details: "Both conversation cursor values are required".to_string(),
            });
        }
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        let query_pattern = query.map(|q| format!("%{q}%"));
        let mut statement = conn
            .prepare(
                r#"
                SELECT id, title, active_language, study_mode_default, created_at_ms, updated_at_ms, updated_at_us, deleted_at_ms
                FROM local_conversations
                WHERE (?1 IS NULL OR title LIKE ?1)
                  AND (?2 = 1 OR deleted_at_ms IS NULL)
                  AND (?3 IS NULL OR updated_at_us < ?3 OR (updated_at_us = ?3 AND id < ?4))
                ORDER BY updated_at_us DESC, id DESC
                LIMIT ?5
                "#,
            )
            .map_err(|_| internal_db_error())?;
        let rows = statement
            .query_map(
                params![
                    query_pattern,
                    include_deleted as i32,
                    before_updated_at_us,
                    before_id,
                    limit.clamp(1, 100)
                ],
                |row| {
                    Ok(ConversationSummary {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        active_language: row.get(2)?,
                        study_mode_default: row.get::<_, i32>(3)? != 0,
                        created_at_ms: row.get(4)?,
                        updated_at_ms: row.get(5)?,
                        updated_at_us: row.get(6)?,
                        deleted_at_ms: row.get(7)?,
                    })
                },
            )
            .map_err(|_| internal_db_error())?;
        rows.map(|row| row.map_err(|_| internal_db_error()))
            .collect()
    }

    pub fn list_messages(
        &self,
        conversation_id: &str,
        limit: u32,
        before_created_at_us: Option<i64>,
        before_id: Option<&str>,
    ) -> Result<Vec<LocalMessage>, CoreError> {
        if before_created_at_us.is_some() != before_id.is_some() {
            return Err(CoreError::Validation {
                details: "Both message cursor values are required".to_string(),
            });
        }
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        let mut statement = conn
            .prepare(
                r#"
                SELECT id, server_id, conversation_id, client_msg_id, role, content_md, status,
                       citations_json, parent_message_id, created_at_ms, created_at_us
                FROM local_messages
                WHERE conversation_id = ?1
                  AND (?2 IS NULL OR created_at_us < ?2 OR (created_at_us = ?2 AND id < ?3))
                ORDER BY created_at_us DESC, id DESC
                LIMIT ?4
                "#,
            )
            .map_err(|_| internal_db_error())?;
        let rows = statement
            .query_map(
                params![
                    conversation_id,
                    before_created_at_us,
                    before_id,
                    limit.clamp(1, 100)
                ],
                |row| {
                    let role: String = row.get(4)?;
                    let status: String = row.get(6)?;
                    Ok(LocalMessage {
                        id: row.get(0)?,
                        server_id: row.get(1)?,
                        conversation_id: row.get(2)?,
                        client_msg_id: row.get(3)?,
                        role: MessageRole::try_from(role.as_str()).map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                4,
                                "role".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?,
                        content_md: row.get(5)?,
                        status: LocalMessageStatus::try_from(status.as_str()).map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                6,
                                "status".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?,
                        citations_json: row.get(7)?,
                        parent_message_id: row.get(8)?,
                        created_at_ms: row.get(9)?,
                        created_at_us: row.get(10)?,
                    })
                },
            )
            .map_err(|_| internal_db_error())?;
        rows.map(|row| row.map_err(|_| internal_db_error()))
            .collect()
    }

    pub fn outbox_state(
        &self,
        conversation_id: Option<&str>,
    ) -> Result<Vec<OutboxStatus>, CoreError> {
        let conn = self.conn.lock().map_err(|_| internal_db_error())?;
        let mut statement = conn
            .prepare(
                r#"
                SELECT client_msg_id, conversation_id, status, attempts
                FROM local_outbox
                WHERE status IN ('pending', 'sending', 'failed')
                  AND (?1 IS NULL OR conversation_id = ?1)
                ORDER BY created_at_ms, client_msg_id
                "#,
            )
            .map_err(|_| internal_db_error())?;
        let rows = statement
            .query_map(params![conversation_id], |row| {
                let state: String = row.get(2)?;
                let state = match state.as_str() {
                    "sending" => OutboxState::Sending,
                    "failed" => OutboxState::Failed,
                    _ => OutboxState::Queued,
                };
                Ok(OutboxStatus {
                    client_msg_id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    state,
                    attempts: row.get::<_, i64>(3)?.max(0) as u32,
                    retry_after_sec: None,
                })
            })
            .map_err(|_| internal_db_error())?;
        rows.map(|row| row.map_err(|_| internal_db_error()))
            .collect()
    }
}

fn ensure_conversation(tx: &Transaction<'_>, conversation_id: &str) -> Result<(), CoreError> {
    let exists: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM local_conversations WHERE id = ?1 AND deleted_at_ms IS NULL)",
            params![conversation_id],
            |row| row.get(0),
        )
        .map_err(|_| internal_db_error())?;
    if !exists {
        return Err(CoreError::NotFound {
            details: "Conversation is not available locally".to_string(),
        });
    }
    Ok(())
}

fn insert_message(conn: &Connection, message: &LocalMessage) -> Result<(), CoreError> {
    conn.execute(
        r#"
        INSERT INTO local_messages
            (id, server_id, conversation_id, client_msg_id, role, content_md, status,
             citations_json, parent_message_id, created_at_ms, created_at_us)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(id) DO UPDATE SET
            server_id = excluded.server_id,
            client_msg_id = excluded.client_msg_id,
            content_md = excluded.content_md,
            status = excluded.status,
            citations_json = excluded.citations_json,
            parent_message_id = excluded.parent_message_id;
        "#,
        params![
            message.id,
            message.server_id,
            message.conversation_id,
            message.client_msg_id,
            message.role.as_str(),
            message.content_md,
            message.status.as_str(),
            message.citations_json,
            message.parent_message_id,
            message.created_at_ms,
            if message.created_at_us == 0 {
                message.created_at_ms * 1000
            } else {
                message.created_at_us
            }
        ],
    )
    .map_err(|_| internal_db_error())?;
    Ok(())
}

fn internal_db_error() -> CoreError {
    CoreError::DatabaseCorrupt {
        details: "Local database operation failed".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn sample_message(id: &str, content: &str) -> LocalMessage {
        LocalMessage {
            id: id.to_string(),
            server_id: None,
            conversation_id: "conv-1".to_string(),
            client_msg_id: Some(id.to_string()),
            role: MessageRole::User,
            content_md: content.to_string(),
            status: LocalMessageStatus::Queued,
            citations_json: None,
            parent_message_id: None,
            created_at_ms: 1,
            created_at_us: if id == "m1" { 1_001 } else { 1_002 },
        }
    }

    #[test]
    #[cfg(feature = "sqlcipher")]
    fn encrypted_database_round_trips_and_rejects_wrong_or_missing_keys() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("encrypted.db");
        let key = [0x42; 32];
        let expected = "Taglish: Kumusta, paano ito? Cebuano: Maayong buntag, unsaon ni? 🌱📚";

        let db = DbHandle::open(db_path.to_str().unwrap(), &key).unwrap();
        db.create_conversation("conv-1", "Study", "en", false)
            .unwrap();
        db.save_draft("conv-1", expected).unwrap();
        drop(db);

        let bytes = fs::read(&db_path).unwrap();
        assert!(bytes.len() >= 16);
        assert_ne!(&bytes[..16], b"SQLite format 3\0");

        let db = DbHandle::open(db_path.to_str().unwrap(), &key).unwrap();
        assert_eq!(db.get_draft("conv-1").unwrap().as_deref(), Some(expected));
        drop(db);

        let wrong_key = [0x24; 32];
        assert!(matches!(
            DbHandle::open(db_path.to_str().unwrap(), &wrong_key),
            Err(CoreError::WrongKey { .. })
        ));
        assert!(matches!(
            DbHandle::open(db_path.to_str().unwrap(), &[]),
            Err(CoreError::InvalidKey { .. })
        ));
        assert!(!temp_dir.path().join("missing-key.db").exists());
    }

    #[test]
    #[cfg(feature = "plain-sqlite-tests")]
    fn logic_test_database_uses_plain_sqlite() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("logic-test.db");
        let key = [0x42; 32];

        let _db = DbHandle::open(db_path.to_str().unwrap(), &key).unwrap();
        let bytes = fs::read(&db_path).unwrap();
        assert!(bytes.len() >= 16);
        assert_eq!(&bytes[..16], b"SQLite format 3\0");
    }

    #[test]
    fn local_message_reads_preserve_enum_types_and_keyset_order() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("messages.db");
        let db = DbHandle::open(path.to_str().unwrap(), &[0x42; 32]).unwrap();
        db.create_conversation("conv-1", "Study", "en", false)
            .unwrap();
        db.insert_message(&sample_message("m1", "one")).unwrap();
        db.insert_message(&sample_message("m2", "two")).unwrap();
        let page = db.list_messages("conv-1", 1, None, None).unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].role, MessageRole::User);
        assert_eq!(page[0].content_md, "two");
        let next_page = db
            .list_messages(
                "conv-1",
                1,
                Some(page[0].created_at_us),
                Some(page[0].id.as_str()),
            )
            .unwrap();
        assert_eq!(next_page[0].content_md, "one");
    }

    #[test]
    fn conversation_keyset_uses_full_timestamp_precision() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("conversations.db");
        let db = DbHandle::open(path.to_str().unwrap(), &[0x42; 32]).unwrap();
        db.create_conversation("c1", "First", "en", false).unwrap();
        db.create_conversation("c2", "Second", "en", false).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE local_conversations SET updated_at_ms = 1, updated_at_us = 1001 WHERE id = 'c1'",
                [],
            )
            .unwrap();
            conn.execute(
                "UPDATE local_conversations SET updated_at_ms = 1, updated_at_us = 1002 WHERE id = 'c2'",
                [],
            )
            .unwrap();
        }

        let first = db.list_conversations(None, 1, None, None, false).unwrap();
        assert_eq!(first[0].id, "c2");
        let second = db
            .list_conversations(
                None,
                1,
                Some(first[0].updated_at_us),
                Some(first[0].id.as_str()),
                false,
            )
            .unwrap();
        assert_eq!(second[0].id, "c1");
    }
}
