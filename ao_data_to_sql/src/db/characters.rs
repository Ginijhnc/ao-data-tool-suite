//! Character database operations.
//!
//! Handles batch inserts and upserts for character data stored as JSONB.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::characters::CharacterData;

/// Result of a database operation: (`success_count`, `error_count`).
pub type DbOperationResult = (usize, usize);

/// Inserts characters in a batch using multi-row INSERT with upsert.
///
/// Uses a single INSERT statement with multiple value rows for efficiency.
pub async fn insert_characters_batch(
    pool: &PgPool,
    characters: &[CharacterData],
) -> Result<usize, sqlx::Error> {
    if characters.is_empty() {
        return Ok(0);
    }

    // Build multi-row VALUES clause: ($1, $2), ($3, $4), ...
    let placeholders: Vec<String> = (0..characters.len())
        .map(|i| {
            let p1 = i * 2 + 1;
            let p2 = i * 2 + 2;
            format!("(${p1}, ${p2})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO characters (name, data)
        VALUES {}
        ON CONFLICT (name) DO UPDATE SET data = EXCLUDED.data
        ",
        placeholders.join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    #[allow(
        clippy::needless_borrowed_reference,
        reason = "required by pattern_type_mismatch lint"
    )]
    for &(ref name, ref data) in characters {
        query_builder = query_builder.bind(name).bind(data);
    }

    query_builder.execute(pool).await?;

    Ok(characters.len())
}

/// Inserts charfiles in batches, returning inserted count and error count.
pub async fn insert_charfiles(
    pool: &PgPool,
    char_data: &[CharacterData],
    batch_size: usize,
) -> DbOperationResult {
    let mut inserted = 0;
    let mut insert_errors = 0;

    for batch in char_data.chunks(batch_size) {
        match insert_characters_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                insert_errors += batch.len();
                error!("Error insertando lote: {}", e);
            }
        }
    }

    (inserted, insert_errors)
}
