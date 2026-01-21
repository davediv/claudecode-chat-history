//! JSONL parsing logic.
//!
//! This module handles discovery and parsing of Claude Code JSONL conversation files
//! from `~/.claude/projects/`. Includes line parsing, conversation aggregation,
//! and content block extraction.

pub mod content;
pub mod jsonl;

pub use content::{extract_preview, parse_content_blocks};
pub use jsonl::{
    discover_jsonl_files, get_claude_projects_dir, parse_conversation_file, parse_jsonl_line,
    ParsedConversation, ParserError, ParserResult, RawContent, RawContentBlock, RawInnerMessage,
    RawMessage, RawMessageType, RawTokenCount,
};
