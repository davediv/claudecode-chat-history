//! Content block parsing and extraction.
//!
//! This module handles parsing raw message content into structured ContentBlocks.
//! It extracts code blocks from markdown fences, handles tool_use/tool_result blocks,
//! and preserves the order of all content.

use crate::models::{ContentBlock, ContentBlockType};
use crate::parser::jsonl::{RawContent, RawContentBlock};
use regex::Regex;
use std::sync::LazyLock;

/// Regex for matching markdown code fences.
/// Matches: ```language\ncode\n``` or ```\ncode\n```
static CODE_FENCE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"```(\w*)\n([\s\S]*?)```").expect("Invalid regex pattern")
});

/// System metadata tag names that should be filtered from display.
/// These are internal Claude Code system messages that shouldn't appear as user chat bubbles.
pub const SYSTEM_METADATA_TAGS: &[&str] = &[
    "command-name",
    "command-message",
    "command-args",
    "local-command-caveat",
    "local-command-stdout",
    "system-reminder",
];

/// Regex for detecting if content contains system metadata XML tags.
static SYSTEM_METADATA_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<(command-name|command-message|command-args|local-command-caveat|local-command-stdout|system-reminder)[^>]*>")
        .expect("Invalid system metadata regex")
});

/// Pre-compiled regexes for stripping each system metadata tag type.
/// Using LazyLock for efficient one-time compilation.
static SYSTEM_METADATA_STRIP_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    SYSTEM_METADATA_TAGS
        .iter()
        .map(|tag| {
            let pattern = format!(r"<{}[^>]*>[\s\S]*?</{}>", tag, tag);
            Regex::new(&pattern).expect("Invalid metadata tag strip regex")
        })
        .collect()
});

/// Checks if content consists only of system metadata XML tags.
/// Returns true if the content should be filtered from display.
///
/// This filters out messages like:
/// - `<command-name>/clear</command-name>`
/// - `<local-command-caveat>...</local-command-caveat>`
/// - `<local-command-stdout></local-command-stdout>`
///
/// These appear when users run CLI commands like `/clear` and shouldn't
/// be shown as separate chat bubbles in the UI.
pub fn is_system_metadata_content(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || !trimmed.starts_with('<') {
        return false;
    }

    // Check if ANY known metadata tag exists
    if !SYSTEM_METADATA_REGEX.is_match(trimmed) {
        return false;
    }

    // Strip all known metadata tags and check if anything meaningful remains
    let remaining = strip_system_metadata_tags(trimmed);

    // If only whitespace remains, this was a metadata-only message
    remaining.is_empty()
}

/// Strips system metadata XML tags from text for display purposes.
///
/// Removes tags like `<command-name>`, `<command-message>`, `<system-reminder>`, etc.
/// while preserving any other content. Uses pre-compiled regexes for efficiency.
///
/// # Examples
/// ```ignore
/// let text = "Hello <command-name>/clear</command-name> World";
/// let result = strip_system_metadata_tags(text);
/// assert_eq!(result, "Hello  World");
/// ```
pub fn strip_system_metadata_tags(text: &str) -> String {
    let mut result = text.to_string();

    for re in SYSTEM_METADATA_STRIP_REGEXES.iter() {
        result = re.replace_all(&result, "").to_string();
    }

    result.trim().to_string()
}

/// Parses raw content into a vector of ContentBlocks.
///
/// Handles three content formats:
/// 1. Plain text - extracts markdown code blocks, returns text and code blocks
/// 2. Array of blocks - converts tool_use, tool_result, and text blocks
/// 3. Empty content - returns empty vector
///
/// # Arguments
/// * `raw_content` - The raw content from a parsed JSONL message
///
/// # Returns
/// * `Vec<ContentBlock>` - Ordered list of content blocks
///
/// # Example
/// ```ignore
/// let raw = RawContent::Text("Here's some code:\n```rust\nfn main() {}\n```".to_string());
/// let blocks = parse_content_blocks(&raw);
/// assert_eq!(blocks.len(), 2); // text block + code block
/// ```
pub fn parse_content_blocks(raw_content: &RawContent) -> Vec<ContentBlock> {
    match raw_content {
        RawContent::Text(text) => parse_text_content(text),
        RawContent::Blocks(blocks) => parse_block_array(blocks),
    }
}

/// Parses plain text content, extracting markdown code fences.
///
/// Text content may contain markdown code blocks like:
/// ```language
/// code here
/// ```
///
/// These are extracted as separate Code blocks, while surrounding
/// text becomes Text blocks.
fn parse_text_content(text: &str) -> Vec<ContentBlock> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut blocks = Vec::new();
    let mut last_end = 0;

    for cap in CODE_FENCE_REGEX.captures_iter(text) {
        let full_match = cap.get(0).unwrap();
        let start = full_match.start();
        let end = full_match.end();

        // Add text before this code block (if any)
        if start > last_end {
            let preceding_text = text[last_end..start].trim();
            if !preceding_text.is_empty() {
                blocks.push(ContentBlock {
                    block_type: ContentBlockType::Text,
                    content: preceding_text.to_string(),
                    language: None,
                    tool_name: None,
                });
            }
        }

        // Extract language and code
        let language = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let code = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        // Determine language (default to "text" if not specified)
        let lang = if language.is_empty() {
            "text".to_string()
        } else {
            language.to_string()
        };

        blocks.push(ContentBlock {
            block_type: ContentBlockType::Code,
            content: code.trim_end().to_string(),
            language: Some(lang),
            tool_name: None,
        });

        last_end = end;
    }

    // Add any remaining text after the last code block
    if last_end < text.len() {
        let remaining_text = text[last_end..].trim();
        if !remaining_text.is_empty() {
            blocks.push(ContentBlock {
                block_type: ContentBlockType::Text,
                content: remaining_text.to_string(),
                language: None,
                tool_name: None,
            });
        }
    }

    // If no code blocks were found, return the entire text as a single block
    if blocks.is_empty() && !text.trim().is_empty() {
        blocks.push(ContentBlock {
            block_type: ContentBlockType::Text,
            content: text.trim().to_string(),
            language: None,
            tool_name: None,
        });
    }

    blocks
}

/// Parses an array of raw content blocks into ContentBlocks.
///
/// Handles these block types:
/// - "text": Extracts text content, also scans for embedded code fences
/// - "tool_use": Extracts tool name and serializes input as content
/// - "tool_result": Extracts tool_use_id and result content
fn parse_block_array(raw_blocks: &[RawContentBlock]) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();

    for raw in raw_blocks {
        match raw.block_type.as_str() {
            "text" => {
                if let Some(text) = &raw.text {
                    // Text blocks may contain embedded code fences
                    let parsed = parse_text_content(text);
                    blocks.extend(parsed);
                }
            }
            "tool_use" => {
                let tool_name = raw.name.clone();
                let content = raw
                    .input
                    .as_ref()
                    .map(|v| serde_json::to_string_pretty(v).unwrap_or_default())
                    .unwrap_or_default();

                blocks.push(ContentBlock {
                    block_type: ContentBlockType::ToolUse,
                    content,
                    language: None,
                    tool_name,
                });
            }
            "tool_result" => {
                // tool_result content can be a string or a more complex structure
                let content = match &raw.content {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(v) => serde_json::to_string_pretty(v).unwrap_or_default(),
                    None => String::new(),
                };

                // Use tool_use_id as a pseudo tool name for reference
                let tool_name = raw.tool_use_id.clone();

                blocks.push(ContentBlock {
                    block_type: ContentBlockType::ToolResult,
                    content,
                    language: None,
                    tool_name,
                });
            }
            _ => {
                // Unknown block types are treated as text
                if let Some(text) = &raw.text {
                    blocks.push(ContentBlock {
                        block_type: ContentBlockType::Text,
                        content: text.clone(),
                        language: None,
                        tool_name: None,
                    });
                }
            }
        }
    }

    blocks
}

/// Extracts the first user message preview from content blocks.
///
/// Returns the first 100 characters of the first text block,
/// useful for conversation list previews.
pub fn extract_preview(blocks: &[ContentBlock]) -> String {
    for block in blocks {
        if block.block_type == ContentBlockType::Text && !block.content.is_empty() {
            let content = &block.content;
            if content.len() <= 100 {
                return content.clone();
            }
            // Truncate at word boundary if possible
            let truncated = &content[..100];
            if let Some(last_space) = truncated.rfind(' ') {
                return format!("{}...", &truncated[..last_space]);
            }
            return format!("{}...", truncated);
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ========== parse_text_content tests ==========

    #[test]
    fn test_parse_plain_text() {
        let text = "Hello, this is plain text without any code.";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
        assert_eq!(blocks[0].content, text);
    }

    #[test]
    fn test_parse_single_code_block() {
        let text = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Code);
        assert_eq!(blocks[0].language, Some("rust".to_string()));
        assert!(blocks[0].content.contains("fn main()"));
    }

    #[test]
    fn test_parse_code_block_no_language() {
        let text = "```\nsome code\n```";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Code);
        assert_eq!(blocks[0].language, Some("text".to_string()));
        assert_eq!(blocks[0].content, "some code");
    }

    #[test]
    fn test_parse_text_with_code_block() {
        let text = "Here's some code:\n```python\nprint('hello')\n```\nThat's it!";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
        assert!(blocks[0].content.contains("Here's some code"));
        assert_eq!(blocks[1].block_type, ContentBlockType::Code);
        assert_eq!(blocks[1].language, Some("python".to_string()));
        assert_eq!(blocks[2].block_type, ContentBlockType::Text);
        assert!(blocks[2].content.contains("That's it"));
    }

    #[test]
    fn test_parse_multiple_code_blocks() {
        let text = "```js\nconst a = 1;\n```\n\n```ts\nconst b: number = 2;\n```";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].language, Some("js".to_string()));
        assert_eq!(blocks[1].language, Some("ts".to_string()));
    }

    #[test]
    fn test_parse_empty_text() {
        let blocks = parse_text_content("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let blocks = parse_text_content("   \n\t  ");
        assert!(blocks.is_empty());
    }

    // ========== parse_block_array tests ==========

    #[test]
    fn test_parse_text_block() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "text".to_string(),
            text: Some("Hello world".to_string()),
            name: None,
            input: None,
            tool_use_id: None,
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
        assert_eq!(blocks[0].content, "Hello world");
    }

    #[test]
    fn test_parse_tool_use_block() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "tool_use".to_string(),
            text: None,
            name: Some("read_file".to_string()),
            input: Some(json!({"path": "/test.txt"})),
            tool_use_id: Some("toolu_123".to_string()),
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::ToolUse);
        assert_eq!(blocks[0].tool_name, Some("read_file".to_string()));
        assert!(blocks[0].content.contains("path"));
    }

    #[test]
    fn test_parse_tool_result_block() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "tool_result".to_string(),
            text: None,
            name: None,
            input: None,
            tool_use_id: Some("toolu_123".to_string()),
            content: Some(json!("File contents here")),
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::ToolResult);
        assert_eq!(blocks[0].tool_name, Some("toolu_123".to_string()));
        assert_eq!(blocks[0].content, "File contents here");
    }

    #[test]
    fn test_parse_mixed_blocks() {
        let raw_blocks = vec![
            RawContentBlock {
                block_type: "text".to_string(),
                text: Some("Let me read that file".to_string()),
                name: None,
                input: None,
                tool_use_id: None,
                content: None,
            },
            RawContentBlock {
                block_type: "tool_use".to_string(),
                text: None,
                name: Some("read_file".to_string()),
                input: Some(json!({"path": "/test.txt"})),
                tool_use_id: Some("toolu_456".to_string()),
                content: None,
            },
        ];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
        assert_eq!(blocks[1].block_type, ContentBlockType::ToolUse);
    }

    #[test]
    fn test_parse_text_block_with_code_fence() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "text".to_string(),
            text: Some("Here's code:\n```rust\nfn test() {}\n```".to_string()),
            name: None,
            input: None,
            tool_use_id: None,
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
        assert_eq!(blocks[1].block_type, ContentBlockType::Code);
        assert_eq!(blocks[1].language, Some("rust".to_string()));
    }

    // ========== parse_content_blocks tests ==========

    #[test]
    fn test_parse_content_blocks_text() {
        let raw = RawContent::Text("Simple text".to_string());
        let blocks = parse_content_blocks(&raw);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
    }

    #[test]
    fn test_parse_content_blocks_array() {
        let raw = RawContent::Blocks(vec![RawContentBlock {
            block_type: "text".to_string(),
            text: Some("From array".to_string()),
            name: None,
            input: None,
            tool_use_id: None,
            content: None,
        }]);

        let blocks = parse_content_blocks(&raw);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, "From array");
    }

    // ========== extract_preview tests ==========

    #[test]
    fn test_extract_preview_short() {
        let blocks = vec![ContentBlock {
            block_type: ContentBlockType::Text,
            content: "Short preview".to_string(),
            language: None,
            tool_name: None,
        }];

        let preview = extract_preview(&blocks);
        assert_eq!(preview, "Short preview");
    }

    #[test]
    fn test_extract_preview_long() {
        let long_text = "a ".repeat(60); // 120 characters
        let blocks = vec![ContentBlock {
            block_type: ContentBlockType::Text,
            content: long_text,
            language: None,
            tool_name: None,
        }];

        let preview = extract_preview(&blocks);
        assert!(preview.len() <= 103); // 100 + "..."
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn test_extract_preview_skips_code() {
        let blocks = vec![
            ContentBlock {
                block_type: ContentBlockType::Code,
                content: "fn main() {}".to_string(),
                language: Some("rust".to_string()),
                tool_name: None,
            },
            ContentBlock {
                block_type: ContentBlockType::Text,
                content: "This is the text".to_string(),
                language: None,
                tool_name: None,
            },
        ];

        let preview = extract_preview(&blocks);
        assert_eq!(preview, "This is the text");
    }

    #[test]
    fn test_extract_preview_empty() {
        let blocks: Vec<ContentBlock> = vec![];
        let preview = extract_preview(&blocks);
        assert!(preview.is_empty());
    }

    // ========== Additional edge case tests for content parsing ==========

    #[test]
    fn test_parse_code_block_at_start() {
        let text = "```python\nprint('hello')\n```\nSome text after";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, ContentBlockType::Code);
        assert_eq!(blocks[0].language, Some("python".to_string()));
        assert_eq!(blocks[1].block_type, ContentBlockType::Text);
        assert!(blocks[1].content.contains("Some text after"));
    }

    #[test]
    fn test_parse_code_block_at_end() {
        let text = "Some text before\n```rust\nfn test() {}\n```";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
        assert!(blocks[0].content.contains("Some text before"));
        assert_eq!(blocks[1].block_type, ContentBlockType::Code);
        assert_eq!(blocks[1].language, Some("rust".to_string()));
    }

    #[test]
    fn test_parse_unclosed_code_fence() {
        // Unclosed code fence should be treated as regular text
        let text = "Here's some code:\n```rust\nfn main() {\n    println!(\"Hello\");";
        let blocks = parse_text_content(text);

        // Since the code fence is not closed, it's all text
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
        assert!(blocks[0].content.contains("```rust"));
    }

    #[test]
    fn test_parse_code_fence_with_special_language() {
        // The regex pattern is ```(\w*)\n which requires \n immediately after \w*
        // For "c++", after "c" comes "++" which is not \n, so the fence doesn't match
        // This is a known limitation - languages with special chars don't parse as code blocks
        let text = "```c++\nint main() { return 0; }\n```";
        let blocks = parse_text_content(text);

        // Entire thing becomes text because the regex doesn't match
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
        assert!(blocks[0].content.contains("```c++"));

        // However, standard language names work fine
        let text_cpp = "```cpp\nint main() { return 0; }\n```";
        let blocks_cpp = parse_text_content(text_cpp);
        assert_eq!(blocks_cpp.len(), 1);
        assert_eq!(blocks_cpp[0].block_type, ContentBlockType::Code);
        assert_eq!(blocks_cpp[0].language, Some("cpp".to_string()));
    }

    #[test]
    fn test_parse_code_fence_with_empty_code() {
        let text = "```rust\n```";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Code);
        assert_eq!(blocks[0].language, Some("rust".to_string()));
        assert!(blocks[0].content.is_empty());
    }

    #[test]
    fn test_parse_adjacent_code_blocks() {
        let text = "```js\nconst a = 1;\n```\n```ts\nconst b: number = 2;\n```";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].language, Some("js".to_string()));
        assert_eq!(blocks[1].language, Some("ts".to_string()));
    }

    #[test]
    fn test_parse_code_block_with_backticks_in_code() {
        // Code block containing backtick characters in the code
        let text = "```bash\necho `date`\n```";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Code);
        assert!(blocks[0].content.contains("`date`"));
    }

    #[test]
    fn test_parse_code_block_with_triple_backticks_in_code() {
        // This is a tricky case - code containing ```
        // The regex will match the first ``` it finds
        let text = "```md\nHere's how to write a code block:\n\\`\\`\\`rust\ncode here\n\\`\\`\\`\n```";
        let blocks = parse_text_content(text);

        // Should treat escaped backticks as content
        assert!(blocks.len() >= 1);
    }

    #[test]
    fn test_parse_text_with_only_whitespace_between_blocks() {
        let text = "```js\na\n```\n   \n\t\n```py\nb\n```";
        let blocks = parse_text_content(text);

        // Whitespace-only text between blocks should be omitted
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].language, Some("js".to_string()));
        assert_eq!(blocks[1].language, Some("py".to_string()));
    }

    #[test]
    fn test_parse_code_block_preserves_internal_whitespace() {
        let text = "```python\n    def foo():\n        pass\n```";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].content.contains("    def foo()"));
        assert!(blocks[0].content.contains("        pass"));
    }

    #[test]
    fn test_tool_use_with_empty_input() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "tool_use".to_string(),
            text: None,
            name: Some("empty_tool".to_string()),
            input: Some(json!({})),
            tool_use_id: Some("toolu_empty".to_string()),
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::ToolUse);
        assert_eq!(blocks[0].content, "{}");
    }

    #[test]
    fn test_tool_use_with_no_input() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "tool_use".to_string(),
            text: None,
            name: Some("no_input_tool".to_string()),
            input: None,
            tool_use_id: Some("toolu_noinput".to_string()),
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::ToolUse);
        assert!(blocks[0].content.is_empty());
    }

    #[test]
    fn test_tool_result_with_empty_content() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "tool_result".to_string(),
            text: None,
            name: None,
            input: None,
            tool_use_id: Some("toolu_result".to_string()),
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::ToolResult);
        assert!(blocks[0].content.is_empty());
    }

    #[test]
    fn test_tool_result_with_array_content() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "tool_result".to_string(),
            text: None,
            name: None,
            input: None,
            tool_use_id: Some("toolu_arr".to_string()),
            content: Some(json!(["item1", "item2", "item3"])),
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::ToolResult);
        assert!(blocks[0].content.contains("item1"));
        assert!(blocks[0].content.contains("item2"));
    }

    #[test]
    fn test_unknown_block_type_with_text() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "unknown_type".to_string(),
            text: Some("Fallback text".to_string()),
            name: None,
            input: None,
            tool_use_id: None,
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Text);
        assert_eq!(blocks[0].content, "Fallback text");
    }

    #[test]
    fn test_unknown_block_type_without_text() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "unknown_type".to_string(),
            text: None,
            name: None,
            input: None,
            tool_use_id: None,
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert!(blocks.is_empty(), "Unknown type without text should produce no block");
    }

    #[test]
    fn test_text_block_with_empty_text() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "text".to_string(),
            text: Some(String::new()),
            name: None,
            input: None,
            tool_use_id: None,
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert!(blocks.is_empty(), "Empty text block should produce no output");
    }

    #[test]
    fn test_extract_preview_truncates_at_word_boundary() {
        // Create text that's just over 100 chars with a word boundary
        let text = "This is a test sentence that is designed to be longer than one hundred characters so we can test truncation at word boundaries properly.";
        let blocks = vec![ContentBlock {
            block_type: ContentBlockType::Text,
            content: text.to_string(),
            language: None,
            tool_name: None,
        }];

        let preview = extract_preview(&blocks);
        assert!(preview.len() <= 103); // 100 + "..."
        assert!(preview.ends_with("..."));
        // The preview should end at a space boundary within first 100 chars
        let without_ellipsis = preview.trim_end_matches("...");
        // Verify the truncation happened and the result is reasonable
        assert!(without_ellipsis.len() <= 100);
        assert!(without_ellipsis.len() > 50); // Should have substantial content
    }

    #[test]
    fn test_extract_preview_no_word_boundary() {
        // 100+ character string with no spaces
        let text = "a".repeat(150);
        let blocks = vec![ContentBlock {
            block_type: ContentBlockType::Text,
            content: text,
            language: None,
            tool_name: None,
        }];

        let preview = extract_preview(&blocks);
        assert_eq!(preview.len(), 103); // 100 + "..."
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn test_extract_preview_exactly_100_chars() {
        let text = "a".repeat(100);
        let blocks = vec![ContentBlock {
            block_type: ContentBlockType::Text,
            content: text.clone(),
            language: None,
            tool_name: None,
        }];

        let preview = extract_preview(&blocks);
        assert_eq!(preview, text); // No truncation needed
    }

    #[test]
    fn test_extract_preview_only_code_blocks() {
        let blocks = vec![
            ContentBlock {
                block_type: ContentBlockType::Code,
                content: "fn main() {}".to_string(),
                language: Some("rust".to_string()),
                tool_name: None,
            },
            ContentBlock {
                block_type: ContentBlockType::ToolUse,
                content: r#"{"path": "/test"}"#.to_string(),
                language: None,
                tool_name: Some("read_file".to_string()),
            },
        ];

        let preview = extract_preview(&blocks);
        assert!(preview.is_empty(), "Preview should be empty when no text blocks");
    }

    #[test]
    fn test_parse_many_code_blocks() {
        // Stress test with many code blocks
        let mut text = String::new();
        for i in 0..20 {
            text.push_str(&format!("Text {}\n```lang{}\ncode {}\n```\n", i, i, i));
        }

        let blocks = parse_text_content(&text);

        // Should have 20 text blocks and 20 code blocks = 40 total
        assert_eq!(blocks.len(), 40);

        // Verify alternating pattern
        for (i, block) in blocks.iter().enumerate() {
            if i % 2 == 0 {
                assert_eq!(block.block_type, ContentBlockType::Text);
            } else {
                assert_eq!(block.block_type, ContentBlockType::Code);
            }
        }
    }

    #[test]
    fn test_parse_unicode_in_code_block() {
        let text = "```python\n# Unicode: 你好世界 🚀\nprint('こんにちは')\n```";
        let blocks = parse_text_content(text);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::Code);
        assert!(blocks[0].content.contains("你好世界"));
        assert!(blocks[0].content.contains("🚀"));
        assert!(blocks[0].content.contains("こんにちは"));
    }

    #[test]
    fn test_parse_tool_use_with_complex_input() {
        let raw_blocks = vec![RawContentBlock {
            block_type: "tool_use".to_string(),
            text: None,
            name: Some("complex_tool".to_string()),
            input: Some(json!({
                "string": "value",
                "number": 42,
                "boolean": true,
                "null": null,
                "array": [1, 2, 3],
                "nested": {
                    "deep": {
                        "value": "found"
                    }
                }
            })),
            tool_use_id: Some("toolu_complex".to_string()),
            content: None,
        }];

        let blocks = parse_block_array(&raw_blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ContentBlockType::ToolUse);
        // JSON should be pretty-printed
        assert!(blocks[0].content.contains("\"string\": \"value\""));
        assert!(blocks[0].content.contains("\"number\": 42"));
        assert!(blocks[0].content.contains("\"deep\""));
    }

    // ========== is_system_metadata_content tests ==========

    #[test]
    fn test_is_system_metadata_command_name() {
        assert!(is_system_metadata_content(
            "<command-name>/clear</command-name>"
        ));
    }

    #[test]
    fn test_is_system_metadata_multiple_tags() {
        let content = "<command-name>/clear</command-name>\n<command-message>clear</command-message>\n<command-args></command-args>";
        assert!(is_system_metadata_content(content));
    }

    #[test]
    fn test_is_system_metadata_local_command() {
        assert!(is_system_metadata_content(
            "<local-command-caveat>Some caveat text</local-command-caveat>"
        ));
    }

    #[test]
    fn test_is_system_metadata_stdout_empty() {
        assert!(is_system_metadata_content(
            "<local-command-stdout></local-command-stdout>"
        ));
    }

    #[test]
    fn test_is_system_metadata_system_reminder() {
        assert!(is_system_metadata_content(
            "<system-reminder>This is a system reminder</system-reminder>"
        ));
    }

    #[test]
    fn test_is_not_system_metadata_plain_text() {
        assert!(!is_system_metadata_content("Hello, how are you?"));
    }

    #[test]
    fn test_is_not_system_metadata_mixed_content() {
        assert!(!is_system_metadata_content(
            "User text <command-name>/clear</command-name> more text"
        ));
    }

    #[test]
    fn test_is_not_system_metadata_empty() {
        assert!(!is_system_metadata_content(""));
    }

    #[test]
    fn test_is_not_system_metadata_whitespace() {
        assert!(!is_system_metadata_content("   \n\t  "));
    }

    #[test]
    fn test_is_not_system_metadata_unknown_xml() {
        assert!(!is_system_metadata_content("<unknown-tag>content</unknown-tag>"));
    }

    #[test]
    fn test_is_system_metadata_with_whitespace() {
        assert!(is_system_metadata_content(
            "  <command-name>/clear</command-name>  "
        ));
    }

    #[test]
    fn test_is_system_metadata_nested_tags() {
        let content = "<command-name>/help</command-name>\n<local-command-caveat>This is a local command</local-command-caveat>\n<local-command-stdout>Output here</local-command-stdout>";
        assert!(is_system_metadata_content(content));
    }

    // ========== strip_system_metadata_tags tests ==========

    #[test]
    fn test_strip_system_metadata_tags_command_name() {
        let text = "<command-name>/clear</command-name>";
        let result = strip_system_metadata_tags(text);
        assert!(result.is_empty());
    }

    #[test]
    fn test_strip_system_metadata_tags_multiple() {
        let text = "<command-name>/help</command-name>\n<command-message>help</command-message>";
        let result = strip_system_metadata_tags(text);
        assert!(result.is_empty());
    }

    #[test]
    fn test_strip_system_metadata_tags_mixed_content() {
        let text = "Hello <command-name>/clear</command-name> World";
        let result = strip_system_metadata_tags(text);
        assert_eq!(result, "Hello  World");
    }

    #[test]
    fn test_strip_system_metadata_tags_plain_text() {
        let text = "Hello, how can I help you?";
        let result = strip_system_metadata_tags(text);
        assert_eq!(result, text);
    }

    #[test]
    fn test_strip_system_metadata_tags_system_reminder() {
        let text = "<system-reminder>This is a reminder</system-reminder>";
        let result = strip_system_metadata_tags(text);
        assert!(result.is_empty());
    }

    #[test]
    fn test_strip_system_metadata_tags_with_attributes() {
        let text = r#"<command-name id="123" class="test">content</command-name>"#;
        let result = strip_system_metadata_tags(text);
        assert!(result.is_empty());
    }

    #[test]
    fn test_strip_system_metadata_tags_preserves_other_xml() {
        let text = "<unknown-tag>should stay</unknown-tag>";
        let result = strip_system_metadata_tags(text);
        assert_eq!(result, text);
    }
}
