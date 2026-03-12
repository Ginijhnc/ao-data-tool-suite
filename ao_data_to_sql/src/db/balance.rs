//! Balance database operations.
//!
//! Handles batch inserts and upserts for balance configuration sections.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::dat::balance::ParsedBalanceSection;

/// Balance data ready for database insertion: (`section_name`, `json_data`).
pub type BalanceData = (String, serde_json::Value);

/// Inserts balance sections in a batch using multi-row INSERT with upsert.
pub async fn insert_balance_batch(
    pool: &PgPool,
    sections: &[BalanceData],
) -> Result<usize, sqlx::Error> {
    if sections.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (0..sections.len())
        .map(|i| {
            let p1 = i * 2 + 1;
            let p2 = i * 2 + 2;
            format!("(${p1}, ${p2})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO balance (section, data)
        VALUES {}
        ON CONFLICT (section) DO UPDATE SET
            data = EXCLUDED.data
        ",
        placeholders.join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for item in sections {
        query_builder = query_builder.bind(&item.0).bind(&item.1);
    }

    query_builder.execute(pool).await?;

    Ok(sections.len())
}

/// Converts parsed balance sections to database-ready format.
#[must_use]
pub fn prepare_balance_data(
    sections: Vec<ParsedBalanceSection>,
) -> Vec<BalanceData> {
    sections
        .into_iter()
        .filter_map(|s| {
            serde_json::to_value(&s.data)
                .ok()
                .map(|json| (s.section, json))
        })
        .collect()
}

/// Inserts all balance sections, returning inserted count and error count.
pub async fn insert_balance(
    pool: &PgPool,
    balance_data: &[BalanceData],
    batch_size: usize,
) -> (usize, usize) {
    let mut inserted = 0;
    let mut errors = 0;

    for batch in balance_data.chunks(batch_size) {
        match insert_balance_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                error!("Error insertando lote de balance: {}", e);
                errors += batch.len();
            }
        }
    }

    (inserted, errors)
}
