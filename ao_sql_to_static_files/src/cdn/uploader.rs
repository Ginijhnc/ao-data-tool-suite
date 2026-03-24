//! High-level CDN upload orchestration with manifest-based change detection.
//!
//! Coordinates R2 uploads, hash checking, and manifest updates.

use anyhow::Result;
use tracing::info;

use crate::manifest;
use crate::queries::RankedCharacter;
use crate::serialization::{serialize_data_for_hash, serialize_export_data};

use super::{R2Client, R2Config};

/// Uploads files to R2 using manifest-based change detection.
///
/// Hashes data without timestamp, compares against stored hashes,
/// and only uploads files whose content has changed.
pub async fn upload_with_manifest(
    pool: &sqlx::PgPool,
    r2_config: &R2Config,
    level_data: Vec<RankedCharacter>,
    pvp_data: Vec<RankedCharacter>,
) -> Result<()> {
    let r2_client = R2Client::new(r2_config)?;
    let mut any_changed = false;

    // Process level ranking
    any_changed |= upload_ranking_if_changed(
        pool,
        &r2_client,
        "characters/top-by-level",
        "characters/top-by-level.json",
        &level_data,
    )
    .await?;

    // Process PvP ranking
    any_changed |= upload_ranking_if_changed(
        pool,
        &r2_client,
        "characters/top-by-kills",
        "characters/top-by-kills.json",
        &pvp_data,
    )
    .await?;

    if any_changed {
        upload_manifest(pool, &r2_client).await?;
    } else {
        info!("Ningún archivo cambió - manifest no actualizado");
    }

    Ok(())
}

/// Uploads a single ranking file if its data has changed.
///
/// Returns true if the file was uploaded.
async fn upload_ranking_if_changed(
    pool: &sqlx::PgPool,
    r2_client: &R2Client,
    manifest_key: &str,
    r2_key: &str,
    data: &[RankedCharacter],
) -> Result<bool> {
    // Hash only the data, without timestamp
    let data_for_hash = serialize_data_for_hash(data)?;
    let hash = manifest::sha256_hex(&data_for_hash);
    let stored = manifest::get_stored_hash(pool, manifest_key).await?;

    if stored.as_deref() == Some(hash.as_str()) {
        info!("Sin cambios en {manifest_key} - omitiendo subida");
        return Ok(false);
    }

    // Data changed - serialize with timestamp and upload
    info!("Cambio detectado en {manifest_key} - subiendo a CDN...");
    let json_with_timestamp = serialize_export_data(data.to_vec())?;

    r2_client
        .upload_json(
            r2_key,
            json_with_timestamp,
            manifest::DATA_FILE_CACHE_CONTROL,
        )
        .await?;

    manifest::upsert_hash(pool, manifest_key, &hash).await?;
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
