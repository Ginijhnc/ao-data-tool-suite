//! File writing functions for game data export JSON.
//!
//! Handles directory creation and file I/O.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Writes a pre-serialized JSON string to a file.
///
/// Creates parent directories if needed.
pub fn write_export_file(
    output_dir: &Path,
    filename: &str,
    json_content: &str,
) -> Result<()> {
    let file_path = output_dir.join(filename);

    // Create parent directories
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Error al crear directorio: {}", parent.display())
        })?;
    }

    // Write to file
    fs::write(&file_path, json_content).with_context(|| {
        format!("Error al escribir archivo: {}", file_path.display())
    })?;

    tracing::info!("Archivo generado: {}", file_path.display());

    Ok(())
}
