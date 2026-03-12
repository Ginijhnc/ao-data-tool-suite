//! Map database operations.
//!
//! Handles batch inserts and upserts for map data stored as JSONB.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::dat::maps::ParsedMap;

/// Map data ready for database insertion: (id, name, `json_data`).
pub type MapData = (i32, String, serde_json::Value);

/// Inserts maps in a batch using multi-row INSERT with upsert.
pub async fn insert_maps_batch(
    pool: &PgPool,
    maps: &[MapData],
) -> Result<usize, sqlx::Error> {
    if maps.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (0..maps.len())
        .map(|i| {
            let p1 = i * 3 + 1;
            let p2 = i * 3 + 2;
            let p3 = i * 3 + 3;
            format!("(${p1}, ${p2}, ${p3})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO maps (id, name, data)
        VALUES {}
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            data = EXCLUDED.data
        ",
        placeholders.join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    #[allow(
        clippy::needless_borrowed_reference,
        reason = "required by pattern_type_mismatch lint"
    )]
    for &(id, ref name, ref data) in maps {
        query_builder = query_builder.bind(id).bind(name).bind(data);
    }

    query_builder.execute(pool).await?;

    Ok(maps.len())
}

/// Converts parsed maps to database-ready format.
#[must_use]
pub fn prepare_map_data(maps: Vec<ParsedMap>) -> Vec<MapData> {
    maps.into_iter()
        .map(|map| (map.id, map.name, map.data))
        .collect()
}

/// Inserts all maps, returning inserted count and error count.
pub async fn insert_maps(
    pool: &PgPool,
    map_data: &[MapData],
    batch_size: usize,
) -> (usize, usize) {
    let mut inserted = 0;
    let mut errors = 0;

    for batch in map_data.chunks(batch_size) {
        match insert_maps_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                error!("Error insertando lote de mapas: {}", e);
                errors += batch.len();
            }
        }
    }

    (inserted, errors)
}
