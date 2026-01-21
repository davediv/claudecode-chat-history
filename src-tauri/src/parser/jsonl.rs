//! JSONL file discovery and parsing.
//!
//! This module handles finding and reading Claude Code JSONL conversation files
//! from the `~/.claude/projects/` directory.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Parser-related errors.
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to get home directory")]
    HomeNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value for '{field}': {reason}")]
    InvalidField { field: String, reason: String },
}

/// Result type for parser operations.
pub type ParserResult<T> = Result<T, ParserError>;

/// Raw message type from JSONL (user, assistant, or system).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RawMessageType {
    User,
    Assistant,
    System,
}

/// Raw content block from JSONL.
/// Represents a single content block that may appear in the content array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    /// Text content (for text blocks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Tool name (for tool_use blocks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tool input (for tool_use blocks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,
    /// Tool use ID (for tool_use and tool_result blocks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// Tool result content (for tool_result blocks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Value>,
}

/// Raw content that can be either a string or an array of content blocks.
#[derive(Debug, Clone)]
pub enum RawContent {
    /// Simple text content.
    Text(String),
    /// Array of content blocks.
    Blocks(Vec<RawContentBlock>),
}

/// Raw inner message structure from JSONL.
#[derive(Debug, Clone)]
pub struct RawInnerMessage {
    pub content: RawContent,
    pub role: Option<String>,
}

/// Raw token count from JSONL.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawTokenCount {
    #[serde(default)]
    pub input: i64,
    #[serde(default)]
    pub output: i64,
}

/// Raw message parsed from a single JSONL line.
/// Contains the unprocessed data directly from the file.
#[derive(Debug, Clone)]
pub struct RawMessage {
    /// Message type (user, assistant, system).
    pub message_type: RawMessageType,
    /// Inner message structure containing content.
    pub message: RawInnerMessage,
    /// ISO 8601 timestamp.
    pub timestamp: Option<String>,
    /// Token count for this message.
    pub token_count: Option<RawTokenCount>,
    /// Unique message ID.
    pub uuid: Option<String>,
    /// Session ID this message belongs to.
    pub session_id: Option<String>,
}

/// Parses a single JSONL line into a RawMessage.
///
/// Handles both string and array content formats as specified in the PRD.
/// Returns descriptive errors for invalid or malformed JSON.
///
/// # Arguments
/// * `line` - A single line from a JSONL file
///
/// # Returns
/// * `Ok(RawMessage)` - Successfully parsed message
/// * `Err(ParserError)` - Parse error with description
///
/// # Example
/// ```ignore
/// let line = r#"{"type":"user","message":{"content":"Hello","role":"user"},"timestamp":"2025-01-01T00:00:00Z"}"#;
/// let msg = parse_jsonl_line(line)?;
/// assert_eq!(msg.message_type, RawMessageType::User);
/// ```
pub fn parse_jsonl_line(line: &str) -> ParserResult<RawMessage> {
    // Skip empty lines
    let line = line.trim();
    if line.is_empty() {
        return Err(ParserError::MissingField("line is empty".to_string()));
    }

    // Parse as generic JSON Value first
    let value: Value = serde_json::from_str(line)?;

    // Extract required 'type' field
    let message_type = match value.get("type") {
        Some(Value::String(t)) => match t.as_str() {
            "user" => RawMessageType::User,
            "assistant" => RawMessageType::Assistant,
            "system" => RawMessageType::System,
            other => {
                return Err(ParserError::InvalidField {
                    field: "type".to_string(),
                    reason: format!("unknown message type: {}", other),
                })
            }
        },
        Some(_) => {
            return Err(ParserError::InvalidField {
                field: "type".to_string(),
                reason: "expected string".to_string(),
            })
        }
        None => return Err(ParserError::MissingField("type".to_string())),
    };

    // Extract 'message' field
    let message_value = value
        .get("message")
        .ok_or_else(|| ParserError::MissingField("message".to_string()))?;

    // Parse inner message content
    let inner_message = parse_inner_message(message_value)?;

    // Extract optional fields
    let timestamp = value
        .get("timestamp")
        .and_then(|v| v.as_str())
        .map(String::from);

    let uuid = value.get("uuid").and_then(|v| v.as_str()).map(String::from);

    let session_id = value
        .get("sessionId")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Parse token count (optional)
    let token_count = value.get("tokenCount").and_then(|v| {
        serde_json::from_value::<RawTokenCount>(v.clone())
            .ok()
            .or_else(|| Some(RawTokenCount::default()))
    });

    Ok(RawMessage {
        message_type,
        message: inner_message,
        timestamp,
        token_count,
        uuid,
        session_id,
    })
}

/// Parses the inner message structure.
fn parse_inner_message(value: &Value) -> ParserResult<RawInnerMessage> {
    // Extract role (optional)
    let role = value
        .get("role")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Extract content (required) - can be string or array
    let content = match value.get("content") {
        Some(Value::String(s)) => RawContent::Text(s.clone()),
        Some(Value::Array(arr)) => {
            let blocks: Result<Vec<RawContentBlock>, _> = arr
                .iter()
                .map(|v| serde_json::from_value(v.clone()))
                .collect();
            RawContent::Blocks(blocks?)
        }
        Some(Value::Null) => RawContent::Text(String::new()),
        Some(_) => {
            return Err(ParserError::InvalidField {
                field: "message.content".to_string(),
                reason: "expected string or array".to_string(),
            })
        }
        None => {
            // Content might be missing in some formats - default to empty
            RawContent::Text(String::new())
        }
    };

    Ok(RawInnerMessage { content, role })
}

/// A parsed conversation aggregated from JSONL messages.
/// Contains all messages grouped by session ID with calculated metadata.
#[derive(Debug, Clone)]
pub struct ParsedConversation {
    /// Unique ID derived from hash of file path + session ID.
    pub id: String,
    /// Original project directory path (extracted from file path).
    pub project_path: String,
    /// Display name (last 2 path segments).
    pub project_name: String,
    /// First message timestamp (ISO 8601).
    pub start_time: String,
    /// Last message timestamp (ISO 8601).
    pub last_time: String,
    /// All raw messages in chronological order.
    pub messages: Vec<RawMessage>,
    /// Total input tokens across all messages.
    pub total_input_tokens: i64,
    /// Total output tokens across all messages.
    pub total_output_tokens: i64,
    /// Session ID from the JSONL file.
    pub session_id: String,
    /// Source file path.
    pub file_path: PathBuf,
}

/// Parses a JSONL conversation file and groups messages by session ID.
///
/// Reads the file line by line, parses each line, and groups messages
/// into conversations. Calculates metadata like timestamps and token counts.
///
/// # Arguments
/// * `file_path` - Path to the JSONL file
///
/// # Returns
/// * `Ok(Vec<ParsedConversation>)` - List of conversations found in the file
/// * Empty vec if file is empty or contains no valid messages
///
/// # Example
/// ```ignore
/// let conversations = parse_conversation_file(Path::new("/path/to/session.jsonl"))?;
/// for conv in conversations {
///     println!("Conversation {} has {} messages", conv.id, conv.messages.len());
/// }
/// ```
pub fn parse_conversation_file(file_path: &Path) -> ParserResult<Vec<ParsedConversation>> {
    debug!("Parsing conversation file: {:?}", file_path);

    // Open the file
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    // Group messages by session ID
    let mut sessions: HashMap<String, Vec<RawMessage>> = HashMap::new();
    let mut line_number = 0;
    let mut parse_errors = 0;

    for line_result in reader.lines() {
        line_number += 1;
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                warn!("Failed to read line {} in {:?}: {}", line_number, file_path, e);
                parse_errors += 1;
                continue;
            }
        };

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse the line
        match parse_jsonl_line(&line) {
            Ok(msg) => {
                // Use session_id if present, otherwise use "default"
                let session_id = msg.session_id.clone().unwrap_or_else(|| "default".to_string());
                sessions.entry(session_id).or_default().push(msg);
            }
            Err(e) => {
                warn!(
                    "Failed to parse line {} in {:?}: {}",
                    line_number, file_path, e
                );
                parse_errors += 1;
            }
        }
    }

    if parse_errors > 0 {
        debug!(
            "Encountered {} parse errors in {:?} ({} lines total)",
            parse_errors, file_path, line_number
        );
    }

    // Extract project info from file path
    let (project_path, project_name) = extract_project_info(file_path);

    // Build conversations from sessions
    let mut conversations = Vec::new();
    for (session_id, messages) in sessions {
        if messages.is_empty() {
            continue;
        }

        // Sort messages by timestamp (if available)
        let mut sorted_messages = messages;
        sorted_messages.sort_by(|a, b| {
            let time_a = a.timestamp.as_deref().unwrap_or("");
            let time_b = b.timestamp.as_deref().unwrap_or("");
            time_a.cmp(time_b)
        });

        // Calculate metadata
        let start_time = sorted_messages
            .first()
            .and_then(|m| m.timestamp.clone())
            .unwrap_or_default();

        let last_time = sorted_messages
            .last()
            .and_then(|m| m.timestamp.clone())
            .unwrap_or_default();

        let (total_input_tokens, total_output_tokens) =
            calculate_total_tokens(&sorted_messages);

        // Generate unique ID
        let id = generate_conversation_id(file_path, &session_id);

        conversations.push(ParsedConversation {
            id,
            project_path: project_path.clone(),
            project_name: project_name.clone(),
            start_time,
            last_time,
            messages: sorted_messages,
            total_input_tokens,
            total_output_tokens,
            session_id,
            file_path: file_path.to_path_buf(),
        });
    }

    // Sort conversations by start time (newest first)
    conversations.sort_by(|a, b| b.start_time.cmp(&a.start_time));

    info!(
        "Parsed {} conversations from {:?}",
        conversations.len(),
        file_path
    );
    Ok(conversations)
}

/// Extracts project path and name from a JSONL file path.
///
/// The file path structure is: `~/.claude/projects/{project-hash}/{session}.jsonl`
/// The project path is the parent directory, and the project name is derived
/// from the last 2 path segments before the hash.
fn extract_project_info(file_path: &Path) -> (String, String) {
    // Get the parent directory (project hash directory)
    let project_dir = file_path.parent().unwrap_or(Path::new(""));
    let project_path = project_dir.to_string_lossy().to_string();

    // Try to get a meaningful project name from the path
    // The structure is: ~/.claude/projects/{project-hash}
    // The hash directory name is typically based on the original project path
    let project_name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    (project_path, project_name)
}

/// Calculates total input and output tokens from a list of messages.
fn calculate_total_tokens(messages: &[RawMessage]) -> (i64, i64) {
    let mut total_input = 0i64;
    let mut total_output = 0i64;

    for msg in messages {
        if let Some(ref tokens) = msg.token_count {
            total_input += tokens.input;
            total_output += tokens.output;
        }
    }

    (total_input, total_output)
}

/// Generates a unique, deterministic conversation ID from file path and session ID.
///
/// Uses a simple hash to create a short, reproducible ID.
fn generate_conversation_id(file_path: &Path, session_id: &str) -> String {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    file_path.to_string_lossy().hash(&mut hasher);
    session_id.hash(&mut hasher);
    let hash = hasher.finish();

    // Format as hex string, taking first 12 characters for brevity
    format!("{:016x}", hash)[..12].to_string()
}

/// Gets the Claude projects directory path.
///
/// Returns `~/.claude/projects/` on all platforms.
pub fn get_claude_projects_dir() -> ParserResult<PathBuf> {
    let home = dirs::home_dir().ok_or(ParserError::HomeNotFound)?;
    Ok(home.join(".claude").join("projects"))
}

/// Discovers all JSONL files in the Claude projects directory.
///
/// Recursively searches `~/.claude/projects/` for `.jsonl` files.
/// Returns files sorted by modification time (newest first).
///
/// # Returns
/// - `Vec<PathBuf>` - List of JSONL file paths, newest first
/// - Empty vec if the directory doesn't exist or is inaccessible
///
/// # Example
/// ```ignore
/// let files = discover_jsonl_files()?;
/// for file in files {
///     println!("Found: {:?}", file);
/// }
/// ```
pub fn discover_jsonl_files() -> ParserResult<Vec<PathBuf>> {
    let projects_dir = get_claude_projects_dir()?;

    if !projects_dir.exists() {
        debug!("Claude projects directory does not exist: {:?}", projects_dir);
        return Ok(Vec::new());
    }

    let mut files = collect_jsonl_files(&projects_dir);

    // Sort by modification time (newest first)
    files.sort_by(|a, b| {
        let time_a = fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let time_b = fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        time_b.cmp(&time_a) // Reverse order for newest first
    });

    debug!("Discovered {} JSONL files", files.len());
    Ok(files)
}

/// Recursively collects all JSONL files from a directory.
fn collect_jsonl_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Cannot read directory {:?}: {}", dir, e);
            return files;
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectories
            files.extend(collect_jsonl_files(&path));
        } else if path.is_file() {
            // Check if it's a JSONL file
            if let Some(ext) = path.extension() {
                if ext == "jsonl" {
                    // Verify we can read the file
                    match fs::metadata(&path) {
                        Ok(_) => {
                            debug!("Found JSONL file: {:?}", path);
                            files.push(path);
                        }
                        Err(e) => {
                            warn!("Cannot access file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_discover_empty_directory() {
        let temp_dir = tempdir().unwrap();

        // Create a mock function that uses our temp dir
        let files = collect_jsonl_files(&temp_dir.path().to_path_buf());
        assert!(files.is_empty(), "Empty directory should return no files");
    }

    #[test]
    fn test_discover_jsonl_files() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create directory structure like ~/.claude/projects/
        let project1 = root.join("project-hash-1");
        let project2 = root.join("project-hash-2");
        fs::create_dir_all(&project1).unwrap();
        fs::create_dir_all(&project2).unwrap();

        // Create JSONL files
        let file1 = project1.join("session1.jsonl");
        let file2 = project1.join("session2.jsonl");
        let file3 = project2.join("session3.jsonl");

        File::create(&file1).unwrap().write_all(b"{}").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        File::create(&file2).unwrap().write_all(b"{}").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        File::create(&file3).unwrap().write_all(b"{}").unwrap();

        // Also create a non-JSONL file (should be ignored)
        let other_file = project1.join("notes.txt");
        File::create(&other_file).unwrap().write_all(b"notes").unwrap();

        let files = collect_jsonl_files(&root.to_path_buf());

        assert_eq!(files.len(), 3, "Should find exactly 3 JSONL files");

        // Verify all are .jsonl files
        for file in &files {
            assert_eq!(
                file.extension().unwrap(),
                "jsonl",
                "All files should be .jsonl"
            );
        }
    }

    #[test]
    fn test_files_sorted_by_modification_time() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create files with different modification times
        let file1 = root.join("old.jsonl");
        let file2 = root.join("middle.jsonl");
        let file3 = root.join("new.jsonl");

        File::create(&file1).unwrap().write_all(b"{}").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        File::create(&file2).unwrap().write_all(b"{}").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        File::create(&file3).unwrap().write_all(b"{}").unwrap();

        let mut files = collect_jsonl_files(&root.to_path_buf());

        // Sort by modification time (newest first)
        files.sort_by(|a, b| {
            let time_a = fs::metadata(a)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let time_b = fs::metadata(b)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            time_b.cmp(&time_a)
        });

        assert_eq!(files.len(), 3);
        assert!(
            files[0].file_name().unwrap() == "new.jsonl",
            "Newest file should be first"
        );
        assert!(
            files[2].file_name().unwrap() == "old.jsonl",
            "Oldest file should be last"
        );
    }

    #[test]
    fn test_get_claude_projects_dir() {
        let result = get_claude_projects_dir();
        assert!(result.is_ok(), "Should be able to get Claude projects dir");

        let path = result.unwrap();
        assert!(
            path.ends_with(".claude/projects"),
            "Path should end with .claude/projects"
        );
    }

    // ========== parse_jsonl_line tests ==========

    #[test]
    fn test_parse_valid_user_message_string_content() {
        let line = r#"{"type":"user","message":{"content":"Hello, how are you?","role":"user"},"timestamp":"2025-01-15T10:30:00Z","uuid":"abc-123","sessionId":"session-456"}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should parse valid user message");

        let msg = result.unwrap();
        assert_eq!(msg.message_type, RawMessageType::User);
        assert_eq!(msg.timestamp, Some("2025-01-15T10:30:00Z".to_string()));
        assert_eq!(msg.uuid, Some("abc-123".to_string()));
        assert_eq!(msg.session_id, Some("session-456".to_string()));
        assert_eq!(msg.message.role, Some("user".to_string()));

        match &msg.message.content {
            RawContent::Text(text) => assert_eq!(text, "Hello, how are you?"),
            RawContent::Blocks(_) => panic!("Expected text content, got blocks"),
        }
    }

    #[test]
    fn test_parse_valid_assistant_message_with_tokens() {
        let line = r#"{"type":"assistant","message":{"content":"I'm doing well, thanks!","role":"assistant"},"timestamp":"2025-01-15T10:30:05Z","tokenCount":{"input":10,"output":25}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should parse valid assistant message");

        let msg = result.unwrap();
        assert_eq!(msg.message_type, RawMessageType::Assistant);

        let tokens = msg.token_count.unwrap();
        assert_eq!(tokens.input, 10);
        assert_eq!(tokens.output, 25);
    }

    #[test]
    fn test_parse_system_message() {
        let line = r#"{"type":"system","message":{"content":"System initialized","role":"system"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should parse system message");

        let msg = result.unwrap();
        assert_eq!(msg.message_type, RawMessageType::System);
    }

    #[test]
    fn test_parse_array_content_text_block() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Here is the answer"}],"role":"assistant"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should parse array content");

        let msg = result.unwrap();
        match &msg.message.content {
            RawContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 1);
                assert_eq!(blocks[0].block_type, "text");
                assert_eq!(blocks[0].text, Some("Here is the answer".to_string()));
            }
            RawContent::Text(_) => panic!("Expected blocks, got text"),
        }
    }

    #[test]
    fn test_parse_array_content_tool_use() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"read_file","tool_use_id":"toolu_123","input":{"path":"/test.txt"}}],"role":"assistant"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should parse tool_use content");

        let msg = result.unwrap();
        match &msg.message.content {
            RawContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 1);
                assert_eq!(blocks[0].block_type, "tool_use");
                assert_eq!(blocks[0].name, Some("read_file".to_string()));
                assert_eq!(blocks[0].tool_use_id, Some("toolu_123".to_string()));
                assert!(blocks[0].input.is_some());
            }
            RawContent::Text(_) => panic!("Expected blocks, got text"),
        }
    }

    #[test]
    fn test_parse_array_content_tool_result() {
        let line = r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"toolu_123","content":"File contents here"}],"role":"user"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should parse tool_result content");

        let msg = result.unwrap();
        match &msg.message.content {
            RawContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 1);
                assert_eq!(blocks[0].block_type, "tool_result");
                assert_eq!(blocks[0].tool_use_id, Some("toolu_123".to_string()));
            }
            RawContent::Text(_) => panic!("Expected blocks, got text"),
        }
    }

    #[test]
    fn test_parse_mixed_content_blocks() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Let me read that file"},{"type":"tool_use","name":"read_file","tool_use_id":"toolu_456","input":{}}],"role":"assistant"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should parse mixed content blocks");

        let msg = result.unwrap();
        match &msg.message.content {
            RawContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 2);
                assert_eq!(blocks[0].block_type, "text");
                assert_eq!(blocks[1].block_type, "tool_use");
            }
            RawContent::Text(_) => panic!("Expected blocks, got text"),
        }
    }

    #[test]
    fn test_parse_missing_optional_fields() {
        // Only required fields: type and message
        let line = r#"{"type":"user","message":{"content":"Hello"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should parse with missing optional fields");

        let msg = result.unwrap();
        assert!(msg.timestamp.is_none());
        assert!(msg.uuid.is_none());
        assert!(msg.session_id.is_none());
        assert!(msg.token_count.is_none());
        assert!(msg.message.role.is_none());
    }

    #[test]
    fn test_parse_null_content() {
        let line = r#"{"type":"user","message":{"content":null,"role":"user"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should handle null content");

        let msg = result.unwrap();
        match &msg.message.content {
            RawContent::Text(text) => assert!(text.is_empty()),
            RawContent::Blocks(_) => panic!("Expected empty text, got blocks"),
        }
    }

    #[test]
    fn test_parse_missing_content() {
        let line = r#"{"type":"user","message":{"role":"user"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should handle missing content");

        let msg = result.unwrap();
        match &msg.message.content {
            RawContent::Text(text) => assert!(text.is_empty()),
            RawContent::Blocks(_) => panic!("Expected empty text, got blocks"),
        }
    }

    #[test]
    fn test_parse_empty_line() {
        let result = parse_jsonl_line("");
        assert!(result.is_err(), "Should error on empty line");

        match result.unwrap_err() {
            ParserError::MissingField(field) => assert!(field.contains("empty")),
            _ => panic!("Expected MissingField error"),
        }
    }

    #[test]
    fn test_parse_whitespace_line() {
        let result = parse_jsonl_line("   \t\n  ");
        assert!(result.is_err(), "Should error on whitespace-only line");
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_jsonl_line("{not valid json}");
        assert!(result.is_err(), "Should error on invalid JSON");

        match result.unwrap_err() {
            ParserError::JsonParse(_) => {}
            _ => panic!("Expected JsonParse error"),
        }
    }

    #[test]
    fn test_parse_missing_type_field() {
        let line = r#"{"message":{"content":"Hello"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_err(), "Should error when type is missing");

        match result.unwrap_err() {
            ParserError::MissingField(field) => assert_eq!(field, "type"),
            _ => panic!("Expected MissingField error for type"),
        }
    }

    #[test]
    fn test_parse_missing_message_field() {
        let line = r#"{"type":"user"}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_err(), "Should error when message is missing");

        match result.unwrap_err() {
            ParserError::MissingField(field) => assert_eq!(field, "message"),
            _ => panic!("Expected MissingField error for message"),
        }
    }

    #[test]
    fn test_parse_invalid_type_value() {
        let line = r#"{"type":"unknown","message":{"content":"Hello"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_err(), "Should error on invalid type value");

        match result.unwrap_err() {
            ParserError::InvalidField { field, reason } => {
                assert_eq!(field, "type");
                assert!(reason.contains("unknown"));
            }
            _ => panic!("Expected InvalidField error"),
        }
    }

    #[test]
    fn test_parse_type_not_string() {
        let line = r#"{"type":123,"message":{"content":"Hello"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_err(), "Should error when type is not string");

        match result.unwrap_err() {
            ParserError::InvalidField { field, .. } => assert_eq!(field, "type"),
            _ => panic!("Expected InvalidField error"),
        }
    }

    #[test]
    fn test_parse_invalid_content_type() {
        let line = r#"{"type":"user","message":{"content":123}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_err(), "Should error when content is invalid type");

        match result.unwrap_err() {
            ParserError::InvalidField { field, .. } => assert_eq!(field, "message.content"),
            _ => panic!("Expected InvalidField error"),
        }
    }

    #[test]
    fn test_parse_preserves_whitespace_in_content() {
        let line = r#"{"type":"user","message":{"content":"  Hello\n  World  "}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok());

        let msg = result.unwrap();
        match &msg.message.content {
            RawContent::Text(text) => assert_eq!(text, "  Hello\n  World  "),
            RawContent::Blocks(_) => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_parse_unicode_content() {
        let line = r#"{"type":"user","message":{"content":"你好世界 🌍 émojis"}}"#;

        let result = parse_jsonl_line(line);
        assert!(result.is_ok(), "Should handle unicode content");

        let msg = result.unwrap();
        match &msg.message.content {
            RawContent::Text(text) => {
                assert!(text.contains("你好"));
                assert!(text.contains("🌍"));
            }
            RawContent::Blocks(_) => panic!("Expected text content"),
        }
    }

    // ========== parse_conversation_file tests ==========

    #[test]
    fn test_parse_conversation_file_single_session() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("project-hash").join("session.jsonl");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();

        let content = r#"{"type":"user","message":{"content":"Hello"},"timestamp":"2025-01-15T10:00:00Z","sessionId":"session-1"}
{"type":"assistant","message":{"content":"Hi there!"},"timestamp":"2025-01-15T10:00:05Z","sessionId":"session-1","tokenCount":{"input":5,"output":10}}"#;

        File::create(&file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let result = parse_conversation_file(&file_path);
        assert!(result.is_ok(), "Should parse conversation file");

        let conversations = result.unwrap();
        assert_eq!(conversations.len(), 1, "Should have 1 conversation");

        let conv = &conversations[0];
        assert_eq!(conv.session_id, "session-1");
        assert_eq!(conv.messages.len(), 2);
        assert_eq!(conv.start_time, "2025-01-15T10:00:00Z");
        assert_eq!(conv.last_time, "2025-01-15T10:00:05Z");
        assert_eq!(conv.total_input_tokens, 5);
        assert_eq!(conv.total_output_tokens, 10);
    }

    #[test]
    fn test_parse_conversation_file_multiple_sessions() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("project-hash").join("multi.jsonl");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();

        let content = r#"{"type":"user","message":{"content":"First session"},"timestamp":"2025-01-15T09:00:00Z","sessionId":"session-A"}
{"type":"user","message":{"content":"Second session"},"timestamp":"2025-01-15T10:00:00Z","sessionId":"session-B"}
{"type":"assistant","message":{"content":"Reply A"},"timestamp":"2025-01-15T09:00:05Z","sessionId":"session-A"}
{"type":"assistant","message":{"content":"Reply B"},"timestamp":"2025-01-15T10:00:05Z","sessionId":"session-B"}"#;

        File::create(&file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let result = parse_conversation_file(&file_path);
        assert!(result.is_ok());

        let conversations = result.unwrap();
        assert_eq!(conversations.len(), 2, "Should have 2 conversations");

        // Should be sorted by start_time (newest first)
        assert_eq!(conversations[0].session_id, "session-B");
        assert_eq!(conversations[1].session_id, "session-A");

        // Each conversation should have 2 messages
        assert_eq!(conversations[0].messages.len(), 2);
        assert_eq!(conversations[1].messages.len(), 2);
    }

    #[test]
    fn test_parse_conversation_file_empty_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("empty.jsonl");

        File::create(&file_path).unwrap();

        let result = parse_conversation_file(&file_path);
        assert!(result.is_ok(), "Should handle empty file gracefully");

        let conversations = result.unwrap();
        assert!(conversations.is_empty(), "Empty file should return no conversations");
    }

    #[test]
    fn test_parse_conversation_file_with_empty_lines() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("with-blanks.jsonl");

        let content = r#"{"type":"user","message":{"content":"Hello"},"sessionId":"s1"}

{"type":"assistant","message":{"content":"Hi"},"sessionId":"s1"}

"#;

        File::create(&file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let result = parse_conversation_file(&file_path);
        assert!(result.is_ok(), "Should skip empty lines");

        let conversations = result.unwrap();
        assert_eq!(conversations.len(), 1);
        assert_eq!(conversations[0].messages.len(), 2);
    }

    #[test]
    fn test_parse_conversation_file_with_malformed_lines() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("malformed.jsonl");

        let content = r#"{"type":"user","message":{"content":"Valid"},"sessionId":"s1"}
{invalid json here}
{"type":"assistant","message":{"content":"Also valid"},"sessionId":"s1"}"#;

        File::create(&file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let result = parse_conversation_file(&file_path);
        assert!(result.is_ok(), "Should skip malformed lines");

        let conversations = result.unwrap();
        assert_eq!(conversations.len(), 1);
        assert_eq!(
            conversations[0].messages.len(),
            2,
            "Should have 2 valid messages"
        );
    }

    #[test]
    fn test_parse_conversation_file_no_session_id() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("no-session.jsonl");

        let content = r#"{"type":"user","message":{"content":"No session ID"}}
{"type":"assistant","message":{"content":"Reply"}}"#;

        File::create(&file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let result = parse_conversation_file(&file_path);
        assert!(result.is_ok());

        let conversations = result.unwrap();
        assert_eq!(conversations.len(), 1);
        assert_eq!(
            conversations[0].session_id, "default",
            "Should use 'default' session ID"
        );
    }

    #[test]
    fn test_parse_conversation_file_token_aggregation() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("tokens.jsonl");

        let content = r#"{"type":"user","message":{"content":"Q1"},"sessionId":"s1","tokenCount":{"input":10,"output":0}}
{"type":"assistant","message":{"content":"A1"},"sessionId":"s1","tokenCount":{"input":0,"output":20}}
{"type":"user","message":{"content":"Q2"},"sessionId":"s1","tokenCount":{"input":15,"output":0}}
{"type":"assistant","message":{"content":"A2"},"sessionId":"s1","tokenCount":{"input":0,"output":30}}"#;

        File::create(&file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let result = parse_conversation_file(&file_path);
        assert!(result.is_ok());

        let conversations = result.unwrap();
        assert_eq!(conversations[0].total_input_tokens, 25);
        assert_eq!(conversations[0].total_output_tokens, 50);
    }

    #[test]
    fn test_parse_conversation_file_messages_sorted_by_timestamp() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("unsorted.jsonl");

        // Messages in file are out of order
        let content = r#"{"type":"assistant","message":{"content":"Third"},"timestamp":"2025-01-15T10:02:00Z","sessionId":"s1"}
{"type":"user","message":{"content":"First"},"timestamp":"2025-01-15T10:00:00Z","sessionId":"s1"}
{"type":"user","message":{"content":"Second"},"timestamp":"2025-01-15T10:01:00Z","sessionId":"s1"}"#;

        File::create(&file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let result = parse_conversation_file(&file_path);
        assert!(result.is_ok());

        let conversations = result.unwrap();
        let msgs = &conversations[0].messages;

        // Should be sorted chronologically
        assert_eq!(msgs[0].timestamp, Some("2025-01-15T10:00:00Z".to_string()));
        assert_eq!(msgs[1].timestamp, Some("2025-01-15T10:01:00Z".to_string()));
        assert_eq!(msgs[2].timestamp, Some("2025-01-15T10:02:00Z".to_string()));
    }

    #[test]
    fn test_extract_project_info() {
        let path = Path::new("/Users/test/.claude/projects/abc123-hash/session.jsonl");
        let (project_path, project_name) = extract_project_info(path);

        assert_eq!(
            project_path,
            "/Users/test/.claude/projects/abc123-hash"
        );
        assert_eq!(project_name, "abc123-hash");
    }

    #[test]
    fn test_generate_conversation_id_deterministic() {
        let path = Path::new("/test/path/session.jsonl");
        let session_id = "session-123";

        let id1 = generate_conversation_id(path, session_id);
        let id2 = generate_conversation_id(path, session_id);

        assert_eq!(id1, id2, "Same inputs should produce same ID");
        assert_eq!(id1.len(), 12, "ID should be 12 characters");
    }

    #[test]
    fn test_generate_conversation_id_unique() {
        let path1 = Path::new("/path/a/session.jsonl");
        let path2 = Path::new("/path/b/session.jsonl");
        let session_id = "session-123";

        let id1 = generate_conversation_id(path1, session_id);
        let id2 = generate_conversation_id(path2, session_id);

        assert_ne!(id1, id2, "Different paths should produce different IDs");
    }

    #[test]
    fn test_calculate_total_tokens() {
        let messages = vec![
            RawMessage {
                message_type: RawMessageType::User,
                message: RawInnerMessage {
                    content: RawContent::Text("test".to_string()),
                    role: Some("user".to_string()),
                },
                timestamp: None,
                token_count: Some(RawTokenCount {
                    input: 10,
                    output: 0,
                }),
                uuid: None,
                session_id: None,
            },
            RawMessage {
                message_type: RawMessageType::Assistant,
                message: RawInnerMessage {
                    content: RawContent::Text("reply".to_string()),
                    role: Some("assistant".to_string()),
                },
                timestamp: None,
                token_count: Some(RawTokenCount {
                    input: 0,
                    output: 25,
                }),
                uuid: None,
                session_id: None,
            },
            RawMessage {
                message_type: RawMessageType::User,
                message: RawInnerMessage {
                    content: RawContent::Text("no tokens".to_string()),
                    role: None,
                },
                timestamp: None,
                token_count: None, // No token count
                uuid: None,
                session_id: None,
            },
        ];

        let (input, output) = calculate_total_tokens(&messages);
        assert_eq!(input, 10);
        assert_eq!(output, 25);
    }

    #[test]
    fn test_parse_conversation_file_nonexistent() {
        let result = parse_conversation_file(Path::new("/nonexistent/file.jsonl"));
        assert!(result.is_err(), "Should error for nonexistent file");

        match result.unwrap_err() {
            ParserError::Io(_) => {}
            other => panic!("Expected Io error, got {:?}", other),
        }
    }
}
