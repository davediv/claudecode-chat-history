// Backend modules
pub mod commands;
pub mod db;
pub mod models;
pub mod parser;
pub mod search;
pub mod state;
pub mod watcher;

use crate::state::AppState;
use crate::watcher::start_watcher;
use std::sync::Arc;
use tracing::{error, info};

// Re-export command handlers
pub use commands::{get_conversation, get_conversations, get_projects, search_conversations};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize application state (database + cache)
    let app_state = AppState::new().expect("Failed to initialize application state");
    info!("Application state initialized");

    // Load initial cache from database
    if let Err(e) = app_state.refresh_conversations_cache() {
        info!("No cached conversations loaded (empty database or error: {})", e);
    }

    // Wrap in Arc for shared state
    let app_state = Arc::new(app_state);
    let app_state_for_watcher = app_state.clone();

    // Also provide database directly for compatibility with existing commands
    let db = app_state.db();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(db)
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![greet, get_conversations, get_conversation, get_projects, search_conversations])
        .setup(move |app| {
            // Start file watcher after app is ready
            let app_handle = app.handle().clone();
            match start_watcher(app_handle, app_state_for_watcher) {
                Ok(handle) => {
                    info!("File watcher started successfully");
                    // Store handle in app state for cleanup on exit
                    // For now, we let it run for the lifetime of the app
                    std::mem::forget(handle);
                }
                Err(e) => {
                    error!("Failed to start file watcher: {}. App will still work but won't detect new conversations.", e);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
