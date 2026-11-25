use rusqlite::{params, Connection};
use std::sync::Mutex;

use crate::error::CoreError;
use crate::models::{ConversationSummary, LocalMessage};

pub struct DbHandle {
    conn: Mutex<Connection>,
}

impl DbHandle {
    pub fn open(path: &str, key: &[u8]) -> Result<Self, CoreError> {
        if key.len() != 32 {
            return Err(CoreError::DatabaseError {
                details: "Database key must be 32 bytes".to_string(),
                retryable: false,
            });
        }

        let mut conn = Connection::open(path).map_err(|e| CoreError::DatabaseError {
            details: e.to_string(),
            retryable: false,
        })?;

        #[cfg(feature = "sqlcipher")]
        {
            let key_hex = key
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            conn.execute_batch(&format!("PRAGMA key = \"x'{key_hex}'\";"))
                .map_err(|e| CoreError::DatabaseError {
                    details: e.to_string(),
                    retryable: false,
                })?;
        }
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| CoreError::DatabaseError {
            details: e.to_string(),
            retryable: false,
        })?;

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
        let conn = self.conn.lock().unwrap();
        let now_ms = chrono::Utc::now().timestamp_millis();
        conn.execute(
            r#"
            INSERT INTO local_conversations (id, title, active_language, study_mode_default, created_at_ms, updated_at_ms)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6);
            "#,
            params![id, title, active_language, study_mode_default as i32, now_ms, now_ms],
        )
        .map_err(|e| CoreError::DatabaseError {
            details: e.to_string(),
            retryable: false,
        })?;
        Ok(())
    }

    pub fn save_draft(&self, conversation_id: &str, text: &str) -> Result<(), CoreError> {
        let conn = self.conn.lock().unwrap();
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
        .map_err(|e| CoreError::DatabaseError {
            details: e.to_string(),
            retryable: false,
        })?;
        Ok(())
    }

    pub fn get_draft(&self, conversation_id: &str) -> Result<Option<String>, CoreError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT draft_text FROM local_drafts WHERE conversation_id = ?1")
            .map_err(|e| CoreError::DatabaseError {
                details: e.to_string(),
                retryable: false,
            })?;
        let mut rows =
            stmt.query(params![conversation_id])
                .map_err(|e| CoreError::DatabaseError {
                    details: e.to_string(),
                    retryable: false,
                })?;
        if let Some(row) = rows.next().map_err(|e| CoreError::DatabaseError {
            details: e.to_string(),
            retryable: false,
        })? {
            let text: String = row.get(0).map_err(|e| CoreError::DatabaseError {
                details: e.to_string(),
                retryable: false,
            })?;
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    pub fn insert_message(&self, msg: &LocalMessage) -> Result<(), CoreError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO local_messages (id, conversation_id, role, content_md, status, citations_json, created_at_ms)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                content_md = excluded.content_md,
                status = excluded.status,
                citations_json = excluded.citations_json;
            "#,
            params![
                msg.id,
                msg.conversation_id,
                msg.role,
                msg.content_md,
                msg.status,
                msg.citations_json,
                msg.created_at_ms
            ],
        )
        .map_err(|e| CoreError::DatabaseError {
            details: e.to_string(),
            retryable: false,
        })?;
        Ok(())
    }

    pub fn list_conversations(
        &self,
        query: Option<&str>,
    ) -> Result<Vec<ConversationSummary>, CoreError> {
        let conn = self.conn.lock().unwrap();
        let sql = match query {
            Some(_) => "SELECT id, title, active_language, study_mode_default, updated_at_ms FROM local_conversations WHERE title LIKE ?1 ORDER BY updated_at_ms DESC",
            None => "SELECT id, title, active_language, study_mode_default, updated_at_ms FROM local_conversations ORDER BY updated_at_ms DESC",
        };

        let mut stmt = conn.prepare(sql).map_err(|e| CoreError::DatabaseError {
            details: e.to_string(),
            retryable: false,
        })?;

        let mut list = Vec::new();
        if let Some(q) = query {
            let pattern = format!("%{}%", q);
            let rows = stmt
                .query_map(params![pattern], |row| {
                    Ok(ConversationSummary {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        active_language: row.get(2)?,
                        study_mode_default: row.get::<_, i32>(3)? != 0,
                        updated_at_ms: row.get(4)?,
                    })
                })
                .map_err(|e| CoreError::DatabaseError {
                    details: e.to_string(),
                    retryable: false,
                })?;
            for r in rows {
                list.push(r.map_err(|e| CoreError::DatabaseError {
                    details: e.to_string(),
                    retryable: false,
                })?);
            }
        } else {
            let rows = stmt
                .query_map([], |row| {
                    Ok(ConversationSummary {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        active_language: row.get(2)?,
                        study_mode_default: row.get::<_, i32>(3)? != 0,
                        updated_at_ms: row.get(4)?,
                    })
                })
                .map_err(|e| CoreError::DatabaseError {
                    details: e.to_string(),
                    retryable: false,
                })?;
            for r in rows {
                list.push(r.map_err(|e| CoreError::DatabaseError {
                    details: e.to_string(),
                    retryable: false,
                })?);
            }
        }

        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
        assert!(DbHandle::open(db_path.to_str().unwrap(), &wrong_key).is_err());

        let missing_key_path = temp_dir.path().join("missing-key.db");
        assert!(DbHandle::open(missing_key_path.to_str().unwrap(), &[]).is_err());
        assert!(!missing_key_path.exists());
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
}
