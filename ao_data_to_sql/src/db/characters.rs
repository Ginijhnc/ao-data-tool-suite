//! Character database operations.
//!
//! Handles batch inserts and upserts for character data stored as JSONB.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::characters::CharacterData;

/// Result of a database operation: (`success_count`, `error_count`).
pub type DbOperationResult = (usize, usize);

/// Inserts characters in a batch using upsert (INSERT ... ON CONFLICT DO UPDATE).
pub async fn insert_characters_batch(
    pool: &PgPool,
    characters: &[CharacterData],
) -> Result<usize, sqlx::Error> {
    let mut tx = pool.begin().await?;

    #[allow(
        clippy::needless_borrowed_reference,
        reason = "required by pattern_type_mismatch lint"
    )]
    for &(ref name, ref data) in characters {
        sqlx::query(
            r"
            INSERT INTO characters (name, data)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET data = EXCLUDED.data
            ",
        )
        .bind(name)
        .bind(data)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
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
