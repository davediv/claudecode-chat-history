//! FTS5 search index population and management.
//!
//! This module handles building and updating the SQLite FTS5 full-text search
//! index for conversation content and metadata.

use crate::db::{Database, DbError, DbResult};
use crate::parser::{ParsedConversation, RawContent, RawMessageType};
use rusqlite::Connection;
use tracing::{debug, info, warn};

/// Builds or updates the search index from parsed conversations.
///
/// Indexes all message content and project names for fast full-text search.
/// This function is incremental - it only updates entries for the provided
/// conversations, not the entire index.
///
/// # Arguments
/// * `db` - Database connection
/// * `conversations` - Conversations to index
///
/// # Returns
/// * Number of conversations indexed
pub fn build_search_index(db: &Database, conversations: &[ParsedConversation]) -> DbResult<usize> {
    if conversations.is_empty() {
        debug!("No conversations to index");
        return Ok(0);
    }

    info!("Building search index for {} conversations", conversations.len());

    db.with_connection_mut(|conn| {
        let tx = conn.transaction()?;

        let mut indexed_count = 0;

        for conversation in conversations {
            // Extract all text content from messages
            let content = extract_searchable_content(conversation);

            // Get the rowid for this conversation from the conversations table
            let rowid: Option<i64> = tx
                .query_row(
                    "SELECT rowid FROM conversations WHERE id = ?1",
                    [&conversation.id],
                    |row| row.get(0),
                )
                .ok();

            match rowid {
                Some(rid) => {
                    // Update existing FTS entry (delete then insert)
                    tx.execute(
                        "DELETE FROM conversations_fts WHERE rowid = ?1",
                        [rid],
                    )?;

                    tx.execute(
                        "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (?1, ?2, ?3)",
                        rusqlite::params![rid, content, conversation.project_name],
                    )?;

                    indexed_count += 1;
                    debug!("Updated FTS index for conversation {}", conversation.id);
                }
                None => {
                    warn!(
                        "Conversation {} not found in database, skipping FTS indexing",
                        conversation.id
                    );
                }
            }
        }

        tx.commit()?;

        info!("Indexed {} conversations in search index", indexed_count);
        Ok(indexed_count)
    })
}

/// Builds the search index for all conversations in the database.
///
/// This performs a full rebuild of the FTS index from the conversations table.
/// Use this when the index needs to be completely rebuilt.
pub fn rebuild_search_index(db: &Database) -> DbResult<usize> {
    info!("Rebuilding full search index");

    db.with_connection_mut(|conn| {
        let tx = conn.transaction()?;

        // Clear existing FTS index
        tx.execute("DELETE FROM conversations_fts", [])?;

        // Get all conversations with their content
        // Note: We need to re-parse files to get full content, or store content summary
        // For now, we'll index what we have in the database (project_name + preview)
        // Collect all data first, then drop the statement before inserting
        let conversations_data: Vec<(i64, String, String)> = {
            let mut stmt = tx.prepare(
                "SELECT rowid, project_name, preview FROM conversations"
            )?;

            let mut rows = stmt.query([])?;
            let mut data = Vec::new();

            while let Some(row) = rows.next()? {
                data.push((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ));
            }
            data
        };

        let mut indexed_count = 0;

        for (rowid, project_name, preview) in conversations_data {
            tx.execute(
                "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (?1, ?2, ?3)",
                rusqlite::params![rowid, preview, project_name],
            )?;

            indexed_count += 1;
        }

        tx.commit()?;

        info!("Rebuilt search index with {} entries", indexed_count);
        Ok(indexed_count)
    })
}

/// Indexes a single conversation in the FTS index.
///
/// This is useful for incremental updates when a single conversation changes.
pub fn index_conversation(conn: &Connection, conversation: &ParsedConversation) -> DbResult<()> {
    let content = extract_searchable_content(conversation);
    index_conversation_content(conn, &conversation.id, &content, &conversation.project_name)
}

/// Indexes a conversation by ID with provided content.
///
/// This is a lower-level function useful when you already have the content
/// extracted (e.g., from the file watcher).
pub fn index_conversation_content(
    conn: &Connection,
    conversation_id: &str,
    content: &str,
    project_name: &str,
) -> DbResult<()> {
    // Get the rowid for this conversation
    let rowid: i64 = conn.query_row(
        "SELECT rowid FROM conversations WHERE id = ?1",
        [conversation_id],
        |row| row.get(0),
    ).map_err(|e| {
        warn!("Conversation {} not found: {}", conversation_id, e);
        DbError::Sqlite(e)
    })?;

    // Delete existing entry if any
    conn.execute("DELETE FROM conversations_fts WHERE rowid = ?1", [rowid])?;

    // Insert new entry
    conn.execute(
        "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (?1, ?2, ?3)",
        rusqlite::params![rowid, content, project_name],
    )?;

    debug!("Indexed conversation {} in FTS", conversation_id);
    Ok(())
}

/// Removes a conversation from the FTS index.
pub fn remove_from_index(conn: &Connection, conversation_id: &str) -> DbResult<()> {
    // Get the rowid for this conversation
    let rowid: Option<i64> = conn
        .query_row(
            "SELECT rowid FROM conversations WHERE id = ?1",
            [conversation_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(rid) = rowid {
        conn.execute("DELETE FROM conversations_fts WHERE rowid = ?1", [rid])?;
        debug!("Removed conversation {} from FTS index", conversation_id);
    }

    Ok(())
}

/// Extracts all searchable text content from a conversation.
///
/// Combines all message text content into a single searchable string.
/// Includes user messages, assistant responses, and relevant tool outputs.
fn extract_searchable_content(conversation: &ParsedConversation) -> String {
    let mut content_parts: Vec<String> = Vec::new();

    for message in &conversation.messages {
        // Include user and assistant messages (skip system for now)
        if message.message_type == RawMessageType::System {
            continue;
        }

        match &message.message.content {
            RawContent::Text(text) => {
                if !text.trim().is_empty() {
                    content_parts.push(text.clone());
                }
            }
            RawContent::Blocks(blocks) => {
                for block in blocks {
                    // Extract text from text blocks
                    if block.block_type == "text" {
                        if let Some(text) = &block.text {
                            if !text.trim().is_empty() {
                                content_parts.push(text.clone());
                            }
                        }
                    }
                    // Also index tool names for searchability
                    if block.block_type == "tool_use" {
                        if let Some(name) = &block.name {
                            content_parts.push(format!("[tool: {}]", name));
                        }
                    }
                }
            }
        }
    }

    // Join all content with spaces
    content_parts.join(" ")
}

/// Clears the entire FTS index.
pub fn clear_search_index(db: &Database) -> DbResult<()> {
    db.with_connection(|conn| {
        conn.execute("DELETE FROM conversations_fts", [])?;
        info!("Cleared search index");
        Ok(())
    })
}

/// Gets the count of entries in the FTS index.
pub fn get_index_count(conn: &Connection) -> DbResult<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM conversations_fts",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{RawInnerMessage, RawMessage, RawTokenCount};
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_conversation(id: &str, project_name: &str, content: &str) -> ParsedConversation {
        ParsedConversation {
            id: id.to_string(),
            project_path: "/test/project".to_string(),
            project_name: project_name.to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            last_time: "2025-01-01T01:00:00Z".to_string(),
            messages: vec![RawMessage {
                message_type: RawMessageType::User,
                message: RawInnerMessage {
                    content: RawContent::Text(content.to_string()),
                    role: Some("user".to_string()),
                },
                timestamp: Some("2025-01-01T00:00:00Z".to_string()),
                token_count: Some(RawTokenCount::default()),
                uuid: Some("test-uuid".to_string()),
                session_id: Some("test-session".to_string()),
            }],
            total_input_tokens: 100,
            total_output_tokens: 200,
            session_id: "test-session".to_string(),
            file_path: PathBuf::from("/test/session.jsonl"),
        }
    }

    #[test]
    fn test_extract_searchable_content() {
        let conversation = create_test_conversation(
            "conv1",
            "my-project",
            "How do I write a Rust function?",
        );

        let content = extract_searchable_content(&conversation);
        assert!(content.contains("Rust function"));
    }

    #[test]
    fn test_extract_content_with_blocks() {
        let mut conversation = create_test_conversation("conv1", "my-project", "");

        // Replace with block content
        conversation.messages[0].message.content = RawContent::Blocks(vec![
            crate::parser::RawContentBlock {
                block_type: "text".to_string(),
                text: Some("Hello world".to_string()),
                name: None,
                input: None,
                tool_use_id: None,
                content: None,
            },
            crate::parser::RawContentBlock {
                block_type: "tool_use".to_string(),
                text: None,
                name: Some("read_file".to_string()),
                input: None,
                tool_use_id: None,
                content: None,
            },
        ]);

        let content = extract_searchable_content(&conversation);
        assert!(content.contains("Hello world"));
        assert!(content.contains("[tool: read_file]"));
    }

    #[test]
    fn test_index_database_operations() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(db_path).unwrap();
        db.init_schema().unwrap();

        // Insert a conversation into the database first
        db.with_connection(|conn| {
            conn.execute(
                r#"INSERT INTO conversations
                   (id, project_path, project_name, start_time, last_time, preview,
                    message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
                rusqlite::params![
                    "conv1",
                    "/test/project",
                    "my-project",
                    "2025-01-01T00:00:00Z",
                    "2025-01-01T01:00:00Z",
                    "Test preview",
                    5,
                    100,
                    200,
                    "/test/session.jsonl",
                    "2025-01-01T00:00:00Z"
                ],
            )?;
            Ok(())
        }).unwrap();

        // Create a conversation to index
        let conversation = create_test_conversation(
            "conv1",
            "my-project",
            "How do I write a Rust function?",
        );

        // Index it
        let count = build_search_index(&db, &[conversation]).unwrap();
        assert_eq!(count, 1);

        // Verify it's in the index
        db.with_connection(|conn| {
            let fts_count = get_index_count(conn)?;
            assert_eq!(fts_count, 1);

            // Search for content
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'Rust'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            assert_eq!(results.len(), 1);

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_rebuild_search_index() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(db_path).unwrap();
        db.init_schema().unwrap();

        // Insert some conversations
        db.with_connection(|conn| {
            for i in 1..=5 {
                conn.execute(
                    r#"INSERT INTO conversations
                       (id, project_path, project_name, start_time, last_time, preview,
                        message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
                    rusqlite::params![
                        format!("conv{}", i),
                        "/test/project",
                        format!("project-{}", i),
                        "2025-01-01T00:00:00Z",
                        "2025-01-01T01:00:00Z",
                        format!("Preview for conversation {}", i),
                        5,
                        100,
                        200,
                        format!("/test/session{}.jsonl", i),
                        "2025-01-01T00:00:00Z"
                    ],
                )?;
            }
            Ok(())
        }).unwrap();

        // Rebuild index
        let count = rebuild_search_index(&db).unwrap();
        assert_eq!(count, 5);

        // Verify count
        db.with_connection(|conn| {
            let fts_count = get_index_count(conn)?;
            assert_eq!(fts_count, 5);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_clear_search_index() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(db_path).unwrap();
        db.init_schema().unwrap();

        // Insert and index a conversation
        db.with_connection(|conn| {
            conn.execute(
                r#"INSERT INTO conversations
                   (id, project_path, project_name, start_time, last_time, preview,
                    message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
                rusqlite::params![
                    "conv1", "/test/project", "my-project",
                    "2025-01-01T00:00:00Z", "2025-01-01T01:00:00Z",
                    "Test preview", 5, 100, 200,
                    "/test/session.jsonl", "2025-01-01T00:00:00Z"
                ],
            )?;
            conn.execute(
                "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (1, 'test content', 'my-project')",
                [],
            )?;
            Ok(())
        }).unwrap();

        // Verify there's data
        db.with_connection(|conn| {
            let count = get_index_count(conn)?;
            assert_eq!(count, 1);
            Ok(())
        }).unwrap();

        // Clear index
        clear_search_index(&db).unwrap();

        // Verify it's empty
        db.with_connection(|conn| {
            let count = get_index_count(conn)?;
            assert_eq!(count, 0);
            Ok(())
        }).unwrap();
    }

    // ========== FTS5 Query Tests ==========

    /// Helper to setup a database with conversations and FTS index
    fn setup_db_with_fts() -> (Database, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(db_path).unwrap();
        db.init_schema().unwrap();
        (db, temp_dir)
    }

    /// Helper to insert a conversation and its FTS entry
    fn insert_conversation_with_fts(
        conn: &Connection,
        id: &str,
        project_name: &str,
        content: &str,
        last_time: &str,
    ) {
        conn.execute(
            r#"INSERT INTO conversations
               (id, project_path, project_name, start_time, last_time, preview,
                message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
               VALUES (?1, '/test/project', ?2, '2025-01-01T00:00:00Z', ?3, ?4, 5, 100, 200, ?5, '2025-01-01T00:00:00Z')"#,
            rusqlite::params![
                id,
                project_name,
                last_time,
                content,
                format!("/test/{}.jsonl", id)
            ],
        ).unwrap();

        let rowid: i64 = conn.query_row(
            "SELECT rowid FROM conversations WHERE id = ?1",
            [id],
            |row| row.get(0),
        ).unwrap();

        conn.execute(
            "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (?1, ?2, ?3)",
            rusqlite::params![rowid, content, project_name],
        ).unwrap();
    }

    #[test]
    fn test_fts5_prefix_matching() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            insert_conversation_with_fts(
                conn,
                "conv1",
                "my-project",
                "How to write Rust code and rustic algorithms",
                "2025-01-01T00:00:00Z",
            );

            // Prefix search should match "Rust" and "rustic"
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'rust*'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            assert_eq!(results.len(), 1, "Prefix search should find the conversation");

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_fts5_phrase_matching() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            insert_conversation_with_fts(
                conn,
                "conv1",
                "my-project",
                "How to write a Rust function for error handling",
                "2025-01-01T00:00:00Z",
            );
            insert_conversation_with_fts(
                conn,
                "conv2",
                "my-project",
                "Rust is great. Function definitions are easy.",
                "2025-01-01T01:00:00Z",
            );

            // Phrase search should only match exact phrase
            let mut stmt = conn.prepare(
                r#"SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH '"Rust function"'"#
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            assert_eq!(results.len(), 1, "Phrase search should only match conv1");

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_fts5_case_insensitivity() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            insert_conversation_with_fts(
                conn,
                "conv1",
                "my-project",
                "RUST is a systems programming language",
                "2025-01-01T00:00:00Z",
            );

            // Search with lowercase should find uppercase content
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'rust'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            assert_eq!(results.len(), 1, "Case-insensitive search should work");

            // Search with uppercase should also work
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'RUST'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            assert_eq!(results.len(), 1, "Uppercase search should also work");

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_fts5_multiple_terms() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            insert_conversation_with_fts(
                conn,
                "conv1",
                "my-project",
                "Rust error handling with Result type",
                "2025-01-01T00:00:00Z",
            );
            insert_conversation_with_fts(
                conn,
                "conv2",
                "my-project",
                "Python error handling with try except",
                "2025-01-01T01:00:00Z",
            );
            insert_conversation_with_fts(
                conn,
                "conv3",
                "my-project",
                "Rust is fast but no error handling here",
                "2025-01-01T02:00:00Z",
            );

            // Search for both terms (AND logic by default in FTS5)
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'rust error'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            // FTS5 default is AND, so both conv1 and conv3 should match
            assert_eq!(results.len(), 2, "Multi-term search should find conversations with both terms");

            Ok(())
        }).unwrap();
    }

    // ========== Filter Combination Tests ==========

    #[test]
    fn test_search_with_project_filter() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            insert_conversation_with_fts(
                conn,
                "conv1",
                "rust-project",
                "How to write Rust code",
                "2025-01-01T00:00:00Z",
            );
            insert_conversation_with_fts(
                conn,
                "conv2",
                "python-project",
                "How to write Rust bindings for Python",
                "2025-01-01T01:00:00Z",
            );

            // Search for "Rust" filtered by project
            let mut stmt = conn.prepare(
                r#"SELECT c.id
                   FROM conversations_fts
                   INNER JOIN conversations c ON conversations_fts.rowid = c.rowid
                   WHERE conversations_fts MATCH 'rust' AND c.project_name = 'rust-project'"#
            ).unwrap();
            let results: Vec<String> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            assert_eq!(results.len(), 1);
            assert_eq!(results[0], "conv1");

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_search_with_date_range_filter() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            insert_conversation_with_fts(
                conn,
                "conv1",
                "my-project",
                "Rust programming basics",
                "2025-01-01T00:00:00Z",
            );
            insert_conversation_with_fts(
                conn,
                "conv2",
                "my-project",
                "Advanced Rust patterns",
                "2025-01-15T00:00:00Z",
            );
            insert_conversation_with_fts(
                conn,
                "conv3",
                "my-project",
                "Rust web frameworks",
                "2025-01-30T00:00:00Z",
            );

            // Search for "Rust" within date range
            let mut stmt = conn.prepare(
                r#"SELECT c.id
                   FROM conversations_fts
                   INNER JOIN conversations c ON conversations_fts.rowid = c.rowid
                   WHERE conversations_fts MATCH 'rust'
                   AND c.last_time >= '2025-01-10T00:00:00Z'
                   AND c.last_time <= '2025-01-20T00:00:00Z'"#
            ).unwrap();
            let results: Vec<String> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            assert_eq!(results.len(), 1);
            assert_eq!(results[0], "conv2");

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_search_with_combined_filters() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            insert_conversation_with_fts(
                conn,
                "conv1",
                "rust-project",
                "Rust error handling",
                "2025-01-01T00:00:00Z",
            );
            insert_conversation_with_fts(
                conn,
                "conv2",
                "rust-project",
                "Rust error handling advanced",
                "2025-01-15T00:00:00Z",
            );
            insert_conversation_with_fts(
                conn,
                "conv3",
                "python-project",
                "Rust error handling from Python",
                "2025-01-15T00:00:00Z",
            );
            insert_conversation_with_fts(
                conn,
                "conv4",
                "rust-project",
                "Rust memory management",
                "2025-01-15T00:00:00Z",
            );

            // Search for "error" with project filter AND date filter
            let mut stmt = conn.prepare(
                r#"SELECT c.id
                   FROM conversations_fts
                   INNER JOIN conversations c ON conversations_fts.rowid = c.rowid
                   WHERE conversations_fts MATCH 'error'
                   AND c.project_name = 'rust-project'
                   AND c.last_time >= '2025-01-10T00:00:00Z'"#
            ).unwrap();
            let results: Vec<String> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            assert_eq!(results.len(), 1);
            assert_eq!(results[0], "conv2");

            Ok(())
        }).unwrap();
    }

    // ========== Search Ranking Tests ==========

    #[test]
    fn test_bm25_ranking_order() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            // Conv1: "rust" appears once
            insert_conversation_with_fts(
                conn,
                "conv1",
                "my-project",
                "Rust is a programming language",
                "2025-01-01T00:00:00Z",
            );
            // Conv2: "rust" appears multiple times (should rank higher)
            insert_conversation_with_fts(
                conn,
                "conv2",
                "my-project",
                "Rust Rust Rust - Learn Rust programming in Rust",
                "2025-01-01T01:00:00Z",
            );
            // Conv3: "rust" appears twice
            insert_conversation_with_fts(
                conn,
                "conv3",
                "my-project",
                "Rust is great for Rust developers",
                "2025-01-01T02:00:00Z",
            );

            // Query with BM25 ranking
            let mut stmt = conn.prepare(
                r#"SELECT c.id, bm25(conversations_fts) as rank
                   FROM conversations_fts
                   INNER JOIN conversations c ON conversations_fts.rowid = c.rowid
                   WHERE conversations_fts MATCH 'rust'
                   ORDER BY rank"#
            ).unwrap();

            let results: Vec<(String, f64)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            assert_eq!(results.len(), 3);
            // BM25 returns negative values; lower (more negative) is better
            // The one with most "rust" occurrences should have the lowest (most negative) rank
            assert_eq!(results[0].0, "conv2", "Conv2 with most 'rust' occurrences should rank first");

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_bm25_idf_scoring() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            // Insert multiple conversations with common word "the"
            for i in 1..=10 {
                insert_conversation_with_fts(
                    conn,
                    &format!("conv{}", i),
                    "my-project",
                    "The quick brown fox jumps over the lazy dog",
                    &format!("2025-01-{:02}T00:00:00Z", i),
                );
            }

            // Insert one conversation with rare word "xylophone"
            insert_conversation_with_fts(
                conn,
                "conv_rare",
                "my-project",
                "The xylophone makes beautiful music",
                "2025-01-15T00:00:00Z",
            );

            // Search for the rare word "xylophone"
            let mut stmt = conn.prepare(
                r#"SELECT c.id, bm25(conversations_fts) as rank
                   FROM conversations_fts
                   INNER JOIN conversations c ON conversations_fts.rowid = c.rowid
                   WHERE conversations_fts MATCH 'xylophone'
                   ORDER BY rank"#
            ).unwrap();

            let results: Vec<(String, f64)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            // Only the rare document should match
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].0, "conv_rare");

            Ok(())
        }).unwrap();
    }

    // ========== Snippet Extraction Tests ==========
    //
    // Note: The FTS5 table uses `content=''` (external content mode) which means
    // snippet() returns NULL because the actual content isn't stored in FTS.
    // The real application queries preview from the conversations table and
    // generates snippets manually. These tests verify the snippet SQL syntax works
    // using a standalone FTS5 table that stores content directly.

    /// Helper to create FTS table with actual content for snippet testing
    /// Uses a standalone FTS5 table (not external content mode)
    fn setup_db_with_content_fts() -> (Database, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(db_path).unwrap();

        // Create a standalone FTS5 table that stores content internally
        // This is different from the production table which uses content=''
        db.with_connection(|conn| {
            conn.execute_batch(
                r#"
                CREATE VIRTUAL TABLE IF NOT EXISTS content_fts USING fts5(
                    content,
                    project_name
                );
                "#
            ).unwrap();
            Ok(())
        }).unwrap();

        (db, temp_dir)
    }

    fn insert_test_content_fts(conn: &Connection, project_name: &str, content: &str) {
        conn.execute(
            "INSERT INTO content_fts (content, project_name) VALUES (?1, ?2)",
            rusqlite::params![content, project_name],
        ).unwrap();
    }

    #[test]
    fn test_snippet_extraction_with_marks() {
        let (db, _temp_dir) = setup_db_with_content_fts();

        db.with_connection(|conn| {
            insert_test_content_fts(
                conn,
                "my-project",
                "This is a test about Rust programming language features",
            );

            // Get snippet with highlights
            let mut stmt = conn.prepare(
                r#"SELECT snippet(content_fts, 0, '<mark>', '</mark>', '...', 50) as snippet
                   FROM content_fts
                   WHERE content_fts MATCH 'rust'"#
            ).unwrap();

            let snippet: String = stmt.query_row([], |row| row.get(0)).unwrap();

            assert!(snippet.contains("<mark>"), "Snippet should contain <mark> tag");
            assert!(snippet.contains("</mark>"), "Snippet should contain </mark> tag");
            assert!(
                snippet.contains("<mark>Rust</mark>") || snippet.to_lowercase().contains("<mark>rust</mark>"),
                "Snippet should highlight the matched term: {}", snippet
            );

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_snippet_context_extraction() {
        let (db, _temp_dir) = setup_db_with_content_fts();

        db.with_connection(|conn| {
            // Insert long content where the match is in the middle
            let long_content = format!(
                "{} This is about Rust programming. {}",
                "prefix ".repeat(20),
                "suffix ".repeat(20)
            );
            insert_test_content_fts(
                conn,
                "my-project",
                &long_content,
            );

            // Get snippet with context
            let mut stmt = conn.prepare(
                r#"SELECT snippet(content_fts, 0, '<mark>', '</mark>', '...', 10) as snippet
                   FROM content_fts
                   WHERE content_fts MATCH 'rust'"#
            ).unwrap();

            let snippet: String = stmt.query_row([], |row| row.get(0)).unwrap();

            // Snippet should be truncated with ellipsis
            assert!(
                snippet.contains("...") || snippet.len() < long_content.len(),
                "Snippet should be truncated for long content"
            );
            assert!(
                snippet.contains("<mark>"),
                "Snippet should contain the highlight"
            );

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_snippet_multiple_matches() {
        let (db, _temp_dir) = setup_db_with_content_fts();

        db.with_connection(|conn| {
            insert_test_content_fts(
                conn,
                "my-project",
                "Rust is great. I love Rust. Rust forever!",
            );

            // Get snippet - may show multiple highlights depending on context window
            let mut stmt = conn.prepare(
                r#"SELECT snippet(content_fts, 0, '<mark>', '</mark>', '...', 50) as snippet
                   FROM content_fts
                   WHERE content_fts MATCH 'rust'"#
            ).unwrap();

            let snippet: String = stmt.query_row([], |row| row.get(0)).unwrap();

            // Should have at least one highlight
            assert!(snippet.contains("<mark>"), "Should have at least one highlight");

            // Count the number of <mark> tags
            let mark_count = snippet.matches("<mark>").count();
            assert!(mark_count >= 1, "Should have at least one <mark> tag");

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_snippet_in_real_application_search() {
        // Test that the real application search query pattern works with external content FTS
        // Note: snippet() returns NULL with content='' so the app should fall back to preview
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            insert_conversation_with_fts(
                conn,
                "conv1",
                "my-project",
                "How to write Rust code efficiently",
                "2025-01-01T00:00:00Z",
            );

            // The real app query pattern - snippet returns NULL, so we use COALESCE with preview
            let mut stmt = conn.prepare(
                r#"SELECT c.id,
                          COALESCE(snippet(conversations_fts, 0, '<mark>', '</mark>', '...', 50), c.preview) as snippet,
                          bm25(conversations_fts) as rank
                   FROM conversations_fts
                   INNER JOIN conversations c ON conversations_fts.rowid = c.rowid
                   WHERE conversations_fts MATCH 'rust'
                   ORDER BY rank"#
            ).unwrap();

            let results: Vec<(String, String, f64)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].0, "conv1");
            // Snippet falls back to preview since FTS uses external content
            assert!(results[0].1.contains("Rust"), "Fallback to preview should contain search term");

            Ok(())
        }).unwrap();
    }

    // ========== Performance Tests ==========

    #[test]
    fn test_search_performance_bulk_data() {
        let (db, _temp_dir) = setup_db_with_fts();

        // Insert 10,000 conversations for performance testing
        let num_conversations: i64 = 10_000;

        db.with_connection(|conn| {
            let tx = conn.unchecked_transaction().unwrap();

            for i in 0..num_conversations {
                let content = if i % 100 == 0 {
                    // Every 100th conversation contains the search term
                    format!("Conversation {} about Rust programming and systems", i)
                } else {
                    format!("Conversation {} about various topics and things", i)
                };

                tx.execute(
                    r#"INSERT INTO conversations
                       (id, project_path, project_name, start_time, last_time, preview,
                        message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                       VALUES (?1, '/test/project', ?2, '2025-01-01T00:00:00Z', ?3, ?4, 5, 100, 200, ?5, '2025-01-01T00:00:00Z')"#,
                    rusqlite::params![
                        format!("conv{}", i),
                        format!("project-{}", i % 10),
                        format!("2025-01-{:02}T{:02}:00:00Z", (i % 28) + 1, i % 24),
                        &content,
                        format!("/test/conv{}.jsonl", i)
                    ],
                ).unwrap();

                let rowid = tx.last_insert_rowid();

                tx.execute(
                    "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (?1, ?2, ?3)",
                    rusqlite::params![rowid, &content, format!("project-{}", i % 10)],
                ).unwrap();
            }

            tx.commit().unwrap();

            Ok(())
        }).unwrap();

        // Verify the data was inserted
        let count: i64 = db.with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM conversations_fts", [], |row| row.get(0))
                .map_err(|e| crate::db::DbError::from(e))
        }).unwrap();
        assert_eq!(count, num_conversations, "Should have inserted all conversations");

        // Warmup query to ensure SQLite has optimized indexes after bulk insert
        db.with_connection(|conn| {
            let _: i64 = conn.query_row(
                "SELECT COUNT(*) FROM conversations_fts WHERE conversations_fts MATCH 'Conversation'",
                [],
                |row| row.get(0)
            ).map_err(|e| crate::db::DbError::from(e))?;
            Ok(())
        }).unwrap();

        // Measure search performance
        // Note: Using COALESCE with preview since snippet() returns NULL with external content FTS
        let start = std::time::Instant::now();

        let search_results = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                r#"SELECT c.id,
                          COALESCE(snippet(conversations_fts, 0, '<mark>', '</mark>', '...', 50), c.preview) as snippet,
                          bm25(conversations_fts) as rank
                   FROM conversations_fts
                   INNER JOIN conversations c ON conversations_fts.rowid = c.rowid
                   WHERE conversations_fts MATCH 'rust'
                   ORDER BY rank
                   LIMIT 100"#
            ).unwrap();

            let results: Vec<(String, String, f64)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            Ok(results)
        }).unwrap();

        let elapsed = start.elapsed();

        // Assert performance requirement: < 200ms
        assert!(
            elapsed.as_millis() < 200,
            "Search should complete in under 200ms, took {}ms",
            elapsed.as_millis()
        );

        // Verify we found results (100 conversations should have "Rust" - every 100th)
        assert_eq!(
            search_results.len(),
            100,
            "Should find 100 results (limited by LIMIT clause)"
        );

        // Verify snippets contain the search term (from preview fallback)
        for (_, snippet, _) in &search_results {
            assert!(
                snippet.contains("Rust"),
                "All snippets should contain 'Rust'"
            );
        }
    }

    #[test]
    fn test_search_performance_complex_query() {
        let (db, _temp_dir) = setup_db_with_fts();

        // Insert 5,000 conversations
        let num_conversations = 5_000;

        db.with_connection(|conn| {
            let tx = conn.unchecked_transaction().unwrap();

            for i in 0..num_conversations {
                let content = format!(
                    "Conversation {} about {} and {} programming",
                    i,
                    if i % 3 == 0 { "Rust" } else { "Python" },
                    if i % 5 == 0 { "error handling" } else { "web development" }
                );

                tx.execute(
                    r#"INSERT INTO conversations
                       (id, project_path, project_name, start_time, last_time, preview,
                        message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                       VALUES (?1, '/test/project', ?2, '2025-01-01T00:00:00Z', ?3, ?4, 5, 100, 200, ?5, '2025-01-01T00:00:00Z')"#,
                    rusqlite::params![
                        format!("conv{}", i),
                        format!("project-{}", i % 5),
                        format!("2025-01-{:02}T{:02}:00:00Z", (i % 28) + 1, i % 24),
                        &content,
                        format!("/test/conv{}.jsonl", i)
                    ],
                ).unwrap();

                let rowid = tx.last_insert_rowid();

                tx.execute(
                    "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (?1, ?2, ?3)",
                    rusqlite::params![rowid, &content, format!("project-{}", i % 5)],
                ).unwrap();
            }

            tx.commit().unwrap();

            Ok(())
        }).unwrap();

        // Warmup query to ensure SQLite has optimized indexes after bulk insert
        db.with_connection(|conn| {
            let _: i64 = conn.query_row(
                "SELECT COUNT(*) FROM conversations_fts WHERE conversations_fts MATCH 'Conversation'",
                [],
                |row| row.get(0)
            ).map_err(|e| crate::db::DbError::from(e))?;
            Ok(())
        }).unwrap();

        // Complex search with filters
        // Note: Using COALESCE with preview since snippet() returns NULL with external content FTS
        let start = std::time::Instant::now();

        let search_results = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                r#"SELECT c.id,
                          COALESCE(snippet(conversations_fts, 0, '<mark>', '</mark>', '...', 50), c.preview) as snippet,
                          bm25(conversations_fts) as rank
                   FROM conversations_fts
                   INNER JOIN conversations c ON conversations_fts.rowid = c.rowid
                   WHERE conversations_fts MATCH 'rust error'
                   AND c.project_name = 'project-0'
                   AND c.last_time >= '2025-01-10T00:00:00Z'
                   ORDER BY rank
                   LIMIT 100"#
            ).unwrap();

            let results: Vec<(String, String, f64)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            Ok(results)
        }).unwrap();

        let elapsed = start.elapsed();

        // Assert performance requirement: < 200ms even with complex query
        assert!(
            elapsed.as_millis() < 200,
            "Complex search should complete in under 200ms, took {}ms",
            elapsed.as_millis()
        );

        // Verify results match all filters
        for (id, _, _) in &search_results {
            // Result should be from project-0 due to filter
            let project: String = db.with_connection(|conn| {
                conn.query_row(
                    "SELECT project_name FROM conversations WHERE id = ?1",
                    [id],
                    |row| row.get(0),
                ).map_err(|e| crate::db::DbError::from(e))
            }).unwrap();
            assert_eq!(project, "project-0");
        }
    }

    // ========== Index Building Edge Cases ==========

    #[test]
    fn test_index_empty_content() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            conn.execute(
                r#"INSERT INTO conversations
                   (id, project_path, project_name, start_time, last_time, preview,
                    message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                   VALUES ('conv1', '/test/project', 'my-project', '2025-01-01T00:00:00Z',
                           '2025-01-01T01:00:00Z', '', 0, 0, 0, '/test/session.jsonl', '2025-01-01T00:00:00Z')"#,
                [],
            ).unwrap();

            Ok(())
        }).unwrap();

        // Create conversation with empty content
        let conversation = ParsedConversation {
            id: "conv1".to_string(),
            project_path: "/test/project".to_string(),
            project_name: "my-project".to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            last_time: "2025-01-01T01:00:00Z".to_string(),
            messages: vec![], // Empty messages
            total_input_tokens: 0,
            total_output_tokens: 0,
            session_id: "test-session".to_string(),
            file_path: PathBuf::from("/test/session.jsonl"),
        };

        // Should handle empty content gracefully
        let count = build_search_index(&db, &[conversation]).unwrap();
        assert_eq!(count, 1);

        // Verify it's in the index (with empty content)
        db.with_connection(|conn| {
            let fts_count = get_index_count(conn)?;
            assert_eq!(fts_count, 1);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_index_special_characters() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            conn.execute(
                r#"INSERT INTO conversations
                   (id, project_path, project_name, start_time, last_time, preview,
                    message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                   VALUES ('conv1', '/test/project', 'my-project', '2025-01-01T00:00:00Z',
                           '2025-01-01T01:00:00Z', 'test', 1, 100, 200, '/test/session.jsonl', '2025-01-01T00:00:00Z')"#,
                [],
            ).unwrap();

            Ok(())
        }).unwrap();

        // Content with special characters
        let conversation = create_test_conversation(
            "conv1",
            "my-project",
            r#"Code: fn main() { println!("Hello, world!"); } -- C++ style // comment"#,
        );

        // Should handle special characters
        let count = build_search_index(&db, &[conversation]).unwrap();
        assert_eq!(count, 1);

        // Search for special content
        db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'println'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            assert_eq!(results.len(), 1);

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_index_update_existing() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            conn.execute(
                r#"INSERT INTO conversations
                   (id, project_path, project_name, start_time, last_time, preview,
                    message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                   VALUES ('conv1', '/test/project', 'my-project', '2025-01-01T00:00:00Z',
                           '2025-01-01T01:00:00Z', 'original', 1, 100, 200, '/test/session.jsonl', '2025-01-01T00:00:00Z')"#,
                [],
            ).unwrap();

            Ok(())
        }).unwrap();

        // Index original content
        let conversation1 = create_test_conversation("conv1", "my-project", "Original content about Python");
        build_search_index(&db, &[conversation1]).unwrap();

        // Update with new content
        let conversation2 = create_test_conversation("conv1", "my-project", "Updated content about Rust");
        build_search_index(&db, &[conversation2]).unwrap();

        // Verify old content is not searchable
        db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'Python'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            assert_eq!(results.len(), 0, "Old content should not be searchable");

            // Verify new content is searchable
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'Rust'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            assert_eq!(results.len(), 1, "New content should be searchable");

            // Verify only one entry in FTS
            let count = get_index_count(conn)?;
            assert_eq!(count, 1, "Should have exactly one FTS entry after update");

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_remove_from_index() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            conn.execute(
                r#"INSERT INTO conversations
                   (id, project_path, project_name, start_time, last_time, preview,
                    message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                   VALUES ('conv1', '/test/project', 'my-project', '2025-01-01T00:00:00Z',
                           '2025-01-01T01:00:00Z', 'test', 1, 100, 200, '/test/session.jsonl', '2025-01-01T00:00:00Z')"#,
                [],
            ).unwrap();

            Ok(())
        }).unwrap();

        // Index a conversation
        let conversation = create_test_conversation("conv1", "my-project", "Content about Rust");
        build_search_index(&db, &[conversation]).unwrap();

        // Verify it's indexed
        db.with_connection(|conn| {
            let count = get_index_count(conn)?;
            assert_eq!(count, 1);
            Ok(())
        }).unwrap();

        // Remove from index
        db.with_connection(|conn| {
            remove_from_index(conn, "conv1")?;
            Ok(())
        }).unwrap();

        // Verify it's removed
        db.with_connection(|conn| {
            let count = get_index_count(conn)?;
            assert_eq!(count, 0);

            // Search should return nothing
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'Rust'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            assert_eq!(results.len(), 0);

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_index_conversation_content_directly() {
        let (db, _temp_dir) = setup_db_with_fts();

        db.with_connection(|conn| {
            conn.execute(
                r#"INSERT INTO conversations
                   (id, project_path, project_name, start_time, last_time, preview,
                    message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                   VALUES ('conv1', '/test/project', 'my-project', '2025-01-01T00:00:00Z',
                           '2025-01-01T01:00:00Z', 'test', 1, 100, 200, '/test/session.jsonl', '2025-01-01T00:00:00Z')"#,
                [],
            ).unwrap();

            // Use the lower-level function directly
            index_conversation_content(
                conn,
                "conv1",
                "Direct content about JavaScript frameworks",
                "my-project",
            ).unwrap();

            // Verify it's searchable
            let mut stmt = conn.prepare(
                "SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'JavaScript'"
            ).unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            assert_eq!(results.len(), 1);

            Ok(())
        }).unwrap();
    }
}
