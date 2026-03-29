//! Orchestration of all ranking exports into ready-to-write entries.
//!
//! Combines global and per-class queries into a single list of `ExportEntry`
//! values. Each entry carries both the timestamped JSON and the hash-only
//! content, so callers can write to disk or upload to CDN without
//! re-serializing.

use anyhow::Result;
use sqlx::PgPool;

use crate::serialization::{serialize_data_for_hash, serialize_export_data};

use super::characters::{
    CHARACTER_CLASSES, RankedCharacter, fetch_top_level,
    fetch_top_level_by_class, fetch_top_pvp_kills,
    fetch_top_pvp_kills_by_class,
};

/// A single file ready to be exported to disk or CDN.
///
/// Contains both the timestamped JSON and the hash-only JSON.
#[non_exhaustive]
pub struct ExportEntry {
    /// Manifest key for hash storage (e.g. `characters/top-by-level/general`)
    pub manifest_key: String,
    /// File path / R2 key (e.g. `characters/top-by-level/general.json`)
    pub file_key: String,
    /// Pretty-printed JSON with timestamp, ready for upload
    pub json_content: String,
    /// Deterministic JSON for hash comparison (no timestamp)
    pub hash_content: String,
}

/// Fetches and builds all ranking export entries.
///
/// Each class produces one level entry and one `PvP` kills entry.
pub async fn build_all_ranking_exports(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<ExportEntry>> {
    let mut entries = Vec::with_capacity(2 + CHARACTER_CLASSES.len() * 2);

    // Global rankings (all classes combined)
    let level_data = fetch_top_level(pool, limit).await?;
    entries.push(build_entry("characters/top-by-level/general", level_data)?);

    let pvp_data = fetch_top_pvp_kills(pool, limit).await?;
    entries.push(build_entry("characters/top-by-kills/general", pvp_data)?);

    // Per-class rankings
    for &(class_id, class_name) in CHARACTER_CLASSES {
        let class_level =
            fetch_top_level_by_class(pool, limit, class_id).await?;
        entries.push(build_entry(
            &format!("characters/top-by-level/{class_name}"),
            class_level,
        )?);

        let class_pvp =
            fetch_top_pvp_kills_by_class(pool, limit, class_id).await?;
        entries.push(build_entry(
            &format!("characters/top-by-kills/{class_name}"),
            class_pvp,
        )?);
    }

    Ok(entries)
}

/// Builds a single export entry from query results.
///
/// Serializes data twice: once for hashing (no timestamp) and once for output.
fn build_entry(
    manifest_key: &str,
    data: Vec<RankedCharacter>,
) -> Result<ExportEntry> {
    let hash_content = serialize_data_for_hash(&data)?;
    let json_content = serialize_export_data(data)?;
    Ok(ExportEntry {
        manifest_key: manifest_key.to_owned(),
        file_key: format!("{manifest_key}.json"),
        json_content,
        hash_content,
    })
}
