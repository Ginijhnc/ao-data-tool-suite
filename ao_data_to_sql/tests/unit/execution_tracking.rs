//! Unit tests for execution timestamp tracking.
//!
//! Tests the read/write functions and filtering logic for incremental processing.

use core::time::Duration;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ao_data_to_sql::execution_tracking::{
    filter_modified_since, read_last_execution_from, write_last_execution_to,
};
use tempfile::TempDir;

// =============================================================================
// Read/Write Tests
// =============================================================================

/// Verifies that reading from a non-existent file returns None instead of panicking.
/// This is the expected behavior on first run when no `last_execution` file exists yet.
#[test]
fn read_last_execution_missing_file_returns_none() {
    let result = read_last_execution_from(Path::new("nonexistent_file_12345"));
    assert!(result.is_none());
}

/// Verifies the complete write-then-read cycle works correctly.
/// 1. Creates a temp directory (auto-cleaned after test)
/// 2. Writes current timestamp to a file
/// 3. Reads it back and verifies it's a valid, recent timestamp (within 10 seconds of now)
#[test]
fn read_write_last_execution_roundtrip() {
    let temp_dir = TempDir::new().expect("crear temp dir");
    let file_path = temp_dir.path().join("last_execution");

    write_last_execution_to(&file_path).expect("escribir timestamp");

    let result = read_last_execution_from(&file_path);
    assert!(result.is_some());

    // Verify the timestamp is recent (written just now, so diff should be < 10 seconds)
    let now = SystemTime::now();
    let diff = now
        .duration_since(result.expect("timestamp valido"))
        .expect("calcular diferencia");
    assert!(diff.as_secs() < 10);
}

/// Verifies that corrupted/invalid file content returns None instead of panicking.
/// If someone manually edits the file or it gets corrupted, we should gracefully
/// fall back to processing all files (same as first run behavior).
#[test]
fn read_last_execution_invalid_content_returns_none() {
    let temp_dir = TempDir::new().expect("crear temp dir");
    let file_path = temp_dir.path().join("last_execution");

    // Write invalid content (not a Unix timestamp)
    std::fs::write(&file_path, "not_a_number")
        .expect("escribir contenido invalido");

    let result = read_last_execution_from(&file_path);
    assert!(result.is_none());
}

// =============================================================================
// Filtering Tests
// =============================================================================

/// Helper to create a test file entry with a timestamp offset from now.
fn make_file_in_past(name: &str, secs_ago: u64) -> (PathBuf, SystemTime) {
    let time = SystemTime::now() - Duration::from_secs(secs_ago);
    (PathBuf::from(name), time)
}

/// First run scenario: when `last_exec` is None, ALL files should be processed
/// regardless of their modification time.
#[test]
fn filter_with_no_last_exec_returns_all_files() {
    let files = vec![
        make_file_in_past("old.chr", 3600), // 1 hour ago
        make_file_in_past("recent.chr", 60), // 1 minute ago
        make_file_in_past("new.chr", 0),    // now
    ];

    let result = filter_modified_since(files, None);

    assert_eq!(result.len(), 3);
    assert!(result.contains(&PathBuf::from("old.chr")));
    assert!(result.contains(&PathBuf::from("recent.chr")));
    assert!(result.contains(&PathBuf::from("new.chr")));
}

/// Files modified AFTER `last_exec` should be included.
#[test]
fn filter_includes_files_newer_than_last_exec() {
    let last_exec = SystemTime::now() - Duration::from_secs(300); // 5 minutes ago
    let files = vec![
        make_file_in_past("new.chr", 60), // 1 minute ago (newer than last_exec)
    ];

    let result = filter_modified_since(files, Some(last_exec));

    assert_eq!(result.len(), 1);
    assert!(result.contains(&PathBuf::from("new.chr")));
}

/// Files modified BEFORE `last_exec` should be excluded.
#[test]
fn filter_excludes_files_older_than_last_exec() {
    let last_exec = SystemTime::now() - Duration::from_secs(300); // 5 minutes ago
    let files = vec![
        make_file_in_past("old.chr", 600), // 10 minutes ago (older than last_exec)
    ];

    let result = filter_modified_since(files, Some(last_exec));

    assert!(result.is_empty());
}

/// Files modified at EXACTLY the same time as `last_exec` should be excluded.
/// We use > comparison, not >=, so equal timestamps are skipped.
#[test]
fn filter_excludes_files_with_equal_timestamp() {
    let last_exec = SystemTime::now();
    let files = vec![(PathBuf::from("same.chr"), last_exec)];

    let result = filter_modified_since(files, Some(last_exec));

    assert!(result.is_empty());
}

/// Mixed scenario: some files newer, some older than `last_exec`.
/// Only the newer ones should be returned.
#[test]
fn filter_mixed_returns_only_newer_files() {
    let last_exec = SystemTime::now() - Duration::from_secs(300); // 5 minutes ago
    let files = vec![
        make_file_in_past("old1.chr", 600), // 10 min ago - exclude
        make_file_in_past("old2.chr", 3600), // 1 hour ago - exclude
        make_file_in_past("new1.chr", 60),  // 1 min ago - include
        make_file_in_past("new2.chr", 120), // 2 min ago - include
    ];

    let result = filter_modified_since(files, Some(last_exec));

    assert_eq!(result.len(), 2);
    assert!(result.contains(&PathBuf::from("new1.chr")));
    assert!(result.contains(&PathBuf::from("new2.chr")));
    assert!(!result.contains(&PathBuf::from("old1.chr")));
    assert!(!result.contains(&PathBuf::from("old2.chr")));
}
