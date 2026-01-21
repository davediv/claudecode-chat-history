//! Application state management.
//!
//! This module provides shared application state with thread-safe access
//! to the database connection and cached conversation data.

use crate::db::sqlite::{Database, DbResult};
use crate::models::ConversationSummary;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Application state shared across all Tauri commands.
///
/// Provides thread-safe access to:
/// - Database connection (via `Database` which has internal `Mutex<Connection>`)
/// - Conversations cache (via `RwLock<Vec<ConversationSummary>>`)
pub struct AppState {
    /// Database connection manager.
    db: Arc<Database>,
    /// Cached conversation summaries for faster list retrieval.
    conversations_cache: RwLock<Vec<ConversationSummary>>,
}

impl AppState {
    /// Creates a new AppState with default database location.
    ///
    /// Opens the database, initializes the schema, and creates an empty cache.
    pub fn new() -> DbResult<Self> {
        let db = Database::open_default()?;
        db.init_schema()?;

        info!("AppState initialized with database at {:?}", db.path());

        Ok(Self {
            db: Arc::new(db),
            conversations_cache: RwLock::new(Vec::new()),
        })
    }

    /// Creates a new AppState with a specific database.
    ///
    /// Useful for testing with in-memory or custom database paths.
    pub fn with_database(db: Database) -> Self {
        Self {
            db: Arc::new(db),
            conversations_cache: RwLock::new(Vec::new()),
        }
    }

    /// Returns a reference to the database (as Arc for shared ownership).
    pub fn db(&self) -> Arc<Database> {
        Arc::clone(&self.db)
    }

    /// Returns the cached conversation summaries.
    ///
    /// Returns an empty vector if the cache hasn't been populated or is poisoned.
    pub fn get_cached_conversations(&self) -> Vec<ConversationSummary> {
        match self.conversations_cache.read() {
            Ok(cache) => cache.clone(),
            Err(poisoned) => {
                // If poisoned, still try to return data
                debug!("Cache lock was poisoned, recovering");
                poisoned.into_inner().clone()
            }
        }
    }

    /// Updates the conversations cache with new data.
    pub fn set_cached_conversations(&self, conversations: Vec<ConversationSummary>) {
        match self.conversations_cache.write() {
            Ok(mut cache) => {
                *cache = conversations;
                debug!("Conversations cache updated with {} items", cache.len());
            }
            Err(poisoned) => {
                // Recover from poisoned lock
                debug!("Cache lock was poisoned, recovering and updating");
                let mut cache = poisoned.into_inner();
                *cache = conversations;
            }
        }
    }

    /// Refreshes the conversations cache from the database.
    ///
    /// Loads all conversation summaries sorted by last_time descending.
    pub fn refresh_conversations_cache(&self) -> DbResult<()> {
        let conversations = self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT c.id, c.project_name, c.start_time, c.last_time, c.preview, c.message_count,
                       (SELECT 1 FROM bookmarks b WHERE b.conversation_id = c.id) IS NOT NULL as bookmarked
                FROM conversations c
                ORDER BY c.last_time DESC
                "#,
            )?;

            let rows = stmt.query_map([], |row| {
                Ok(ConversationSummary {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    start_time: row.get(2)?,
                    last_time: row.get(3)?,
                    preview: row.get(4)?,
                    message_count: row.get(5)?,
                    bookmarked: row.get::<_, i32>(6)? != 0,
                })
            })?;

            let mut results = Vec::new();
            for row_result in rows {
                results.push(row_result?);
            }

            Ok(results)
        })?;

        let count = conversations.len();
        self.set_cached_conversations(conversations);
        info!("Conversations cache refreshed with {} items", count);

        Ok(())
    }

    /// Clears the conversations cache.
    pub fn clear_cache(&self) {
        self.set_cached_conversations(Vec::new());
        debug!("Conversations cache cleared");
    }

    /// Returns the number of cached conversations.
    pub fn cache_size(&self) -> usize {
        match self.conversations_cache.read() {
            Ok(cache) => cache.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    /// Checks if the cache is empty.
    pub fn is_cache_empty(&self) -> bool {
        self.cache_size() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_state() -> AppState {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(db_path).unwrap();
        db.init_schema().unwrap();
        AppState::with_database(db)
    }

    #[test]
    fn test_new_state_empty_cache() {
        let state = setup_test_state();
        assert!(state.is_cache_empty());
        assert_eq!(state.cache_size(), 0);
    }

    #[test]
    fn test_set_and_get_cache() {
        let state = setup_test_state();

        let conversations = vec![
            ConversationSummary {
                id: "conv1".to_string(),
                project_name: "project-a".to_string(),
                start_time: "2025-01-01T00:00:00Z".to_string(),
                last_time: "2025-01-01T01:00:00Z".to_string(),
                preview: "Hello world".to_string(),
                message_count: 5,
                bookmarked: false,
            },
            ConversationSummary {
                id: "conv2".to_string(),
                project_name: "project-b".to_string(),
                start_time: "2025-01-02T00:00:00Z".to_string(),
                last_time: "2025-01-02T01:00:00Z".to_string(),
                preview: "Another conversation".to_string(),
                message_count: 10,
                bookmarked: true,
            },
        ];

        state.set_cached_conversations(conversations);

        assert_eq!(state.cache_size(), 2);
        assert!(!state.is_cache_empty());

        let cached = state.get_cached_conversations();
        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].id, "conv1");
        assert_eq!(cached[1].id, "conv2");
    }

    #[test]
    fn test_clear_cache() {
        let state = setup_test_state();

        let conversations = vec![ConversationSummary {
            id: "conv1".to_string(),
            project_name: "project".to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            last_time: "2025-01-01T01:00:00Z".to_string(),
            preview: "Test".to_string(),
            message_count: 1,
            bookmarked: false,
        }];

        state.set_cached_conversations(conversations);
        assert_eq!(state.cache_size(), 1);

        state.clear_cache();
        assert!(state.is_cache_empty());
    }

    #[test]
    fn test_refresh_cache_empty_db() {
        let state = setup_test_state();

        // Refresh from empty database should succeed
        let result = state.refresh_conversations_cache();
        assert!(result.is_ok());
        assert!(state.is_cache_empty());
    }

    #[test]
    fn test_refresh_cache_with_data() {
        let state = setup_test_state();

        // Insert test data directly into database
        state
            .db
            .with_connection(|conn| {
                conn.execute(
                    r#"
                INSERT INTO conversations (id, project_path, project_name, start_time, last_time, preview, message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                VALUES ('conv1', '/path/a', 'project-a', '2025-01-01T00:00:00Z', '2025-01-01T01:00:00Z', 'Test preview', 5, 100, 200, '/test/file.jsonl', '2025-01-01T00:00:00Z')
                "#,
                    [],
                )?;
                conn.execute(
                    r#"
                INSERT INTO conversations (id, project_path, project_name, start_time, last_time, preview, message_count, total_input_tokens, total_output_tokens, file_path, file_modified_at)
                VALUES ('conv2', '/path/b', 'project-b', '2025-01-02T00:00:00Z', '2025-01-02T01:00:00Z', 'Another preview', 10, 150, 300, '/test/file2.jsonl', '2025-01-02T00:00:00Z')
                "#,
                    [],
                )?;
                Ok(())
            })
            .unwrap();

        // Refresh cache
        let result = state.refresh_conversations_cache();
        assert!(result.is_ok());

        assert_eq!(state.cache_size(), 2);

        // Cache should be sorted by last_time desc
        let cached = state.get_cached_conversations();
        assert_eq!(cached[0].id, "conv2"); // More recent
        assert_eq!(cached[1].id, "conv1");
    }

    #[test]
    fn test_db_access() {
        let state = setup_test_state();
        let db = state.db();

        // Should be able to use the database
        let result = db.with_connection(|conn| {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
                .unwrap();
            Ok(count)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
