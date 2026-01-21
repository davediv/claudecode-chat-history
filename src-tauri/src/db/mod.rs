//! Database operations and connection management.
//!
//! This module handles SQLite database initialization, connection pooling,
//! schema creation, and CRUD operations for conversation data.

pub mod metadata;
pub mod sqlite;

pub use metadata::{
    clear_all_metadata, get_all_file_metadata, get_modified_files, is_metadata_empty,
    remove_stale_metadata, update_file_metadata, update_file_metadata_batch, FileMetadata,
    ModifiedFile,
};
pub use sqlite::{Database, DbError, DbResult, init_db};
