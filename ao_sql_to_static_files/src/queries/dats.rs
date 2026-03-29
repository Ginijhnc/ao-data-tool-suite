//! Database query functions for exporting DAT table data.
//!
//! Provides row types, flatten logic, and a builder that produces one
//! `ExportEntry` per dat table. Supports four schema shapes: named (id, name,
//! data), id-only (id, data), balance (section, data), and maps (nested JSONB).

use anyhow::{Context, Result};
use serde_json::json;
use sqlx::PgPool;

use crate::serialization::serialize_flat_export;

use super::ranking_builder::ExportEntry;

/// Row from a named dat table, fetching only id and data.
///
/// Used by npcs, objects, and spells tables. The name column is skipped
/// because NAME/NOMBRE already lives inside the JSONB data.
#[derive(sqlx::FromRow)]
struct NamedDatRow {
    /// Entity numeric identifier
    id: i32,
    /// Flat JSONB data: { "KEY": "value", ... }
    data: serde_json::Value,
}

/// Row from a table with (id, data) columns (no name).
///
/// Used by `carpenter_objects`, `blacksmith_armors`, `blacksmith_weapons`, `faction_armors`.
#[derive(sqlx::FromRow)]
struct IdOnlyDatRow {
    /// Entity numeric identifier
    id: i32,
    /// Flat JSONB data: { "KEY": "value", ... }
    data: serde_json::Value,
}

/// Row from the balance table with (section, data) columns.
///
/// Used exclusively by the balance table.
#[derive(sqlx::FromRow)]
struct BalanceRow {
    /// INI section name (e.g. "DISTRIBUCION", "MODEVASION")
    section: String,
    /// Flat JSONB data: { "KEY": "value", ... }
    data: serde_json::Value,
}

/// Row from the maps table, fetching only id and data.
///
/// Maps data is nested: { "MAPA{n}": {...}, "SONIDOS": {...}, "SONIDO1": {...}, ... }
/// The name column is skipped because NAME lives inside the MAPA{n} section.
#[derive(sqlx::FromRow)]
struct MapRow {
    /// Map numeric identifier
    id: i32,
    /// Nested JSONB data: `section_name` -> { key: value, ... }
    data: serde_json::Value,
}

/// Schema shape of a dat table.
///
/// Determines which row type and flatten function to use for each table.
#[non_exhaustive]
enum DatTableKind {
    /// (id INTEGER, name TEXT, data JSONB) — flat JSONB
    Named,
    /// (id INTEGER, data JSONB) — flat JSONB
    IdOnly,
    /// (section TEXT, data JSONB) — flat JSONB
    Balance,
    /// (id INTEGER, name TEXT, data JSONB) — nested JSONB with MAPA{n} sections
    Map,
}

/// Definition of a dat table to export.
///
/// Maps a SQL table to an output file path and schema shape.
struct DatTableDef {
    /// SQL table name
    table: &'static str,
    /// Output slug under dat/ (e.g. "npcs" -> dat/npcs.json)
    slug: &'static str,
    /// Schema shape
    kind: DatTableKind,
}

/// Registry of all dat tables to export.
const DAT_TABLES: &[DatTableDef] = &[
    DatTableDef {
        table: "npcs",
        slug: "npcs",
        kind: DatTableKind::Named,
    },
    DatTableDef {
        table: "objects",
        slug: "objects",
        kind: DatTableKind::Named,
    },
    DatTableDef {
        table: "spells",
        slug: "spells",
        kind: DatTableKind::Named,
    },
    DatTableDef {
        table: "maps",
        slug: "maps",
        kind: DatTableKind::Map,
    },
    DatTableDef {
        table: "carpenter_objects",
        slug: "carpenter-objects",
        kind: DatTableKind::IdOnly,
    },
    DatTableDef {
        table: "blacksmith_armors",
        slug: "blacksmith-armors",
        kind: DatTableKind::IdOnly,
    },
    DatTableDef {
        table: "blacksmith_weapons",
        slug: "blacksmith-weapons",
        kind: DatTableKind::IdOnly,
    },
    DatTableDef {
        table: "faction_armors",
        slug: "faction-armors",
        kind: DatTableKind::IdOnly,
    },
    DatTableDef {
        table: "balance",
        slug: "balance",
        kind: DatTableKind::Balance,
    },
];

/// Flattens a named row into a single JSON object.
///
/// Injects `ID` from the DB column, then merges all JSONB keys at the same level.
/// Normalizes `NOMBRE` to `NAME` for consistency (spells use `NOMBRE` in their INI).
/// Used by tables parsed from NPCs.dat, Obj.dat, and Hechizos.dat.
fn flatten_named(row: NamedDatRow) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("ID".to_owned(), json!(row.id));
    if let serde_json::Value::Object(data) = row.data {
        map.extend(data);
    }
    // Normalize NOMBRE -> NAME for consistency (Hechizos.dat uses NOMBRE)
    if map.contains_key("NOMBRE")
        && !map.contains_key("NAME")
        && let Some(val) = map.remove("NOMBRE")
    {
        map.insert("NAME".to_owned(), val);
    }
    serde_json::Value::Object(map)
}

/// Flattens an id-only row into a single JSON object.
///
/// Injects `ID` from the DB column, then merges all JSONB keys at the same level.
/// Used by tables parsed from ObjCarpintero.dat, ArmadurasHerrero.dat,
/// ArmasHerrero.dat, and ArmadurasFaccionarias.dat.
fn flatten_id_only(row: IdOnlyDatRow) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("ID".to_owned(), json!(row.id));
    if let serde_json::Value::Object(data) = row.data {
        map.extend(data);
    }
    serde_json::Value::Object(map)
}

/// Flattens a balance row into a single JSON object.
///
/// Injects `SECTION` from the DB column, then merges all JSONB keys at the same level.
fn flatten_balance(row: BalanceRow) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("SECTION".to_owned(), json!(row.section));
    if let serde_json::Value::Object(data) = row.data {
        map.extend(data);
    }
    serde_json::Value::Object(map)
}

/// Flattens a map row into a single JSON object.
///
/// The MAPA{n} section is promoted to top level (its keys become top-level keys).
/// Other sections (SONIDOS, SONIDO1, SONIDO2, ...) remain as nested objects.
/// `ID` is injected from the DB column.
fn flatten_map(row: MapRow) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("ID".to_owned(), json!(row.id));

    if let serde_json::Value::Object(sections) = row.data {
        for (section_name, section_value) in sections {
            if section_name.starts_with("MAPA") {
                // Flatten the main MAPA{n} section: promote its keys to top level
                if let serde_json::Value::Object(inner) = section_value {
                    map.extend(inner);
                }
            } else {
                // Keep other sections (SONIDOS, SONIDO1, ...) as nested objects
                map.insert(section_name, section_value);
            }
        }
    }

    serde_json::Value::Object(map)
}

/// Fetches id and data from a named dat table, ordered by id.
///
/// The name column is intentionally excluded — NAME/NOMBRE lives in JSONB.
async fn fetch_named_table(
    pool: &PgPool,
    table: &str,
) -> Result<Vec<NamedDatRow>> {
    let sql = format!("SELECT id, data FROM {table} ORDER BY id");
    sqlx::query_as::<_, NamedDatRow>(&sql)
        .fetch_all(pool)
        .await
        .with_context(|| format!("Error al consultar tabla {table}"))
}

/// Fetches all rows from an id-only dat table (id, data).
async fn fetch_id_only_table(
    pool: &PgPool,
    table: &str,
) -> Result<Vec<IdOnlyDatRow>> {
    let sql = format!("SELECT id, data FROM {table} ORDER BY id");
    sqlx::query_as::<_, IdOnlyDatRow>(&sql)
        .fetch_all(pool)
        .await
        .with_context(|| format!("Error al consultar tabla {table}"))
}

/// Fetches all rows from the balance table (section, data).
async fn fetch_balance_table(pool: &PgPool) -> Result<Vec<BalanceRow>> {
    sqlx::query_as::<_, BalanceRow>(
        "SELECT section, data FROM balance ORDER BY section",
    )
    .fetch_all(pool)
    .await
    .context("Error al consultar tabla balance")
}

/// Fetches id and data from the maps table, ordered by id.
///
/// The name column is intentionally excluded — NAME lives inside the MAPA{n} section.
async fn fetch_map_table(pool: &PgPool) -> Result<Vec<MapRow>> {
    sqlx::query_as::<_, MapRow>("SELECT id, data FROM maps ORDER BY id")
        .fetch_all(pool)
        .await
        .context("Error al consultar tabla maps")
}

/// Builds a single dat export entry from flattened rows.
///
/// Serializes data twice: once for hashing (compact, no timestamp) and once
/// for output (pretty-printed with timestamp).
fn build_dat_entry(
    slug: &str,
    data: &[serde_json::Value],
) -> Result<ExportEntry> {
    let manifest_key = format!("dat/{slug}");
    let hash_content = serde_json::to_string(data)
        .context("Error al serializar datos para hash")?;
    let json_content = serialize_flat_export(data.to_vec())?;
    Ok(ExportEntry {
        manifest_key: manifest_key.clone(),
        file_key: format!("{manifest_key}.json"),
        json_content,
        hash_content,
    })
}

/// Builds export entries for all dat tables.
///
/// Queries each table, flattens rows, and produces ready-to-export entries.
pub async fn build_all_dat_exports(pool: &PgPool) -> Result<Vec<ExportEntry>> {
    let mut entries = Vec::with_capacity(DAT_TABLES.len());

    for def in DAT_TABLES {
        let flat_rows: Vec<serde_json::Value> = match def.kind {
            DatTableKind::Named => {
                let rows = fetch_named_table(pool, def.table).await?;
                rows.into_iter().map(flatten_named).collect()
            }
            DatTableKind::IdOnly => {
                let rows = fetch_id_only_table(pool, def.table).await?;
                rows.into_iter().map(flatten_id_only).collect()
            }
            DatTableKind::Balance => {
                let rows = fetch_balance_table(pool).await?;
                rows.into_iter().map(flatten_balance).collect()
            }
            DatTableKind::Map => {
                let rows = fetch_map_table(pool).await?;
                rows.into_iter().map(flatten_map).collect()
            }
        };
        entries.push(build_dat_entry(def.slug, &flat_rows)?);
    }

    Ok(entries)
}
