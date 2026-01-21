//! Data structures and models.
//!
//! This module contains all data structures used throughout the application,
//! including `Conversation`, `Message`, `ContentBlock`, and filter types.
//! All structs derive serde traits for serialization.

use serde::{Deserialize, Serialize};

/// Token count for input/output tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenCount {
    pub input: i64,
    pub output: i64,
}

/// Content block type discriminator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentBlockType {
    Text,
    Code,
    ToolUse,
    ToolResult,
}

/// A content block within a message.
/// Messages can contain multiple blocks of different types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: ContentBlockType,
    pub content: String,
    /// Programming language for code blocks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Tool name for tool_use/tool_result blocks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

/// Message role discriminator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: Vec<ContentBlock>,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Token count for this message (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<TokenCount>,
}

/// A complete conversation with all messages.
/// Used when viewing conversation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    /// Unique ID derived from file path + session.
    pub id: String,
    /// Original project directory path.
    pub project_path: String,
    /// Display name (last path segments).
    pub project_name: String,
    /// First message timestamp (ISO 8601).
    pub start_time: String,
    /// Last message timestamp (ISO 8601).
    pub last_time: String,
    /// All messages in the conversation.
    pub messages: Vec<Message>,
    /// Total token usage.
    pub total_tokens: TokenCount,
    /// User bookmark status (MVP extension point).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmarked: Option<bool>,
    /// User-defined tags (MVP extension point).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Lightweight conversation summary for list view.
/// Does not include full message content for performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: String,
    pub project_name: String,
    /// First message timestamp (ISO 8601).
    pub start_time: String,
    /// Last message timestamp (ISO 8601).
    pub last_time: String,
    /// First user message, truncated to 100 characters.
    pub preview: String,
    /// Total number of messages.
    pub message_count: i32,
    /// Whether this conversation is bookmarked.
    #[serde(default)]
    pub bookmarked: bool,
}

/// Filter options for querying conversations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConversationFilters {
    /// Filter by project name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Start of date range (inclusive, ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_start: Option<String>,
    /// End of date range (inclusive, ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_end: Option<String>,
    /// Filter by bookmark status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmarked: Option<bool>,
}

/// A search result with matching conversation info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// ID of the matching conversation.
    pub conversation_id: String,
    /// Context snippet around the match (50 chars before/after).
    pub snippet: String,
    /// Number of matches in this conversation.
    pub match_count: i32,
    /// Search relevance rank (lower is better).
    pub rank: f64,
}

/// Project information for the project filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    /// Full project path.
    pub project_path: String,
    /// Display name.
    pub project_name: String,
    /// Number of conversations in this project.
    pub conversation_count: i32,
    /// Timestamp of most recent activity (ISO 8601).
    pub last_activity: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_block_serialization() {
        let block = ContentBlock {
            block_type: ContentBlockType::Code,
            content: "fn main() {}".to_string(),
            language: Some("rust".to_string()),
            tool_name: None,
        };

        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"code\""));
        assert!(json.contains("\"language\":\"rust\""));
        assert!(!json.contains("toolName")); // Should be skipped when None

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.block_type, ContentBlockType::Code);
        assert_eq!(deserialized.language, Some("rust".to_string()));
    }

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::Assistant;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"assistant\"");

        let deserialized: MessageRole = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MessageRole::Assistant);
    }

    #[test]
    fn test_conversation_summary_serialization() {
        let summary = ConversationSummary {
            id: "abc123".to_string(),
            project_name: "my-project".to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            last_time: "2025-01-01T01:00:00Z".to_string(),
            preview: "How do I...".to_string(),
            message_count: 10,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"projectName\":\"my-project\""));
        assert!(json.contains("\"messageCount\":10"));

        let deserialized: ConversationSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "abc123");
    }

    #[test]
    fn test_filters_default() {
        let filters = ConversationFilters::default();
        assert!(filters.project.is_none());
        assert!(filters.date_start.is_none());
        assert!(filters.date_end.is_none());
    }
}
