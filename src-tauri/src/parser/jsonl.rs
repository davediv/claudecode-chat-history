//! JSONL file discovery and parsing.
//!
//! This module handles finding and reading Claude Code JSONL conversation files
//! from the `~/.claude/projects/` directory.

use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use tracing::{debug, warn};

/// Parser-related errors.
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to get home directory")]
    HomeNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for parser operations.
pub type ParserResult<T> = Result<T, ParserError>;

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
}
