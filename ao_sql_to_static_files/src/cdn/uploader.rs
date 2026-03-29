//! High-level CDN upload orchestration with manifest-based change detection.
//!
//! Coordinates R2 uploads, hash checking, and manifest updates.

use anyhow::Result;
use tracing::info;

use crate::manifest;
use crate::queries::ExportEntry;

use super::{R2Client, R2Config};

/// Uploads files to R2 using manifest-based change detection.
///
/// Hashes data without timestamp, compares against stored hashes,
/// and only uploads files whose content has changed.
pub async fn upload_with_manifest(
    pool: &sqlx::PgPool,
    r2_config: &R2Config,
    entries: Vec<ExportEntry>,
) -> Result<()> {
    let r2_client = R2Client::new(r2_config)?;
    let mut any_changed = false;

    for entry in &entries {
        any_changed |=
            upload_entry_if_changed(pool, &r2_client, entry).await?;
    }

    if any_changed {
        upload_manifest(pool, &r2_client).await?;
    } else {
        info!("Ningún archivo cambió - manifest no actualizado");
    }

    Ok(())
}

/// Uploads a single export entry if its content has changed.
///
/// Returns true if the file was uploaded.
async fn upload_entry_if_changed(
    pool: &sqlx::PgPool,
    r2_client: &R2Client,
    entry: &ExportEntry,
) -> Result<bool> {
    let hash = manifest::sha256_hex(&entry.hash_content);
    let stored = manifest::get_stored_hash(pool, &entry.manifest_key).await?;

    if stored.as_deref() == Some(hash.as_str()) {
        info!("Sin cambios en {} - omitiendo subida", entry.manifest_key);
        return Ok(false);
    }

    info!(
        "Cambio detectado en {} - subiendo a CDN...",
        entry.manifest_key
    );
    r2_client
        .upload_json(
            &entry.file_key,
            entry.json_content.clone(),
            manifest::DATA_FILE_CACHE_CONTROL,
        )
        .await?;
    manifest::upsert_hash(pool, &entry.manifest_key, &hash).await?;
    Ok(true)
}

/// Builds and uploads the manifest file to R2.
async fn upload_manifest(
    pool: &sqlx::PgPool,
    r2_client: &R2Client,
) -> Result<()> {
    let manifest_json = manifest::build_client_manifest(pool).await?;
    r2_client
        .upload_json(
            manifest::MANIFEST_KEY,
            manifest_json,
            manifest::MANIFEST_CACHE_CONTROL,
        )
        .await?;
    info!("Manifest actualizado en R2");
    Ok(())
}
