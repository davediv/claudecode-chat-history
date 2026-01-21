//! SQLite FTS5 full-text search indexing.
//!
//! This module handles building and querying the FTS5 search index
//! for conversation content and metadata.

pub mod index;

pub use index::{
    build_search_index, clear_search_index, get_index_count, index_conversation,
    rebuild_search_index, remove_from_index,
};
