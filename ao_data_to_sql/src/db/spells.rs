//! Spell database operations.
//!
//! Handles batch inserts and upserts for spell data stored as JSONB.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::dat::spells::ParsedSpell;
use crate::parsers::ini::coerce_ini_section;

/// Spell data ready for database insertion: (id, name, `json_data`).
pub type SpellData = (i32, String, serde_json::Value);

/// Inserts spells in a batch using multi-row INSERT with upsert.
pub async fn insert_spells_batch(
    pool: &PgPool,
    spells: &[SpellData],
) -> Result<usize, sqlx::Error> {
    if spells.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (0..spells.len())
        .map(|i| {
            let p1 = i * 3 + 1;
            let p2 = i * 3 + 2;
            let p3 = i * 3 + 3;
            format!("(${p1}, ${p2}, ${p3})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO spells (id, name, data)
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
    for &(id, ref name, ref data) in spells {
        query_builder = query_builder.bind(id).bind(name).bind(data);
    }

    query_builder.execute(pool).await?;

    Ok(spells.len())
}

/// Converts parsed spells to database-ready format.
#[must_use]
pub fn prepare_spell_data(spells: Vec<ParsedSpell>) -> Vec<SpellData> {
    spells
        .into_iter()
        .map(|spell| {
            let json = coerce_ini_section(spell.data);
            (spell.id, spell.name, json)
        })
        .collect()
}

/// Inserts all spells, returning inserted count and error count.
pub async fn insert_spells(
    pool: &PgPool,
    spell_data: &[SpellData],
    batch_size: usize,
) -> (usize, usize) {
    let mut inserted = 0;
    let mut errors = 0;

    for batch in spell_data.chunks(batch_size) {
        match insert_spells_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                error!("Error insertando lote de hechizos: {}", e);
                errors += batch.len();
            }
        }
    }

    (inserted, errors)
}
