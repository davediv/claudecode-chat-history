// Backend modules
pub mod commands;
pub mod db;
pub mod models;
pub mod parser;
pub mod search;
pub mod state;
pub mod watcher;

use crate::db::metadata::get_modified_files;
use crate::parser::jsonl::discover_jsonl_files;
use crate::state::AppState;
use crate::watcher::{process_files_and_emit, start_watcher};
use std::sync::Arc;
use tauri::Manager;
use tracing::{error, info};

// Re-export command handlers
pub use commands::{get_all_tags, get_conversation, get_conversations, get_projects, search_conversations, set_tags, toggle_bookmark};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

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
        .plugin(tauri_plugin_dialog::init())
        .manage(db)
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![greet, get_conversations, get_conversation, get_projects, search_conversations, toggle_bookmark, set_tags, get_all_tags])
        .setup(move |app| {
            // Open devtools in debug mode
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            // Start file watcher after app is ready
            let app_handle = app.handle().clone();
            match start_watcher(app_handle.clone(), app_state_for_watcher.clone()) {
                Ok(handle) => {
                    info!("File watcher started successfully");
                    // Store handle in app state for cleanup on exit
                    // For now, we let it run for the lifetime of the app
                    std::mem::forget(handle);

                    // Perform initial scan of existing JSONL files in a background thread
                    let scan_app_handle = app_handle;
                    let scan_app_state = app_state_for_watcher;
                    std::thread::spawn(move || {
                        match discover_jsonl_files() {
                            Ok(all_files) if !all_files.is_empty() => {
                                info!("Initial scan: found {} JSONL files", all_files.len());
                                let db = scan_app_state.db();
                                match db.with_connection(|conn| get_modified_files(conn, &all_files)) {
                                    Ok(modified) if !modified.is_empty() => {
                                        info!("Initial scan: {} files need processing", modified.len());
                                        process_files_and_emit(&modified, &scan_app_handle, &scan_app_state);
                                    }
                                    Ok(_) => info!("Initial scan: all files already up to date"),
                                    Err(e) => error!("Initial scan: failed to check modified files: {}", e),
                                }
                            }
                            Ok(_) => info!("Initial scan: no JSONL files found in ~/.claude/projects/"),
                            Err(e) => error!("Initial scan: failed to discover JSONL files: {}", e),
                        }
                    });
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
