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
}
