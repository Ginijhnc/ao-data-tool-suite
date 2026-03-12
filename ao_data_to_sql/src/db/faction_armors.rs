//! Faction armor database operations.
//!
//! Handles batch inserts and upserts for faction armor class definitions.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::dat::faction_armors::ParsedFactionArmor;

/// Faction armor data ready for database insertion: (id, `json_data`).
pub type FactionArmorData = (i32, serde_json::Value);

/// Inserts faction armors in a batch using multi-row INSERT with upsert.
pub async fn insert_faction_armors_batch(
    pool: &PgPool,
    armors: &[FactionArmorData],
) -> Result<usize, sqlx::Error> {
    if armors.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (0..armors.len())
        .map(|i| {
            let p1 = i * 2 + 1;
            let p2 = i * 2 + 2;
            format!("(${p1}, ${p2})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO faction_armors (id, data)
        VALUES {}
        ON CONFLICT (id) DO UPDATE SET
            data = EXCLUDED.data
        ",
        placeholders.join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    #[allow(
        clippy::needless_borrowed_reference,
        reason = "required by pattern_type_mismatch lint"
    )]
    for &(id, ref data) in armors {
        query_builder = query_builder.bind(id).bind(data);
    }

    query_builder.execute(pool).await?;

    Ok(armors.len())
}

/// Converts parsed faction armors to database-ready format.
#[must_use]
pub fn prepare_faction_armor_data(
    armors: Vec<ParsedFactionArmor>,
) -> Vec<FactionArmorData> {
    armors
        .into_iter()
        .filter_map(|armor| {
            serde_json::to_value(&armor.data)
                .ok()
                .map(|json| (armor.id, json))
        })
        .collect()
}

/// Inserts all faction armors, returning inserted count and error count.
pub async fn insert_faction_armors(
    pool: &PgPool,
    armor_data: &[FactionArmorData],
    batch_size: usize,
) -> (usize, usize) {
    let mut inserted = 0;
    let mut errors = 0;

    for batch in armor_data.chunks(batch_size) {
        match insert_faction_armors_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                error!(
                    "Error insertando lote de armaduras faccionarias: {}",
                    e
                );
                errors += batch.len();
            }
        }
    }

    (inserted, errors)
}
