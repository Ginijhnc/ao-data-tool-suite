//! File writing functions for game data export JSON.
//!
//! Handles directory creation and file I/O.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use super::json::serialize_export_data;

/// Writes export data to a JSON file.
///
/// Creates parent directories if needed and outputs pretty-printed JSON.
pub fn write_export_file<T: Serialize>(
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

    // Serialize data to JSON
    let json = serialize_export_data(data)?;

    // Write to file
    fs::write(&file_path, json).with_context(|| {
        format!("Error al escribir archivo: {}", file_path.display())
    })?;

    tracing::info!("Archivo generado: {}", file_path.display());

    Ok(())
}
