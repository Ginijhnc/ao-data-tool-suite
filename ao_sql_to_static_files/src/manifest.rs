//! Content-addressed manifest for CDN upload deduplication.
//!
//! Uses SHA-256 hashes stored in `PostgreSQL` to detect whether exported
//! files have changed since the last run, avoiding redundant uploads.

extern crate alloc;

use alloc::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

/// R2 key for the manifest file served to clients.
pub const MANIFEST_KEY: &str = "manifest.json";

/// Cache control header for data files.
pub const DATA_FILE_CACHE_CONTROL: &str =
    "public, max-age=31536000, immutable";

/// Cache control header for manifest file.
pub const MANIFEST_CACHE_CONTROL: &str = "public, max-age=60";

/// Computes the SHA-256 hex digest of a string.
#[must_use]
pub fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    result
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            use core::fmt::Write;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

/// Fetches the stored hash for a file key from the database.
///
/// Returns None if the key has never been exported.
pub async fn get_stored_hash(
    pool: &PgPool,
    file_key: &str,
) -> Result<Option<String>> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT hash FROM export_manifest WHERE file_key = $1")
            .bind(file_key)
            .fetch_optional(pool)
            .await
            .context("Error al consultar hash del manifest")?;

    Ok(row.map(|(hash,)| hash))
}

/// Updates the stored hash for a file key after successful upload.
///
/// Uses upsert to handle both first-time and subsequent exports.
pub async fn upsert_hash(
    pool: &PgPool,
    file_key: &str,
    hash: &str,
) -> Result<()> {
    sqlx::query(
        r"
        INSERT INTO export_manifest (file_key, hash, updated_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (file_key)
        DO UPDATE SET hash = EXCLUDED.hash, updated_at = EXCLUDED.updated_at
        ",
    )
    .bind(file_key)
    .bind(hash)
    .execute(pool)
    .await
    .context("Error al actualizar hash del manifest")?;

    Ok(())
}

/// Metadata for a single exported file (client-facing JSON).
#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct ManifestFileEntry {
    /// SHA-256 hex digest of the file contents
    pub hash: String,
    /// ISO-8601 timestamp of last update
    pub updated_at: String,
}

/// Client-facing manifest served via R2.
#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct ClientManifest {
    /// ISO-8601 timestamp of manifest generation
    pub generated_at: String,
    /// Map of file key to metadata
    pub files: BTreeMap<String, ManifestFileEntry>,
}

/// Builds the client-facing manifest JSON from the database.
///
/// Reads all entries from `export_manifest` and serializes to pretty JSON.
pub async fn build_client_manifest(pool: &PgPool) -> Result<String> {
    let rows: Vec<(String, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT file_key, hash, updated_at FROM export_manifest ORDER BY file_key",
    )
    .fetch_all(pool)
    .await
    .context("Error al consultar manifest completo")?;

    let mut files = BTreeMap::new();
    for (file_key, hash, updated_at) in rows {
        files.insert(
            file_key,
            ManifestFileEntry {
                hash,
                updated_at: updated_at.to_rfc3339(),
            },
        );
    }

    let manifest = ClientManifest {
        generated_at: chrono::Utc::now().to_rfc3339(),
        files,
    };

    serde_json::to_string_pretty(&manifest)
        .context("Error al serializar manifest para cliente")
}
