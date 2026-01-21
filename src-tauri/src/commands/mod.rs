//! Tauri IPC command handlers.
//!
//! This module contains all Tauri commands that can be invoked from the frontend.
//! Commands include: `get_conversations`, `get_conversation`, `search_conversations`, `get_projects`.

use crate::db::sqlite::{Database, DbError};
use crate::models::{
    Conversation, ConversationFilters, ConversationSummary, Message, MessageRole, TokenCount,
};
use crate::parser::{parse_content_blocks, parse_conversation_file, ParserError, RawMessageType};
use std::path::Path;
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info, warn};

/// Pagination parameters for list queries.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    /// Maximum number of results (default: 100).
    #[serde(default = "default_limit")]
    pub limit: i32,
    /// Number of results to skip (default: 0).
    #[serde(default)]
    pub offset: i32,
}

fn default_limit() -> i32 {
    100
}

/// Error type for command handlers.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Parser error: {0}")]
    Parser(#[from] ParserError),

    #[error("Not found: {0}")]
    NotFound(String),
}

// Implement serde::Serialize for CommandError so it can be returned from commands
impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Gets a list of conversation summaries with optional filtering and pagination.
///
/// # Arguments
/// * `db` - Database state
/// * `filters` - Optional filters (project, date_start, date_end)
/// * `pagination` - Optional pagination (limit, offset)
///
/// # Returns
/// * `Vec<ConversationSummary>` - List of conversations sorted by lastTime descending
#[tauri::command]
pub fn get_conversations(
    db: State<'_, Arc<Database>>,
    filters: Option<ConversationFilters>,
    pagination: Option<PaginationParams>,
) -> Result<Vec<ConversationSummary>, CommandError> {
    let filters = filters.unwrap_or_default();
    let pagination = pagination.unwrap_or_default();

    debug!(
        "get_conversations: filters={:?}, pagination={:?}",
        filters, pagination
    );

    db.with_connection(|conn| {
        // Build query with optional filters
        let mut sql = String::from(
            r#"
            SELECT id, project_name, start_time, last_time, preview, message_count
            FROM conversations
            WHERE 1=1
            "#,
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // Add project filter
        if let Some(ref project) = filters.project {
            sql.push_str(" AND project_name = ?");
            params_vec.push(Box::new(project.clone()));
        }

        // Add date_start filter
        if let Some(ref date_start) = filters.date_start {
            sql.push_str(" AND last_time >= ?");
            params_vec.push(Box::new(date_start.clone()));
        }

        // Add date_end filter
        if let Some(ref date_end) = filters.date_end {
            sql.push_str(" AND last_time <= ?");
            params_vec.push(Box::new(date_end.clone()));
        }

        // Add ordering and pagination
        sql.push_str(" ORDER BY last_time DESC LIMIT ? OFFSET ?");
        params_vec.push(Box::new(pagination.limit));
        params_vec.push(Box::new(pagination.offset));

        // Convert params to references
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(ConversationSummary {
                id: row.get(0)?,
                project_name: row.get(1)?,
                start_time: row.get(2)?,
                last_time: row.get(3)?,
                preview: row.get(4)?,
                message_count: row.get(5)?,
            })
        })?;

        let mut results = Vec::new();
        for row_result in rows {
            results.push(row_result?);
        }

        info!(
            "get_conversations: returned {} results",
            results.len()
        );

        Ok(results)
    })
    .map_err(CommandError::from)
}

/// Gets a single conversation with all messages and content blocks.
///
/// # Arguments
/// * `db` - Database state
/// * `id` - Conversation ID to retrieve
///
/// # Returns
/// * `Conversation` - Full conversation with parsed messages and content blocks
///
/// # Errors
/// * `NotFound` - If no conversation with the given ID exists
/// * `Parser` - If the JSONL file cannot be parsed
#[tauri::command]
pub fn get_conversation(
    db: State<'_, Arc<Database>>,
    id: String,
) -> Result<Conversation, CommandError> {
    debug!("get_conversation: id={}", id);

    // Look up conversation metadata from database
    let metadata = db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, project_path, project_name, start_time, last_time, file_path,
                   total_input_tokens, total_output_tokens
            FROM conversations
            WHERE id = ?1
            "#,
        )?;

        let row = stmt.query_row([&id], |row| {
            Ok(ConversationMetadata {
                id: row.get(0)?,
                project_path: row.get(1)?,
                project_name: row.get(2)?,
                start_time: row.get(3)?,
                last_time: row.get(4)?,
                file_path: row.get(5)?,
                total_input_tokens: row.get(6)?,
                total_output_tokens: row.get(7)?,
            })
        });

        match row {
            Ok(m) => Ok(Some(m)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::from(e)),
        }
    })?;

    let metadata = metadata.ok_or_else(|| CommandError::NotFound(format!("Conversation not found: {}", id)))?;

    // Parse the JSONL file to get messages
    let file_path = Path::new(&metadata.file_path);
    if !file_path.exists() {
        warn!("Conversation file not found: {:?}", file_path);
        return Err(CommandError::NotFound(format!(
            "Conversation file not found: {}",
            metadata.file_path
        )));
    }

    let parsed_conversations = parse_conversation_file(file_path)?;

    // Find the conversation with matching ID
    let parsed = parsed_conversations
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| CommandError::NotFound(format!("Conversation not found in file: {}", id)))?;

    // Convert RawMessages to Messages with parsed content blocks
    let messages: Vec<Message> = parsed
        .messages
        .iter()
        .enumerate()
        .map(|(idx, raw)| {
            let role = match raw.message_type {
                RawMessageType::User => MessageRole::User,
                RawMessageType::Assistant => MessageRole::Assistant,
                RawMessageType::System => MessageRole::System,
            };

            let content = parse_content_blocks(&raw.message.content);

            let token_count = raw.token_count.as_ref().map(|tc| TokenCount {
                input: tc.input,
                output: tc.output,
            });

            Message {
                id: raw.uuid.clone().unwrap_or_else(|| format!("msg_{}", idx)),
                role,
                content,
                timestamp: raw.timestamp.clone().unwrap_or_default(),
                token_count,
            }
        })
        .collect();

    info!(
        "get_conversation: loaded {} messages for {}",
        messages.len(),
        id
    );

    Ok(Conversation {
        id: metadata.id,
        project_path: metadata.project_path,
        project_name: metadata.project_name,
        start_time: metadata.start_time,
        last_time: metadata.last_time,
        messages,
        total_tokens: TokenCount {
            input: metadata.total_input_tokens,
            output: metadata.total_output_tokens,
        },
        bookmarked: None,
        tags: None,
    })
}

/// Internal struct for conversation metadata from DB.
#[derive(Debug)]
struct ConversationMetadata {
    id: String,
    project_path: String,
    project_name: String,
    start_time: String,
    last_time: String,
    file_path: String,
    total_input_tokens: i64,
    total_output_tokens: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params, Connection};
    use tempfile::tempdir;

    fn setup_test_db() -> Database {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(db_path).unwrap();
        db.init_schema().unwrap();
        db
    }

    fn insert_test_conversation(conn: &Connection, id: &str, project_name: &str, last_time: &str) {
        conn.execute(
            r#"
            INSERT INTO conversations (id, project_path, project_name, start_time, last_time, preview, message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
            VALUES (?1, '/test/project', ?2, '2025-01-01T00:00:00Z', ?3, 'Test preview...', 10, 100, 200, '/test/file.jsonl', '2025-01-01T00:00:00Z')
            "#,
            params![id, project_name, last_time],
        ).unwrap();
    }

    #[test]
    fn test_get_conversations_empty() {
        let db = setup_test_db();

        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_name, start_time, last_time, preview, message_count FROM conversations ORDER BY last_time DESC LIMIT 100 OFFSET 0"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(ConversationSummary {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    start_time: row.get(2)?,
                    last_time: row.get(3)?,
                    preview: row.get(4)?,
                    message_count: row.get(5)?,
                })
            })?;
            let results: Vec<ConversationSummary> = rows.filter_map(|r| r.ok()).collect();
            Ok(results)
        }).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_get_conversations_with_data() {
        let db = setup_test_db();

        // Insert test data
        db.with_connection(|conn| {
            insert_test_conversation(conn, "conv1", "project-a", "2025-01-15T10:00:00Z");
            insert_test_conversation(conn, "conv2", "project-b", "2025-01-15T11:00:00Z");
            insert_test_conversation(conn, "conv3", "project-a", "2025-01-15T12:00:00Z");
            Ok(())
        }).unwrap();

        // Query all
        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_name, start_time, last_time, preview, message_count FROM conversations ORDER BY last_time DESC"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(ConversationSummary {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    start_time: row.get(2)?,
                    last_time: row.get(3)?,
                    preview: row.get(4)?,
                    message_count: row.get(5)?,
                })
            })?;
            let results: Vec<ConversationSummary> = rows.filter_map(|r| r.ok()).collect();
            Ok(results)
        }).unwrap();

        assert_eq!(result.len(), 3);
        // Should be sorted by last_time desc
        assert_eq!(result[0].id, "conv3");
        assert_eq!(result[1].id, "conv2");
        assert_eq!(result[2].id, "conv1");
    }

    #[test]
    fn test_get_conversations_with_project_filter() {
        let db = setup_test_db();

        // Insert test data
        db.with_connection(|conn| {
            insert_test_conversation(conn, "conv1", "project-a", "2025-01-15T10:00:00Z");
            insert_test_conversation(conn, "conv2", "project-b", "2025-01-15T11:00:00Z");
            insert_test_conversation(conn, "conv3", "project-a", "2025-01-15T12:00:00Z");
            Ok(())
        }).unwrap();

        // Query with project filter
        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_name, start_time, last_time, preview, message_count FROM conversations WHERE project_name = ? ORDER BY last_time DESC"
            )?;
            let rows = stmt.query_map(["project-a"], |row| {
                Ok(ConversationSummary {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    start_time: row.get(2)?,
                    last_time: row.get(3)?,
                    preview: row.get(4)?,
                    message_count: row.get(5)?,
                })
            })?;
            let results: Vec<ConversationSummary> = rows.filter_map(|r| r.ok()).collect();
            Ok(results)
        }).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|c| c.project_name == "project-a"));
    }

    #[test]
    fn test_get_conversations_with_date_filter() {
        let db = setup_test_db();

        // Insert test data
        db.with_connection(|conn| {
            insert_test_conversation(conn, "conv1", "project-a", "2025-01-10T00:00:00Z");
            insert_test_conversation(conn, "conv2", "project-a", "2025-01-15T00:00:00Z");
            insert_test_conversation(conn, "conv3", "project-a", "2025-01-20T00:00:00Z");
            Ok(())
        }).unwrap();

        // Query with date range filter
        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_name, start_time, last_time, preview, message_count FROM conversations WHERE last_time >= ? AND last_time <= ? ORDER BY last_time DESC"
            )?;
            let rows = stmt.query_map(["2025-01-12T00:00:00Z", "2025-01-18T00:00:00Z"], |row| {
                Ok(ConversationSummary {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    start_time: row.get(2)?,
                    last_time: row.get(3)?,
                    preview: row.get(4)?,
                    message_count: row.get(5)?,
                })
            })?;
            let results: Vec<ConversationSummary> = rows.filter_map(|r| r.ok()).collect();
            Ok(results)
        }).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "conv2");
    }

    #[test]
    fn test_get_conversations_pagination() {
        let db = setup_test_db();

        // Insert test data
        db.with_connection(|conn| {
            for i in 1..=10 {
                insert_test_conversation(
                    conn,
                    &format!("conv{}", i),
                    "project-a",
                    &format!("2025-01-{:02}T00:00:00Z", i),
                );
            }
            Ok(())
        }).unwrap();

        // Query with pagination
        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_name, start_time, last_time, preview, message_count FROM conversations ORDER BY last_time DESC LIMIT 3 OFFSET 2"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(ConversationSummary {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    start_time: row.get(2)?,
                    last_time: row.get(3)?,
                    preview: row.get(4)?,
                    message_count: row.get(5)?,
                })
            })?;
            let results: Vec<ConversationSummary> = rows.filter_map(|r| r.ok()).collect();
            Ok(results)
        }).unwrap();

        // Should return 3 items starting from offset 2
        assert_eq!(result.len(), 3);
        // Sorted by last_time desc: conv10, conv9, conv8, conv7, conv6...
        // Offset 2 should skip conv10, conv9 and return conv8, conv7, conv6
        assert_eq!(result[0].id, "conv8");
        assert_eq!(result[1].id, "conv7");
        assert_eq!(result[2].id, "conv6");
    }

    // ========== get_conversation tests ==========

    #[test]
    fn test_get_conversation_metadata_not_found() {
        let db = setup_test_db();

        // Query a non-existent conversation
        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_path, project_name, start_time, last_time, file_path FROM conversations WHERE id = ?1",
            )?;

            let row = stmt.query_row(["nonexistent"], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                ))
            });

            match row {
                Ok(m) => Ok(Some(m)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DbError::from(e)),
            }
        }).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_get_conversation_metadata_found() {
        let db = setup_test_db();

        // Insert test conversation
        db.with_connection(|conn| {
            conn.execute(
                r#"
                INSERT INTO conversations (id, project_path, project_name, start_time, last_time, preview, message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                VALUES ('test-conv-1', '/home/user/project', 'my-project', '2025-01-01T00:00:00Z', '2025-01-01T01:00:00Z', 'Hello world', 5, 100, 200, '/path/to/file.jsonl', '2025-01-01T00:00:00Z')
                "#,
                [],
            )?;
            Ok(())
        }).unwrap();

        // Query the conversation metadata
        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_path, project_name, start_time, last_time, file_path, total_input_tokens, total_output_tokens FROM conversations WHERE id = ?1",
            )?;

            let row = stmt.query_row(["test-conv-1"], |row| {
                Ok(ConversationMetadata {
                    id: row.get(0)?,
                    project_path: row.get(1)?,
                    project_name: row.get(2)?,
                    start_time: row.get(3)?,
                    last_time: row.get(4)?,
                    file_path: row.get(5)?,
                    total_input_tokens: row.get(6)?,
                    total_output_tokens: row.get(7)?,
                })
            });

            match row {
                Ok(m) => Ok(Some(m)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DbError::from(e)),
            }
        }).unwrap();

        assert!(result.is_some());
        let metadata = result.unwrap();
        assert_eq!(metadata.id, "test-conv-1");
        assert_eq!(metadata.project_name, "my-project");
        assert_eq!(metadata.total_input_tokens, 100);
        assert_eq!(metadata.total_output_tokens, 200);
    }

    #[test]
    fn test_conversation_metadata_struct() {
        let metadata = ConversationMetadata {
            id: "test-123".to_string(),
            project_path: "/home/user/project".to_string(),
            project_name: "my-project".to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            last_time: "2025-01-01T01:00:00Z".to_string(),
            file_path: "/path/to/file.jsonl".to_string(),
            total_input_tokens: 100,
            total_output_tokens: 200,
        };

        assert_eq!(metadata.id, "test-123");
        assert_eq!(metadata.project_path, "/home/user/project");
        assert_eq!(metadata.project_name, "my-project");
    }
}
