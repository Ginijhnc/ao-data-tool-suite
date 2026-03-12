//! Blacksmith weapon database operations.
//!
//! Handles batch inserts and upserts for blacksmith weapons.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::dat::blacksmith_weapons::ParsedBlacksmithWeapon;

/// Blacksmith weapon data ready for database insertion: (id, `json_data`).
pub type BlacksmithWeaponData = (i32, serde_json::Value);

/// Inserts blacksmith weapons in a batch using multi-row INSERT with upsert.
pub async fn insert_blacksmith_weapons_batch(
    pool: &PgPool,
    weapons: &[BlacksmithWeaponData],
) -> Result<usize, sqlx::Error> {
    if weapons.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (0..weapons.len())
        .map(|i| {
            let p1 = i * 2 + 1;
            let p2 = i * 2 + 2;
            format!("(${p1}, ${p2})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO blacksmith_weapons (id, data)
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
    for &(id, ref data) in weapons {
        query_builder = query_builder.bind(id).bind(data);
    }

    query_builder.execute(pool).await?;

    Ok(weapons.len())
}

/// Converts parsed blacksmith weapons to database-ready format.
#[must_use]
pub fn prepare_blacksmith_weapon_data(
    weapons: Vec<ParsedBlacksmithWeapon>,
) -> Vec<BlacksmithWeaponData> {
    weapons
        .into_iter()
        .filter_map(|weapon| {
            serde_json::to_value(&weapon.data)
                .ok()
                .map(|json| (weapon.id, json))
        })
        .collect()
}

/// Inserts all blacksmith weapons, returning inserted count and error count.
pub async fn insert_blacksmith_weapons(
    pool: &PgPool,
    weapon_data: &[BlacksmithWeaponData],
    batch_size: usize,
) -> (usize, usize) {
    let mut inserted = 0;
    let mut errors = 0;

    for batch in weapon_data.chunks(batch_size) {
        match insert_blacksmith_weapons_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                error!("Error insertando lote de armas de herrero: {}", e);
                errors += batch.len();
            }
        }
    }

    (inserted, errors)
}
