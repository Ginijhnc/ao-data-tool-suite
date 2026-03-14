//! Character database operations.
//!
//! Handles batch inserts and upserts for character data stored as JSONB.

use futures::stream::{self, StreamExt};
use sqlx::PgPool;
use tracing::error;

use crate::parsers::characters::CharacterData;

/// Result of a database operation: (`success_count`, `error_count`).
pub type DbOperationResult = (usize, usize);

/// Default number of concurrent batch insertions.
const DEFAULT_CONCURRENCY: usize = 8;

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

    // Build multi-row VALUES clause: ($1, $2, $3), ($4, $5, $6), ...
    let placeholders: Vec<String> = (0..characters.len())
        .map(|i| {
            let p1 = i * 3 + 1;
            let p2 = i * 3 + 2;
            let p3 = i * 3 + 3;
            format!("(${p1}, ${p2}, ${p3})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO characters (name, data, is_gm)
        VALUES {}
        ON CONFLICT (name) DO UPDATE SET data = EXCLUDED.data, is_gm = EXCLUDED.is_gm
        ",
        placeholders.join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for &(ref name, ref data, is_gm) in characters {
        query_builder = query_builder.bind(name).bind(data).bind(is_gm);
    }

    query_builder.execute(pool).await?;

    Ok(characters.len())
}

/// Inserts charfiles in parallel batches, returning inserted count and error count.
///
/// Uses concurrent batch processing to maximize connection pool utilization.
pub async fn insert_charfiles(
    pool: &PgPool,
    char_data: &[CharacterData],
    batch_size: usize,
) -> DbOperationResult {
    let batches: Vec<_> = char_data.chunks(batch_size).collect();

    let results: Vec<_> = stream::iter(batches)
        .map(|batch| async move {
            let batch_len = batch.len();
            match insert_characters_batch(pool, batch).await {
                Ok(count) => (count, 0),
                Err(e) => {
                    error!("Error insertando lote: {}", e);
                    (0, batch_len)
                }
            }
        })
        .buffer_unordered(DEFAULT_CONCURRENCY)
        .collect()
        .await;

    let (inserted, insert_errors) = results
        .iter()
        .fold((0, 0), |(acc_ins, acc_err), &(ins, err)| {
            (acc_ins + ins, acc_err + err)
        });

    (inserted, insert_errors)
}
