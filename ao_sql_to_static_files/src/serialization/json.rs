//! Shared JSON serialization functions for game data exports.
//!
//! Provides reusable serialization logic for both disk writer and CDN uploader.

use anyhow::{Context, Result};
use serde::Serialize;

/// Wrapper structure for game data export JSON files.
///
/// Contains timestamp metadata and indexed entries.
#[derive(Serialize)]
struct GameDataExport<T> {
    /// ISO-8601 timestamp of when the file was generated
    generated_at: String,
    /// Indexed data entries
    data: Vec<IndexedEntry<T>>,
}

/// Single entry in an export with position number.
#[derive(Serialize)]
struct IndexedEntry<T> {
    /// Position in the export (1-indexed)
    rank: u32,
    /// Flattened entry data
    #[serde(flatten)]
    entry: T,
}

/// Serializes export data to a JSON string.
///
/// Wraps data with position indices (1-indexed) and timestamp (ISO-8601).
/// Returns pretty-printed JSON string.
pub fn serialize_export_data<T: Serialize>(data: Vec<T>) -> Result<String> {
    // Wrap data with position indices
    let indexed_data = data
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let rank = u32::try_from(index + 1)
                .context("Export index exceeds u32::MAX")?;
            Ok(IndexedEntry { rank, entry })
        })
        .collect::<Result<Vec<IndexedEntry<T>>>>()?;

    // Create file structure with timestamp
    // Note: Using chrono for ISO-8601 formatting. std::time::SystemTime cannot
    // produce RFC3339/ISO-8601 strings directly, requiring manual formatting logic.
    // chrono provides this via .to_rfc3339() for CDN-compatible timestamps.
    let export_file = GameDataExport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        data: indexed_data,
    };

    // Serialize to pretty JSON
    serde_json::to_string_pretty(&export_file)
        .context("Error al serializar datos a JSON")
}

/// Serializes only the data array for hash calculation.
///
/// This excludes the timestamp to ensure identical data produces identical hashes.
pub fn serialize_data_for_hash<T: Serialize>(data: &[T]) -> Result<String> {
    // Wrap data with position indices (same as full serialization)
    let indexed_data: Result<Vec<_>> = data
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let rank = u32::try_from(index + 1)
                .context("Export index exceeds u32::MAX")?;
            Ok(IndexedEntry { rank, entry })
        })
        .collect();

    // Serialize just the data array, deterministically
    serde_json::to_string(&indexed_data?)
        .context("Error al serializar datos para hash")
}
