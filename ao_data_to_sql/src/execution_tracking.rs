//! Execution timestamp tracking for incremental processing.
//!
//! Tracks the last execution time to skip unchanged files on subsequent runs.

use core::time::Duration;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};

/// Filename for tracking last execution timestamp.
const LAST_EXECUTION_FILE: &str = "last_execution";

/// Returns the path to the last execution file.
///
/// Checks `LAST_EXECUTION_FILE` env var first, falls back to default filename.
/// The env var is primarily for test isolation and is not intended for production use.
fn last_execution_path() -> PathBuf {
    std::env::var("LAST_EXECUTION_FILE")
        .map_or_else(|_| PathBuf::from(LAST_EXECUTION_FILE), PathBuf::from)
}

/// Reads the last execution timestamp from the default file.
#[must_use]
pub fn read_last_execution() -> Option<SystemTime> {
    read_last_execution_from(&last_execution_path())
}

/// Reads the last execution timestamp from a specific path.
#[must_use]
pub fn read_last_execution_from(path: &Path) -> Option<SystemTime> {
    let content = std::fs::read_to_string(path).ok()?;
    let secs: u64 = content.trim().parse().ok()?;
    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
}

/// Writes the current timestamp to the default last execution file.
pub fn write_last_execution() -> Result<()> {
    write_last_execution_to(&last_execution_path())
}

/// Writes the current timestamp to a specific path.
pub fn write_last_execution_to(path: &Path) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("Error obteniendo timestamp actual")?;
    std::fs::write(path, now.as_secs().to_string())
        .context("Error escribiendo archivo last_execution")?;
    Ok(())
}

/// Filters files to only those modified after the last execution.
/// Returns all files if `last_exec` is None (first run scenario).
#[must_use]
pub fn filter_modified_since(
    files: Vec<(PathBuf, SystemTime)>,
    last_exec: Option<SystemTime>,
) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter(|&(_, mtime)| last_exec.is_none_or(|last| mtime > last))
        .map(|(path, _)| path)
        .collect()
}
