//! NPC database operations.
//!
//! Handles batch inserts and upserts for NPC data stored as JSONB.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::dat::npcs::ParsedNpc;

/// NPC data ready for database insertion: (id, name, `json_data`).
pub type NpcData = (i32, String, serde_json::Value);

/// Inserts NPCs in a batch using multi-row INSERT with upsert.
pub async fn insert_npcs_batch(
    pool: &PgPool,
    npcs: &[NpcData],
) -> Result<usize, sqlx::Error> {
    if npcs.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (0..npcs.len())
        .map(|i| {
            let p1 = i * 3 + 1;
            let p2 = i * 3 + 2;
            let p3 = i * 3 + 3;
            format!("(${p1}, ${p2}, ${p3})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO npcs (id, name, data)
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
    for &(id, ref name, ref data) in npcs {
        query_builder = query_builder.bind(id).bind(name).bind(data);
    }

    query_builder.execute(pool).await?;

    Ok(npcs.len())
}

/// Converts parsed NPCs to database-ready format.
#[must_use]
pub fn prepare_npc_data(npcs: Vec<ParsedNpc>) -> Vec<NpcData> {
    npcs.into_iter()
        .filter_map(|npc| {
            serde_json::to_value(&npc.data)
                .ok()
                .map(|json| (npc.id, npc.name, json))
        })
        .collect()
}

/// Inserts all NPCs, returning inserted count and error count.
pub async fn insert_npcs(
    pool: &PgPool,
    npc_data: &[NpcData],
    batch_size: usize,
) -> (usize, usize) {
    let mut inserted = 0;
    let mut errors = 0;

    for batch in npc_data.chunks(batch_size) {
        match insert_npcs_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                error!("Error insertando lote de NPCs: {}", e);
                errors += batch.len();
            }
        }
    }

    (inserted, errors)
}
