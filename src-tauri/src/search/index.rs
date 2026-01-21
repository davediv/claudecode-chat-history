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
}
