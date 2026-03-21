//! File writing functions for ranking JSON export.
//!
//! Handles directory creation, rank assignment, and JSON serialization.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

/// Wrapper structure for ranking JSON files.
///
/// Contains timestamp metadata and ranked entries.
#[derive(Serialize)]
struct RankingFile<T> {
    /// ISO-8601 timestamp of when the file was generated
    generated_at: String,
    /// Ranked data entries
    data: Vec<RankedEntry<T>>,
}

/// Single entry in a ranking with position number.
#[derive(Serialize)]
struct RankedEntry<T> {
    /// Position in the ranking (1-indexed)
    rank: u32,
    /// Flattened entry data
    #[serde(flatten)]
    entry: T,
}

/// Writes ranking data to a JSON file.
///
/// Creates parent directories if needed and outputs pretty-printed JSON.
pub fn write_ranking_file<T: Serialize>(
    output_dir: &Path,
    filename: &str,
    data: Vec<T>,
) -> Result<()> {
    let file_path = output_dir.join(filename);

    // Create parent directories
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Error al crear directorio: {}", parent.display())
        })?;
    }

    // Wrap data with ranks
    let ranked_data = data
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let rank = u32::try_from(index + 1)
                .context("Ranking index exceeds u32::MAX")?;
            Ok(RankedEntry { rank, entry })
        })
        .collect::<Result<Vec<RankedEntry<T>>>>()?;

    // Create file structure with timestamp
    // Note: Using chrono for ISO-8601 formatting. std::time::SystemTime cannot
    // produce RFC3339/ISO-8601 strings directly, requiring manual formatting logic.
    // chrono provides this via .to_rfc3339() for CDN-compatible timestamps.
    let ranking_file = RankingFile {
        generated_at: chrono::Utc::now().to_rfc3339(),
        data: ranked_data,
    };

    // Write pretty JSON
    let json = serde_json::to_string_pretty(&ranking_file)
        .context("Error al serializar datos a JSON")?;

    fs::write(&file_path, json).with_context(|| {
        format!("Error al escribir archivo: {}", file_path.display())
    })?;

    tracing::info!("Archivo generado: {}", file_path.display());

    Ok(())
}
