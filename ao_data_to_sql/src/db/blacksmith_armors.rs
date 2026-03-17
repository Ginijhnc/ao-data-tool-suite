//! Blacksmith armor database operations.
//!
//! Handles batch inserts and upserts for blacksmith armors.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::dat::blacksmith_armors::ParsedBlacksmithArmor;
use crate::parsers::ini::coerce_ini_section;

/// Blacksmith armor data ready for database insertion: (id, `json_data`).
pub type BlacksmithArmorData = (i32, serde_json::Value);

/// Inserts blacksmith armors in a batch using multi-row INSERT with upsert.
pub async fn insert_blacksmith_armors_batch(
    pool: &PgPool,
    armors: &[BlacksmithArmorData],
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
        INSERT INTO blacksmith_armors (id, data)
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

/// Converts parsed blacksmith armors to database-ready format.
#[must_use]
pub fn prepare_blacksmith_armor_data(
    armors: Vec<ParsedBlacksmithArmor>,
) -> Vec<BlacksmithArmorData> {
    armors
        .into_iter()
        .map(|armor| {
            let json = coerce_ini_section(armor.data);
            (armor.id, json)
        })
        .collect()
}

/// Inserts all blacksmith armors, returning inserted count and error count.
pub async fn insert_blacksmith_armors(
    pool: &PgPool,
    armor_data: &[BlacksmithArmorData],
    batch_size: usize,
) -> (usize, usize) {
    let mut inserted = 0;
    let mut errors = 0;

    for batch in armor_data.chunks(batch_size) {
        match insert_blacksmith_armors_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                error!("Error insertando lote de armaduras de herrero: {}", e);
                errors += batch.len();
            }
        }
    }

    (inserted, errors)
}
