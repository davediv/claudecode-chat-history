//! File system watcher for detecting new/modified conversations.
//!
//! This module uses the `notify` crate to watch `~/.claude/projects/`
//! for changes and trigger incremental updates.
