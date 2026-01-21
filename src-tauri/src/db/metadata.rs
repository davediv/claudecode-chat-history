//! File metadata tracking for incremental parsing.
//!
//! This module provides functions to track file modification times,
//! enabling efficient incremental parsing that only processes changed files.

use crate::db::sqlite::DbResult;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Metadata about a tracked file.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// Path to the file.
    pub file_path: PathBuf,
    /// Last known modification time (ISO 8601).
    pub modified_at: String,
    /// When we last parsed this file (ISO 8601).
    pub parsed_at: String,
}

/// Information about a file that needs processing.
#[derive(Debug, Clone)]
pub struct ModifiedFile {
    /// Path to the file.
    pub file_path: PathBuf,
    /// Current modification time from filesystem.
    pub current_modified_at: String,
    /// Whether this is a new file (not in metadata) or modified.
    pub is_new: bool,
}

/// Gets all tracked file metadata from the database.
///
/// Returns a map of file path to metadata for quick lookup.
pub fn get_all_file_metadata(conn: &Connection) -> DbResult<HashMap<String, FileMetadata>> {
    let mut stmt = conn.prepare(
        "SELECT file_path, modified_at, parsed_at FROM file_metadata"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(FileMetadata {
            file_path: PathBuf::from(row.get::<_, String>(0)?),
            modified_at: row.get(1)?,
            parsed_at: row.get(2)?,
        })
    })?;

    let mut metadata_map = HashMap::new();
    for row_result in rows {
        let metadata = row_result?;
        let path_str = metadata.file_path.to_string_lossy().to_string();
        metadata_map.insert(path_str, metadata);
    }

    debug!("Loaded {} file metadata entries", metadata_map.len());
    Ok(metadata_map)
}

/// Checks if the metadata table is empty (needs full rescan).
pub fn is_metadata_empty(conn: &Connection) -> DbResult<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM file_metadata",
        [],
        |row| row.get(0),
    )?;
    Ok(count == 0)
}

/// Gets files that have been modified since they were last parsed.
///
/// Compares current filesystem modification times against stored metadata.
/// Returns files that need to be re-parsed.
///
/// # Arguments
/// * `conn` - Database connection
/// * `discovered_files` - List of JSONL files found in the projects directory
///
/// # Returns
/// * `Vec<ModifiedFile>` - Files that need processing (new or modified)
///
/// # Behavior
/// - If metadata table is empty, returns ALL discovered files as "new"
/// - Otherwise, returns only files where modification time has changed
pub fn get_modified_files(
    conn: &Connection,
    discovered_files: &[PathBuf],
) -> DbResult<Vec<ModifiedFile>> {
    // Check if we need a full rescan
    let needs_full_scan = is_metadata_empty(conn)?;

    if needs_full_scan {
        info!("Metadata table empty - performing full scan");
        return Ok(discovered_files
            .iter()
            .filter_map(|path| {
                let modified_at = get_file_modified_time(path)?;
                Some(ModifiedFile {
                    file_path: path.clone(),
                    current_modified_at: modified_at,
                    is_new: true,
                })
            })
            .collect());
    }

    // Load existing metadata
    let metadata_map = get_all_file_metadata(conn)?;

    let mut modified_files = Vec::new();

    for file_path in discovered_files {
        let path_str = file_path.to_string_lossy().to_string();

        // Get current modification time
        let current_modified_at = match get_file_modified_time(file_path) {
            Some(time) => time,
            None => continue, // Skip files we can't read
        };

        match metadata_map.get(&path_str) {
            Some(stored_metadata) => {
                // File exists in metadata - check if modified
                if current_modified_at != stored_metadata.modified_at {
                    debug!(
                        "File modified: {:?} (was: {}, now: {})",
                        file_path, stored_metadata.modified_at, current_modified_at
                    );
                    modified_files.push(ModifiedFile {
                        file_path: file_path.clone(),
                        current_modified_at,
                        is_new: false,
                    });
                }
            }
            None => {
                // New file not in metadata
                debug!("New file discovered: {:?}", file_path);
                modified_files.push(ModifiedFile {
                    file_path: file_path.clone(),
                    current_modified_at,
                    is_new: true,
                });
            }
        }
    }

    info!(
        "Found {} modified/new files out of {} total",
        modified_files.len(),
        discovered_files.len()
    );

    Ok(modified_files)
}

/// Updates the metadata for a single file after successful parsing.
///
/// Records the modification time and current timestamp as parsed time.
pub fn update_file_metadata(
    conn: &Connection,
    file_path: &Path,
    modified_at: &str,
) -> DbResult<()> {
    let now = Utc::now().to_rfc3339();
    let path_str = file_path.to_string_lossy().to_string();

    conn.execute(
        r#"
        INSERT INTO file_metadata (file_path, modified_at, parsed_at)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(file_path) DO UPDATE SET
            modified_at = excluded.modified_at,
            parsed_at = excluded.parsed_at
        "#,
        [&path_str, modified_at, &now],
    )?;

    debug!("Updated metadata for {:?}", file_path);
    Ok(())
}

/// Updates metadata for multiple files in a batch.
///
/// Uses a transaction for efficiency.
pub fn update_file_metadata_batch(
    conn: &mut Connection,
    files: &[(PathBuf, String)], // (path, modified_at)
) -> DbResult<()> {
    let tx = conn.transaction()?;
    let now = Utc::now().to_rfc3339();

    {
        let mut stmt = tx.prepare(
            r#"
            INSERT INTO file_metadata (file_path, modified_at, parsed_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(file_path) DO UPDATE SET
                modified_at = excluded.modified_at,
                parsed_at = excluded.parsed_at
            "#,
        )?;

        for (path, modified_at) in files {
            let path_str = path.to_string_lossy().to_string();
            stmt.execute([&path_str, modified_at, &now])?;
        }
    }

    tx.commit()?;

    info!("Updated metadata for {} files", files.len());
    Ok(())
}

/// Removes metadata for files that no longer exist.
///
/// Call this during cleanup to remove stale entries.
pub fn remove_stale_metadata(
    conn: &Connection,
    existing_files: &[PathBuf],
) -> DbResult<usize> {
    // Get all paths currently in metadata
    let metadata_map = get_all_file_metadata(conn)?;

    // Build set of existing file paths
    let existing_set: std::collections::HashSet<String> = existing_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Find stale entries
    let stale_paths: Vec<&String> = metadata_map
        .keys()
        .filter(|path| !existing_set.contains(*path))
        .collect();

    if stale_paths.is_empty() {
        return Ok(0);
    }

    // Delete stale entries
    let mut deleted = 0;
    for path in &stale_paths {
        conn.execute("DELETE FROM file_metadata WHERE file_path = ?1", [path])?;
        deleted += 1;
    }

    info!("Removed {} stale metadata entries", deleted);
    Ok(deleted)
}

/// Clears all file metadata (forces full rescan on next run).
pub fn clear_all_metadata(conn: &Connection) -> DbResult<()> {
    conn.execute("DELETE FROM file_metadata", [])?;
    info!("Cleared all file metadata");
    Ok(())
}

/// Gets the modification time of a file as an ISO 8601 string.
fn get_file_modified_time(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let datetime: DateTime<Utc> = modified.into();
    Some(datetime.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::sqlite::init_db;
    use rusqlite::Connection;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn test_is_metadata_empty() {
        let conn = setup_test_db();

        // Initially empty
        assert!(is_metadata_empty(&conn).unwrap());

        // Add an entry
        conn.execute(
            "INSERT INTO file_metadata (file_path, modified_at, parsed_at) VALUES ('test.jsonl', '2025-01-01', '2025-01-01')",
            [],
        ).unwrap();

        // No longer empty
        assert!(!is_metadata_empty(&conn).unwrap());
    }

    #[test]
    fn test_update_file_metadata() {
        let conn = setup_test_db();
        let path = Path::new("/test/file.jsonl");
        let modified_at = "2025-01-15T10:00:00Z";

        // Insert
        update_file_metadata(&conn, path, modified_at).unwrap();

        // Verify
        let stored: String = conn
            .query_row(
                "SELECT modified_at FROM file_metadata WHERE file_path = ?1",
                [path.to_string_lossy().to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, modified_at);

        // Update (upsert)
        let new_modified = "2025-01-15T11:00:00Z";
        update_file_metadata(&conn, path, new_modified).unwrap();

        let stored: String = conn
            .query_row(
                "SELECT modified_at FROM file_metadata WHERE file_path = ?1",
                [path.to_string_lossy().to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, new_modified);
    }

    #[test]
    fn test_get_all_file_metadata() {
        let conn = setup_test_db();

        conn.execute(
            "INSERT INTO file_metadata (file_path, modified_at, parsed_at) VALUES ('/a.jsonl', '2025-01-01', '2025-01-01')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO file_metadata (file_path, modified_at, parsed_at) VALUES ('/b.jsonl', '2025-01-02', '2025-01-02')",
            [],
        ).unwrap();

        let metadata = get_all_file_metadata(&conn).unwrap();
        assert_eq!(metadata.len(), 2);
        assert!(metadata.contains_key("/a.jsonl"));
        assert!(metadata.contains_key("/b.jsonl"));
    }

    #[test]
    fn test_get_modified_files_full_scan() {
        let conn = setup_test_db();
        let temp_dir = tempdir().unwrap();

        // Create test files
        let file1 = temp_dir.path().join("a.jsonl");
        let file2 = temp_dir.path().join("b.jsonl");
        File::create(&file1).unwrap().write_all(b"{}").unwrap();
        File::create(&file2).unwrap().write_all(b"{}").unwrap();

        let files = vec![file1.clone(), file2.clone()];

        // With empty metadata, should return all files as new
        let modified = get_modified_files(&conn, &files).unwrap();
        assert_eq!(modified.len(), 2);
        assert!(modified.iter().all(|f| f.is_new));
    }

    #[test]
    fn test_get_modified_files_incremental() {
        let conn = setup_test_db();
        let temp_dir = tempdir().unwrap();

        // Create test files
        let file1 = temp_dir.path().join("unchanged.jsonl");
        let file2 = temp_dir.path().join("changed.jsonl");
        let file3 = temp_dir.path().join("new.jsonl");

        File::create(&file1).unwrap().write_all(b"{}").unwrap();
        File::create(&file2).unwrap().write_all(b"{}").unwrap();

        // Get modification times
        let time1 = get_file_modified_time(&file1).unwrap();
        let time2 = get_file_modified_time(&file2).unwrap();

        // Store metadata for file1 and file2 (with same time)
        update_file_metadata(&conn, &file1, &time1).unwrap();
        update_file_metadata(&conn, &file2, &time2).unwrap();

        // Modify file2
        std::thread::sleep(std::time::Duration::from_millis(50));
        File::create(&file2).unwrap().write_all(b"{\"modified\":true}").unwrap();

        // Create new file
        File::create(&file3).unwrap().write_all(b"{}").unwrap();

        let files = vec![file1.clone(), file2.clone(), file3.clone()];

        // Should return file2 (modified) and file3 (new)
        let modified = get_modified_files(&conn, &files).unwrap();
        assert_eq!(modified.len(), 2);

        let modified_paths: Vec<_> = modified.iter().map(|f| &f.file_path).collect();
        assert!(modified_paths.contains(&&file2));
        assert!(modified_paths.contains(&&file3));

        // file3 should be marked as new
        let new_file = modified.iter().find(|f| f.file_path == file3).unwrap();
        assert!(new_file.is_new);
    }

    #[test]
    fn test_update_file_metadata_batch() {
        let mut conn = setup_test_db();

        let files = vec![
            (PathBuf::from("/a.jsonl"), "2025-01-01T00:00:00Z".to_string()),
            (PathBuf::from("/b.jsonl"), "2025-01-02T00:00:00Z".to_string()),
            (PathBuf::from("/c.jsonl"), "2025-01-03T00:00:00Z".to_string()),
        ];

        update_file_metadata_batch(&mut conn, &files).unwrap();

        let metadata = get_all_file_metadata(&conn).unwrap();
        assert_eq!(metadata.len(), 3);
    }

    #[test]
    fn test_remove_stale_metadata() {
        let conn = setup_test_db();

        // Add some metadata
        conn.execute(
            "INSERT INTO file_metadata (file_path, modified_at, parsed_at) VALUES ('/exists.jsonl', '2025-01-01', '2025-01-01')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO file_metadata (file_path, modified_at, parsed_at) VALUES ('/gone.jsonl', '2025-01-01', '2025-01-01')",
            [],
        ).unwrap();

        // Only /exists.jsonl still exists
        let existing = vec![PathBuf::from("/exists.jsonl")];

        let removed = remove_stale_metadata(&conn, &existing).unwrap();
        assert_eq!(removed, 1);

        let metadata = get_all_file_metadata(&conn).unwrap();
        assert_eq!(metadata.len(), 1);
        assert!(metadata.contains_key("/exists.jsonl"));
    }

    #[test]
    fn test_clear_all_metadata() {
        let conn = setup_test_db();

        // Add some metadata
        conn.execute(
            "INSERT INTO file_metadata (file_path, modified_at, parsed_at) VALUES ('/a.jsonl', '2025-01-01', '2025-01-01')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO file_metadata (file_path, modified_at, parsed_at) VALUES ('/b.jsonl', '2025-01-01', '2025-01-01')",
            [],
        ).unwrap();

        assert!(!is_metadata_empty(&conn).unwrap());

        clear_all_metadata(&conn).unwrap();

        assert!(is_metadata_empty(&conn).unwrap());
    }

    #[test]
    fn test_get_file_modified_time() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.jsonl");
        File::create(&file_path).unwrap().write_all(b"{}").unwrap();

        let time = get_file_modified_time(&file_path);
        assert!(time.is_some());

        // Should be a valid ISO 8601 timestamp
        let time_str = time.unwrap();
        assert!(time_str.contains("T"));
        assert!(time_str.ends_with("Z") || time_str.contains("+"));
    }
}
