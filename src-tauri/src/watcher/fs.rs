//! File system watcher implementation using the `notify` crate.
//!
//! Watches `~/.claude/projects/` for new/modified JSONL files and triggers
//! incremental parsing and indexing when changes are detected.

use crate::db::metadata::{get_modified_files, update_file_metadata};
use crate::db::sqlite::Database;
use crate::parser::jsonl::{discover_jsonl_files, get_claude_projects_dir, parse_conversation_file};
use crate::search::index::index_conversation_content;
use crate::state::AppState;
use notify::{
    event::{CreateKind, ModifyKind},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Debounce duration for rapid file changes (100ms as per PRD).
const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// Watcher-related errors.
#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("Failed to get Claude projects directory: {0}")]
    ProjectsDirNotFound(String),

    #[error("Failed to create file watcher: {0}")]
    WatcherCreation(String),

    #[error("Failed to start watching: {0}")]
    WatchStart(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Parser error: {0}")]
    Parser(String),
}

/// Event name for conversations updated events sent to frontend.
pub const CONVERSATIONS_UPDATED_EVENT: &str = "conversations-updated";

/// Payload for the conversations-updated event.
#[derive(Clone, serde::Serialize)]
pub struct ConversationsUpdatedPayload {
    /// Number of new conversations added.
    pub new_count: usize,
    /// Number of existing conversations updated.
    pub updated_count: usize,
    /// Whether this was triggered by file watcher (vs initial load).
    pub from_watcher: bool,
}

/// Handle to control the file watcher.
pub struct WatcherHandle {
    /// Flag to signal the watcher thread to stop.
    stop_flag: Arc<AtomicBool>,
    /// Join handle for the watcher thread.
    thread_handle: Option<JoinHandle<()>>,
}

impl WatcherHandle {
    /// Signals the watcher to stop and waits for it to finish.
    pub fn stop(mut self) {
        info!("Stopping file watcher...");
        self.stop_flag.store(true, Ordering::SeqCst);

        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                warn!("Error joining watcher thread: {:?}", e);
            }
        }

        info!("File watcher stopped");
    }
}

/// Stops the file watcher by signaling it to stop.
pub fn stop_watcher(handle: WatcherHandle) {
    handle.stop();
}

/// Starts the file system watcher in a background thread.
///
/// Watches `~/.claude/projects/` for new/modified `.jsonl` files.
/// When changes are detected:
/// 1. Debounces rapid changes (100ms)
/// 2. Parses the modified file
/// 3. Updates the database and search index
/// 4. Refreshes the AppState cache
/// 5. Emits a Tauri event to the frontend
///
/// # Arguments
/// * `app_handle` - Tauri app handle for emitting events to frontend
/// * `app_state` - Shared application state with database and cache
///
/// # Returns
/// * `Ok(WatcherHandle)` - Handle to control the watcher
/// * `Err(WatcherError)` - If watcher creation fails
pub fn start_watcher(
    app_handle: AppHandle,
    app_state: Arc<AppState>,
) -> Result<WatcherHandle, WatcherError> {
    // Get the Claude projects directory
    let projects_dir = get_claude_projects_dir()
        .map_err(|e| WatcherError::ProjectsDirNotFound(e.to_string()))?;

    // Check if directory exists
    if !projects_dir.exists() {
        warn!(
            "Claude projects directory does not exist: {:?}. Watcher will still run.",
            projects_dir
        );
    }

    info!("Starting file watcher for: {:?}", projects_dir);

    // Create channel for receiving events
    let (tx, rx) = mpsc::channel::<Event>();

    // Create the watcher
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // Only send relevant events (creates and modifies)
                match event.kind {
                    EventKind::Create(CreateKind::File)
                    | EventKind::Modify(ModifyKind::Data(_))
                    | EventKind::Modify(ModifyKind::Any) => {
                        // Filter to only JSONL files
                        let has_jsonl = event.paths.iter().any(|p| {
                            p.extension()
                                .map(|ext| ext == "jsonl")
                                .unwrap_or(false)
                        });
                        if has_jsonl {
                            let _ = tx.send(event);
                        }
                    }
                    _ => {}
                }
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )
    .map_err(|e| WatcherError::WatcherCreation(e.to_string()))?;

    // Start watching the directory
    watcher
        .watch(&projects_dir, RecursiveMode::Recursive)
        .map_err(|e| WatcherError::WatchStart(e.to_string()))?;

    // Create stop flag
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    // Spawn the watcher thread
    let thread_handle = thread::spawn(move || {
        // Keep watcher alive in this scope
        let _watcher = watcher;

        // Track pending files and last event time for debouncing
        let mut pending_files: HashSet<PathBuf> = HashSet::new();
        let mut last_event_time: Option<Instant> = None;

        loop {
            // Check if we should stop
            if stop_flag_clone.load(Ordering::SeqCst) {
                debug!("Watcher thread received stop signal");
                break;
            }

            // Try to receive with timeout
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(event) => {
                    // Add paths to pending set
                    for path in event.paths {
                        if path.extension().map(|ext| ext == "jsonl").unwrap_or(false) {
                            debug!("File change detected: {:?}", path);
                            pending_files.insert(path);
                        }
                    }
                    last_event_time = Some(Instant::now());
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if we have pending files and debounce time has passed
                    if !pending_files.is_empty() {
                        if let Some(last_time) = last_event_time {
                            if last_time.elapsed() >= DEBOUNCE_DURATION {
                                // Process pending files
                                let files: Vec<PathBuf> = pending_files.drain().collect();
                                info!("Processing {} changed files after debounce", files.len());

                                if let Err(e) = process_changed_files(
                                    &files,
                                    &app_handle,
                                    &app_state,
                                ) {
                                    error!("Error processing changed files: {}", e);
                                }

                                last_event_time = None;
                            }
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    warn!("Watcher channel disconnected");
                    break;
                }
            }
        }
    });

    Ok(WatcherHandle {
        stop_flag,
        thread_handle: Some(thread_handle),
    })
}

/// Processes changed files: parses, updates database, and emits events.
fn process_changed_files(
    changed_paths: &[PathBuf],
    app_handle: &AppHandle,
    app_state: &Arc<AppState>,
) -> Result<(), WatcherError> {
    let db = app_state.db();

    // Discover all JSONL files to get current state
    let all_files = discover_jsonl_files()
        .map_err(|e| WatcherError::Parser(e.to_string()))?;

    // Get modified files from database comparison
    let modified_files = db
        .with_connection(|conn| get_modified_files(conn, &all_files))
        .map_err(|e| WatcherError::Database(e.to_string()))?;

    // Filter to only files that were actually changed (from watcher)
    let files_to_process: Vec<_> = modified_files
        .into_iter()
        .filter(|f| changed_paths.contains(&f.file_path))
        .collect();

    if files_to_process.is_empty() {
        debug!("No files need processing after filtering");
        return Ok(());
    }

    info!("Processing {} modified files", files_to_process.len());

    let mut new_count = 0;
    let mut updated_count = 0;

    // Process each file
    for modified_file in &files_to_process {
        match process_single_file(&db, &modified_file.file_path, &modified_file.current_modified_at)
        {
            Ok(count) => {
                if modified_file.is_new {
                    new_count += count;
                } else {
                    updated_count += count;
                }
            }
            Err(e) => {
                error!(
                    "Error processing file {:?}: {}",
                    modified_file.file_path, e
                );
            }
        }
    }

    // Refresh the conversations cache
    if let Err(e) = app_state.refresh_conversations_cache() {
        error!("Error refreshing conversations cache: {}", e);
    }

    // Emit event to frontend
    let payload = ConversationsUpdatedPayload {
        new_count,
        updated_count,
        from_watcher: true,
    };

    if let Err(e) = app_handle.emit(CONVERSATIONS_UPDATED_EVENT, payload) {
        error!("Error emitting conversations-updated event: {}", e);
    } else {
        info!(
            "Emitted conversations-updated event: {} new, {} updated",
            new_count, updated_count
        );
    }

    Ok(())
}

/// Processes a single file: parses it and updates the database.
/// Returns the number of conversations processed.
fn process_single_file(
    db: &Arc<Database>,
    file_path: &PathBuf,
    modified_at: &str,
) -> Result<usize, WatcherError> {
    debug!("Processing file: {:?}", file_path);

    // Parse the file
    let conversations = parse_conversation_file(file_path)
        .map_err(|e| WatcherError::Parser(e.to_string()))?;

    if conversations.is_empty() {
        debug!("No conversations found in {:?}", file_path);
        return Ok(0);
    }

    let count = conversations.len();

    // Update database
    db.with_connection_mut(|conn| {
        let tx = conn.transaction().map_err(crate::db::sqlite::DbError::Sqlite)?;

        for conv in &conversations {
            // Generate preview from first message content
            let preview = generate_preview(&conv.messages);

            // Insert or update conversation
            tx.execute(
                r#"
                INSERT INTO conversations (
                    id, project_path, project_name, start_time, last_time,
                    preview, message_count, total_input_tokens, total_output_tokens,
                    file_path, file_modified_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(id) DO UPDATE SET
                    project_path = excluded.project_path,
                    project_name = excluded.project_name,
                    start_time = excluded.start_time,
                    last_time = excluded.last_time,
                    preview = excluded.preview,
                    message_count = excluded.message_count,
                    total_input_tokens = excluded.total_input_tokens,
                    total_output_tokens = excluded.total_output_tokens,
                    file_path = excluded.file_path,
                    file_modified_at = excluded.file_modified_at
                "#,
                rusqlite::params![
                    conv.id,
                    conv.project_path,
                    conv.project_name,
                    conv.start_time,
                    conv.last_time,
                    preview,
                    conv.messages.len(),
                    conv.total_input_tokens,
                    conv.total_output_tokens,
                    conv.file_path.to_string_lossy(),
                    modified_at,
                ],
            )
            .map_err(crate::db::sqlite::DbError::Sqlite)?;

            // Update search index (best-effort: log warning if fails but continue)
            if let Err(e) = index_conversation_content(&tx, &conv.id, &preview, &conv.project_name) {
                warn!("Error indexing conversation {}: {}", conv.id, e);
            }
        }

        // Update file metadata
        update_file_metadata(&tx, file_path, modified_at)?;

        tx.commit().map_err(crate::db::sqlite::DbError::Sqlite)?;
        Ok(())
    })
    .map_err(|e| WatcherError::Database(e.to_string()))?;

    debug!("Processed {} conversations from {:?}", count, file_path);
    Ok(count)
}

/// Generates a preview string from conversation messages.
fn generate_preview(messages: &[crate::parser::jsonl::RawMessage]) -> String {
    use crate::parser::jsonl::RawContent;

    // Find first user message for preview
    for msg in messages {
        if let crate::parser::jsonl::RawMessageType::User = msg.message_type {
            match &msg.message.content {
                RawContent::Text(text) => {
                    // Truncate to reasonable preview length
                    let preview = text.chars().take(200).collect::<String>();
                    return preview.replace('\n', " ").trim().to_string();
                }
                RawContent::Blocks(blocks) => {
                    // Get text from first text block
                    for block in blocks {
                        if block.block_type == "text" {
                            if let Some(text) = &block.text {
                                let preview = text.chars().take(200).collect::<String>();
                                return preview.replace('\n', " ").trim().to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to first message of any type
    if let Some(first) = messages.first() {
        match &first.message.content {
            RawContent::Text(text) => {
                let preview = text.chars().take(200).collect::<String>();
                return preview.replace('\n', " ").trim().to_string();
            }
            RawContent::Blocks(blocks) => {
                for block in blocks {
                    if let Some(text) = &block.text {
                        let preview = text.chars().take(200).collect::<String>();
                        return preview.replace('\n', " ").trim().to_string();
                    }
                }
            }
        }
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debounce_duration() {
        assert_eq!(DEBOUNCE_DURATION, Duration::from_millis(100));
    }

    #[test]
    fn test_generate_preview_text_content() {
        use crate::parser::jsonl::{RawContent, RawInnerMessage, RawMessage, RawMessageType};

        let messages = vec![RawMessage {
            message_type: RawMessageType::User,
            message: RawInnerMessage {
                content: RawContent::Text("Hello, how can I help you today?".to_string()),
                role: Some("user".to_string()),
            },
            timestamp: None,
            token_count: None,
            uuid: None,
            session_id: None,
        }];

        let preview = generate_preview(&messages);
        assert_eq!(preview, "Hello, how can I help you today?");
    }

    #[test]
    fn test_generate_preview_truncates_long_text() {
        use crate::parser::jsonl::{RawContent, RawInnerMessage, RawMessage, RawMessageType};

        let long_text = "a".repeat(300);
        let messages = vec![RawMessage {
            message_type: RawMessageType::User,
            message: RawInnerMessage {
                content: RawContent::Text(long_text),
                role: Some("user".to_string()),
            },
            timestamp: None,
            token_count: None,
            uuid: None,
            session_id: None,
        }];

        let preview = generate_preview(&messages);
        assert_eq!(preview.len(), 200);
    }

    #[test]
    fn test_generate_preview_removes_newlines() {
        use crate::parser::jsonl::{RawContent, RawInnerMessage, RawMessage, RawMessageType};

        let messages = vec![RawMessage {
            message_type: RawMessageType::User,
            message: RawInnerMessage {
                content: RawContent::Text("Line 1\nLine 2\nLine 3".to_string()),
                role: Some("user".to_string()),
            },
            timestamp: None,
            token_count: None,
            uuid: None,
            session_id: None,
        }];

        let preview = generate_preview(&messages);
        assert!(!preview.contains('\n'));
        assert_eq!(preview, "Line 1 Line 2 Line 3");
    }

    #[test]
    fn test_generate_preview_empty_messages() {
        let messages: Vec<crate::parser::jsonl::RawMessage> = vec![];
        let preview = generate_preview(&messages);
        assert!(preview.is_empty());
    }
}
