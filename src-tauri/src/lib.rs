// Backend modules
pub mod commands;
pub mod db;
pub mod models;
pub mod parser;
pub mod search;
pub mod watcher;

use crate::db::sqlite::Database;
use std::sync::Arc;
use tracing::info;

// Re-export command handlers
pub use commands::get_conversations;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database
    let db = Database::open_default().expect("Failed to open database");
    db.init_schema().expect("Failed to initialize database schema");
    info!("Database initialized at {:?}", db.path());

    // Wrap in Arc for shared state
    let db = Arc::new(db);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(db)
        .invoke_handler(tauri::generate_handler![greet, get_conversations])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
