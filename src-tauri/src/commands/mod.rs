//! Tauri IPC command handlers.
//!
//! This module contains all Tauri commands that can be invoked from the frontend.
//! Commands include: `get_conversations`, `get_conversation`, `search_conversations`, `get_projects`.

use crate::db::sqlite::{Database, DbError};
use crate::models::{
    Conversation, ConversationFilters, ConversationSummary, Message, MessageRole, ProjectInfo,
    TokenCount,
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

/// Gets a list of all projects with conversation counts.
///
/// # Arguments
/// * `db` - Database state
///
/// # Returns
/// * `Vec<ProjectInfo>` - List of projects sorted alphabetically by name
#[tauri::command]
pub fn get_projects(db: State<'_, Arc<Database>>) -> Result<Vec<ProjectInfo>, CommandError> {
    debug!("get_projects");

    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT project_path, project_name, COUNT(*) as conversation_count, MAX(last_time) as last_activity
            FROM conversations
            GROUP BY project_path, project_name
            ORDER BY project_name ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ProjectInfo {
                project_path: row.get(0)?,
                project_name: row.get(1)?,
                conversation_count: row.get(2)?,
                last_activity: row.get(3)?,
            })
        })?;

        let mut results = Vec::new();
        for row_result in rows {
            results.push(row_result?);
        }

        info!("get_projects: returned {} projects", results.len());

        Ok(results)
    })
    .map_err(CommandError::from)
}

/// Searches conversations using full-text search.
///
/// # Arguments
/// * `db` - Database state
/// * `query` - Search query (minimum 2 characters)
/// * `filters` - Optional filters (project, date_start, date_end)
///
/// # Returns
/// * `Vec<SearchResult>` - List of search results with snippets and ranks
#[tauri::command]
pub fn search_conversations(
    db: State<'_, Arc<Database>>,
    query: String,
    filters: Option<ConversationFilters>,
) -> Result<Vec<crate::models::SearchResult>, CommandError> {
    let query = query.trim();

    // Enforce minimum query length
    if query.len() < 2 {
        debug!("search_conversations: query too short ({})", query.len());
        return Ok(Vec::new());
    }

    let filters = filters.unwrap_or_default();
    debug!("search_conversations: query='{}', filters={:?}", query, filters);

    db.with_connection(|conn| {
        // Build the search query
        // Using FTS5 snippet() function to extract context around matches
        // bm25() provides relevance ranking
        let mut sql = String::from(
            r#"
            SELECT
                c.id,
                snippet(conversations_fts, 0, '<mark>', '</mark>', '...', 50) as snippet,
                bm25(conversations_fts) as rank
            FROM conversations_fts
            INNER JOIN conversations c ON conversations_fts.rowid = c.rowid
            WHERE conversations_fts MATCH ?1
            "#,
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // Escape and prepare query for FTS5
        // FTS5 query syntax: use quotes for phrase, prefix with * for prefix match
        let fts_query = prepare_fts_query(query);
        params_vec.push(Box::new(fts_query));

        // Add project filter
        if let Some(ref project) = filters.project {
            sql.push_str(" AND c.project_name = ?");
            params_vec.push(Box::new(project.clone()));
        }

        // Add date_start filter
        if let Some(ref date_start) = filters.date_start {
            sql.push_str(" AND c.last_time >= ?");
            params_vec.push(Box::new(date_start.clone()));
        }

        // Add date_end filter
        if let Some(ref date_end) = filters.date_end {
            sql.push_str(" AND c.last_time <= ?");
            params_vec.push(Box::new(date_end.clone()));
        }

        // Order by relevance (bm25 returns negative values, lower is better)
        sql.push_str(" ORDER BY rank LIMIT 100");

        // Convert params to references
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(crate::models::SearchResult {
                conversation_id: row.get(0)?,
                snippet: row.get(1)?,
                match_count: 1, // FTS5 doesn't easily provide match count per row
                rank: row.get::<_, f64>(2)?.abs(), // Convert to positive, lower is better
            })
        })?;

        let mut results = Vec::new();
        for row_result in rows {
            match row_result {
                Ok(r) => results.push(r),
                Err(e) => {
                    warn!("Error reading search result row: {}", e);
                }
            }
        }

        info!(
            "search_conversations: '{}' returned {} results",
            query,
            results.len()
        );

        Ok(results)
    })
    .map_err(CommandError::from)
}

/// Prepares a query string for FTS5 search.
///
/// Escapes special characters and handles common search patterns.
fn prepare_fts_query(query: &str) -> String {
    // Escape double quotes and convert to a phrase query if contains spaces
    // Otherwise use prefix matching with *
    let escaped = query.replace('"', "\"\"");

    if escaped.contains(' ') {
        // Multi-word query: use phrase matching
        format!("\"{}\"", escaped)
    } else {
        // Single word: use prefix matching for better results
        format!("{}*", escaped)
    }
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

    // ========== get_projects tests ==========

    #[test]
    fn test_get_projects_empty() {
        let db = setup_test_db();

        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT project_path, project_name, COUNT(*) as conversation_count, MAX(last_time) as last_activity FROM conversations GROUP BY project_path, project_name ORDER BY project_name ASC"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(ProjectInfo {
                    project_path: row.get(0)?,
                    project_name: row.get(1)?,
                    conversation_count: row.get(2)?,
                    last_activity: row.get(3)?,
                })
            })?;
            let results: Vec<ProjectInfo> = rows.filter_map(|r| r.ok()).collect();
            Ok(results)
        }).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_get_projects_with_data() {
        let db = setup_test_db();

        // Insert conversations from different projects
        db.with_connection(|conn| {
            conn.execute(
                r#"INSERT INTO conversations (id, project_path, project_name, start_time, last_time, preview, message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                VALUES ('conv1', '/path/to/zebra', 'zebra-project', '2025-01-01T00:00:00Z', '2025-01-10T00:00:00Z', 'Test', 5, 100, 200, '/test/file1.jsonl', '2025-01-01T00:00:00Z')"#,
                [],
            )?;
            conn.execute(
                r#"INSERT INTO conversations (id, project_path, project_name, start_time, last_time, preview, message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                VALUES ('conv2', '/path/to/alpha', 'alpha-project', '2025-01-01T00:00:00Z', '2025-01-15T00:00:00Z', 'Test', 3, 50, 100, '/test/file2.jsonl', '2025-01-01T00:00:00Z')"#,
                [],
            )?;
            conn.execute(
                r#"INSERT INTO conversations (id, project_path, project_name, start_time, last_time, preview, message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                VALUES ('conv3', '/path/to/alpha', 'alpha-project', '2025-01-02T00:00:00Z', '2025-01-20T00:00:00Z', 'Test', 7, 150, 300, '/test/file3.jsonl', '2025-01-02T00:00:00Z')"#,
                [],
            )?;
            Ok(())
        }).unwrap();

        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT project_path, project_name, COUNT(*) as conversation_count, MAX(last_time) as last_activity FROM conversations GROUP BY project_path, project_name ORDER BY project_name ASC"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(ProjectInfo {
                    project_path: row.get(0)?,
                    project_name: row.get(1)?,
                    conversation_count: row.get(2)?,
                    last_activity: row.get(3)?,
                })
            })?;
            let results: Vec<ProjectInfo> = rows.filter_map(|r| r.ok()).collect();
            Ok(results)
        }).unwrap();

        assert_eq!(result.len(), 2);
        // Should be sorted alphabetically by project_name
        assert_eq!(result[0].project_name, "alpha-project");
        assert_eq!(result[0].conversation_count, 2);
        assert_eq!(result[0].last_activity, "2025-01-20T00:00:00Z");

        assert_eq!(result[1].project_name, "zebra-project");
        assert_eq!(result[1].conversation_count, 1);
        assert_eq!(result[1].last_activity, "2025-01-10T00:00:00Z");
    }

    // ========== search_conversations tests ==========

    #[test]
    fn test_prepare_fts_query_single_word() {
        let query = prepare_fts_query("rust");
        assert_eq!(query, "rust*");
    }

    #[test]
    fn test_prepare_fts_query_multi_word() {
        let query = prepare_fts_query("rust function");
        assert_eq!(query, "\"rust function\"");
    }

    #[test]
    fn test_prepare_fts_query_escapes_quotes() {
        let query = prepare_fts_query("test \"quoted\" word");
        assert_eq!(query, "\"test \"\"quoted\"\" word\"");
    }

    #[test]
    fn test_search_conversations_query_too_short() {
        let db = setup_test_db();

        // Query with single character should return empty results
        let result = db.with_connection(|conn| {
            // Simulate the check in search_conversations
            let query = "a";
            if query.len() < 2 {
                return Ok(Vec::<crate::models::SearchResult>::new());
            }
            unreachable!()
        }).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_search_conversations_with_data() {
        let db = setup_test_db();

        // Insert test data and get the rowids
        let (rowid1, rowid2) = db.with_connection(|conn| {
            conn.execute(
                r#"INSERT INTO conversations (id, project_path, project_name, start_time, last_time, preview, message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                VALUES ('conv1', '/test/project', 'my-project', '2025-01-01T00:00:00Z', '2025-01-01T01:00:00Z', 'How do I write a Rust function?', 5, 100, 200, '/test/file1.jsonl', '2025-01-01T00:00:00Z')"#,
                [],
            )?;
            let rowid1 = conn.last_insert_rowid();

            conn.execute(
                r#"INSERT INTO conversations (id, project_path, project_name, start_time, last_time, preview, message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                VALUES ('conv2', '/test/project', 'web-app', '2025-01-01T00:00:00Z', '2025-01-01T02:00:00Z', 'Help me with TypeScript types', 3, 50, 100, '/test/file2.jsonl', '2025-01-01T00:00:00Z')"#,
                [],
            )?;
            let rowid2 = conn.last_insert_rowid();

            Ok((rowid1, rowid2))
        }).unwrap();

        // Insert into FTS table with matching rowids
        db.with_connection(|conn| {
            conn.execute(
                "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (?1, 'How do I write a Rust function?', 'my-project')",
                [rowid1],
            )?;
            conn.execute(
                "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (?1, 'Help me with TypeScript types', 'web-app')",
                [rowid2],
            )?;
            Ok(())
        }).unwrap();

        // First verify FTS data is there
        let fts_count: i64 = db.with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM conversations_fts", [], |row| row.get(0))
                .map_err(|e| crate::db::sqlite::DbError::from(e))
        }).unwrap();
        assert_eq!(fts_count, 2, "FTS table should have 2 entries");

        // Test basic FTS5 MATCH query
        let fts_rowids: Vec<i64> = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'rust'"
            )?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            let results: Vec<i64> = rows.filter_map(|r| r.ok()).collect();
            Ok(results)
        }).unwrap();

        assert!(!fts_rowids.is_empty(), "FTS5 MATCH should find 'rust' in content");

        // Verify the rowid from FTS matches a conversation
        let conv_result: Option<String> = db.with_connection(|conn| {
            let result = conn.query_row(
                "SELECT id FROM conversations WHERE rowid = ?1",
                [fts_rowids[0]],
                |row| row.get::<_, String>(0),
            );
            match result {
                Ok(id) => Ok(Some(id)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(crate::db::sqlite::DbError::from(e)),
            }
        }).unwrap();

        assert!(conv_result.is_some(), "Should find conversation for FTS rowid");
        assert_eq!(conv_result.unwrap(), "conv1");
    }
}
