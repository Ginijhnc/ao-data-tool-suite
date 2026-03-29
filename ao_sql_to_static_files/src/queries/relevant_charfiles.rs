//! Full character profile exports for ranked characters.
//!
//! Fetches complete JSONB data for all characters appearing in any ranking
//! and builds one `ExportEntry` per character for CDN distribution.

use anyhow::{Context, Result};
use serde_json::json;
use sqlx::PgPool;

use crate::serialization::serialize_single_export;

use super::ranking_builder::ExportEntry;

/// Row returned by the charfile batch query.
#[derive(sqlx::FromRow)]
struct CharfileRow {
    /// Character name (from the DB column, not inside JSONB)
    name: String,
    /// Full JSONB data blob as stored in the characters table
    data: serde_json::Value,
}

/// Converts a character name to a URL-safe slug.
///
/// Lowercases and replaces spaces with hyphens.
fn name_to_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

/// Fetches full JSONB profiles for the given character names in a single query.
async fn fetch_charfile_rows(
    pool: &PgPool,
    names: Vec<String>,
) -> Result<Vec<CharfileRow>> {
    sqlx::query_as::<_, CharfileRow>(
        "SELECT name, data
         FROM characters
         WHERE name = ANY($1)
           AND is_gm = FALSE",
    )
    .bind(names)
    .fetch_all(pool)
    .await
    .context("Error al consultar perfiles de personajes")
}

/// Builds a single charfile export entry from a DB row.
///
/// Injects `name` into the data object since it lives in the DB column, not in JSONB.
fn build_charfile_entry(row: CharfileRow) -> Result<ExportEntry> {
    let slug = name_to_slug(&row.name);
    let manifest_key = format!("characters/profiles/{slug}");

    let mut data_map = match row.data {
        serde_json::Value::Object(map) => map,
        other => {
            let mut m = serde_json::Map::new();
            m.insert("data".to_owned(), other);
            m
        }
    };
    data_map.insert("name".to_owned(), json!(row.name));
    let data = serde_json::Value::Object(data_map);

    let hash_content = serde_json::to_string(&data)
        .context("Error al serializar perfil para hash")?;
    let json_content = serialize_single_export(&data)?;

    Ok(ExportEntry {
        manifest_key: manifest_key.clone(),
        file_key: format!("{manifest_key}.json"),
        json_content,
        hash_content,
    })
}

/// Builds export entries for all ranked character profiles.
///
/// Fetches full JSONB in a single batch query and produces one entry per character.
pub async fn build_all_charfile_exports(
    pool: &PgPool,
    names: Vec<String>,
) -> Result<Vec<ExportEntry>> {
    let rows = fetch_charfile_rows(pool, names).await?;
    rows.into_iter().map(build_charfile_entry).collect()
}
