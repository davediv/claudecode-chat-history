//! SQLite database initialization and connection management.
//!
//! This module provides database connectivity for storing conversation
//! metadata and full-text search indexes.

use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Database-related errors.
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Failed to get app data directory")]
    AppDataNotFound,

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Database is locked: {0}")]
    Locked(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for database operations.
pub type DbResult<T> = Result<T, DbError>;

/// Database connection manager.
///
/// Provides a single connection with proper lifecycle management.
/// Uses a Mutex for thread-safe access from Tauri commands.
pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    /// Opens or creates the database at the specified path.
    ///
    /// Creates the parent directory if it doesn't exist.
    pub fn open(path: PathBuf) -> DbResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        debug!("Opening database at: {:?}", path);

        // Open with flags that handle busy/locked scenarios
        let conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        // Configure connection for better concurrency handling
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        // Enable WAL mode for better concurrent read/write performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        info!("Database opened successfully at: {:?}", path);

        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    /// Opens or creates the database in the default app data directory.
    ///
    /// The database file is created at `{app_data}/conversations.db`.
    pub fn open_default() -> DbResult<Self> {
        let app_data_dir = get_app_data_dir()?;
        let db_path = app_data_dir.join("conversations.db");
        Self::open(db_path)
    }

    /// Returns the database file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Executes a function with the database connection.
    ///
    /// This provides thread-safe access to the connection.
    pub fn with_connection<F, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&Connection) -> DbResult<T>,
    {
        let conn = self.conn.lock().map_err(|e| {
            warn!("Database lock poisoned: {}", e);
            DbError::Locked(e.to_string())
        })?;
        f(&conn)
    }

    /// Executes a function with a mutable database connection.
    ///
    /// This provides thread-safe access for operations that need mutable access.
    pub fn with_connection_mut<F, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut Connection) -> DbResult<T>,
    {
        let mut conn = self.conn.lock().map_err(|e| {
            warn!("Database lock poisoned: {}", e);
            DbError::Locked(e.to_string())
        })?;
        f(&mut conn)
    }

    /// Initializes the database schema.
    ///
    /// Creates tables if they don't exist. Safe to call multiple times.
    pub fn init_schema(&self) -> DbResult<()> {
        self.with_connection(|conn| {
            init_db(conn)?;
            Ok(())
        })
    }
}

/// Gets the application data directory.
///
/// On macOS: `~/Library/Application Support/com.claudecode.history-viewer`
/// On Windows: `%APPDATA%\com.claudecode.history-viewer`
/// On Linux: `~/.local/share/com.claudecode.history-viewer`
fn get_app_data_dir() -> DbResult<PathBuf> {
    let base_dir = dirs::data_dir().ok_or(DbError::AppDataNotFound)?;
    Ok(base_dir.join("com.claudecode.history-viewer"))
}

/// Initializes the database schema.
///
/// Creates the conversations table and FTS5 virtual table for full-text search.
/// This function is idempotent - safe to call multiple times.
pub fn init_db(conn: &Connection) -> DbResult<()> {
    debug!("Initializing database schema");

    // Create conversations metadata table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY NOT NULL,
            project_path TEXT NOT NULL,
            project_name TEXT NOT NULL,
            start_time TEXT NOT NULL,
            last_time TEXT NOT NULL,
            preview TEXT NOT NULL DEFAULT '',
            message_count INTEGER NOT NULL DEFAULT 0,
            total_input_tokens INTEGER NOT NULL DEFAULT 0,
            total_output_tokens INTEGER NOT NULL DEFAULT 0,
            file_path TEXT NOT NULL,
            file_modified_at TEXT NOT NULL
        );

        -- Indexes for common queries
        CREATE INDEX IF NOT EXISTS idx_conversations_project_name
            ON conversations(project_name);
        CREATE INDEX IF NOT EXISTS idx_conversations_start_time
            ON conversations(start_time);
        CREATE INDEX IF NOT EXISTS idx_conversations_last_time
            ON conversations(last_time);
        CREATE INDEX IF NOT EXISTS idx_conversations_file_path
            ON conversations(file_path);
        "#,
    )?;

    // Create file metadata table for incremental parsing
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS file_metadata (
            file_path TEXT PRIMARY KEY NOT NULL,
            modified_at TEXT NOT NULL,
            parsed_at TEXT NOT NULL
        );
        "#,
    )?;

    // Create FTS5 virtual table for full-text search
    // Uses content="" for external content mode - we manage content ourselves
    // This indexes conversation content and project names for fast searching
    conn.execute_batch(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS conversations_fts USING fts5(
            content,
            project_name,
            content='',
            contentless_delete=1
        );
        "#,
    )?;

    // Create bookmarks table for user-marked conversations
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS bookmarks (
            conversation_id TEXT PRIMARY KEY NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );

        -- Index for efficient bookmark lookups
        CREATE INDEX IF NOT EXISTS idx_bookmarks_conversation_id
            ON bookmarks(conversation_id);
        "#,
    )?;

    // Create conversation_tags table for user-defined tags
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS conversation_tags (
            conversation_id TEXT NOT NULL,
            tag TEXT NOT NULL,
            created_at TEXT NOT NULL,
            PRIMARY KEY (conversation_id, tag),
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );

        -- Index for efficient tag lookups
        CREATE INDEX IF NOT EXISTS idx_conversation_tags_conversation_id
            ON conversation_tags(conversation_id);
        CREATE INDEX IF NOT EXISTS idx_conversation_tags_tag
            ON conversation_tags(tag);
        "#,
    )?;

    info!("Database schema initialized successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::open(db_path.clone()).unwrap();
        assert!(db_path.exists());
        assert_eq!(db.path(), &db_path);
    }

    #[test]
    fn test_schema_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::open(db_path).unwrap();
        db.init_schema().unwrap();

        // Verify tables exist
        db.with_connection(|conn| {
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='conversations'")
                .unwrap();
            let exists: bool = stmt.exists([]).unwrap();
            assert!(exists, "conversations table should exist");

            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='file_metadata'")
                .unwrap();
            let exists: bool = stmt.exists([]).unwrap();
            assert!(exists, "file_metadata table should exist");

            // Verify FTS5 virtual table exists
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='conversations_fts'")
                .unwrap();
            let exists: bool = stmt.exists([]).unwrap();
            assert!(exists, "conversations_fts FTS5 table should exist");

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_fts5_insert_and_search() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::open(db_path).unwrap();
        db.init_schema().unwrap();

        db.with_connection(|conn| {
            // Insert test data into FTS5 table
            conn.execute(
                "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (1, 'How do I write a Rust function?', 'my-rust-project')",
                [],
            )
            .unwrap();

            conn.execute(
                "INSERT INTO conversations_fts(rowid, content, project_name) VALUES (2, 'Help me with TypeScript types', 'web-app')",
                [],
            )
            .unwrap();

            // Search for Rust content
            let mut stmt = conn
                .prepare("SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'Rust'")
                .unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            assert_eq!(results, vec![1], "Should find the Rust conversation");

            // Search for TypeScript content
            let mut stmt = conn
                .prepare("SELECT rowid FROM conversations_fts WHERE conversations_fts MATCH 'TypeScript'")
                .unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            assert_eq!(results, vec![2], "Should find the TypeScript conversation");

            // Search by project name
            let mut stmt = conn
                .prepare("SELECT rowid FROM conversations_fts WHERE project_name MATCH 'rust'")
                .unwrap();
            let results: Vec<i64> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            assert_eq!(results, vec![1], "Should find by project name");

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_schema_idempotent() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::open(db_path).unwrap();

        // Should succeed multiple times
        db.init_schema().unwrap();
        db.init_schema().unwrap();
        db.init_schema().unwrap();
    }

    #[test]
    fn test_with_connection() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::open(db_path).unwrap();
        db.init_schema().unwrap();

        // Test read operation
        let count = db
            .with_connection(|conn| {
                let count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
                    .unwrap();
                Ok(count)
            })
            .unwrap();

        assert_eq!(count, 0);
    }
}
