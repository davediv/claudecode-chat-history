//! File system watcher for detecting new/modified conversations.
//!
//! This module uses the `notify` crate to watch `~/.claude/projects/`
//! for changes and trigger incremental updates.

pub mod fs;

pub use fs::{start_watcher, stop_watcher, WatcherError, WatcherHandle};
