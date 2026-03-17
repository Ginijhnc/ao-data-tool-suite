//! Carpenter object database operations.
//!
//! Handles batch inserts and upserts for carpenter objects.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::dat::carpenter::ParsedCarpenterObject;
use crate::parsers::ini::coerce_ini_section;

/// Carpenter object data ready for database insertion: (id, `json_data`).
pub type CarpenterObjectData = (i32, serde_json::Value);

/// Inserts carpenter objects in a batch using multi-row INSERT with upsert.
pub async fn insert_carpenter_objects_batch(
    pool: &PgPool,
    objects: &[CarpenterObjectData],
) -> Result<usize, sqlx::Error> {
    if objects.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (0..objects.len())
        .map(|i| {
            let p1 = i * 2 + 1;
            let p2 = i * 2 + 2;
            format!("(${p1}, ${p2})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO carpenter_objects (id, data)
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
    for &(id, ref data) in objects {
        query_builder = query_builder.bind(id).bind(data);
    }

    query_builder.execute(pool).await?;

    Ok(objects.len())
}

/// Converts parsed carpenter objects to database-ready format.
#[must_use]
pub fn prepare_carpenter_object_data(
    objects: Vec<ParsedCarpenterObject>,
) -> Vec<CarpenterObjectData> {
    objects
        .into_iter()
        .map(|obj| {
            let json = coerce_ini_section(obj.data);
            (obj.id, json)
        })
        .collect()
}

/// Inserts all carpenter objects, returning inserted count and error count.
pub async fn insert_carpenter_objects(
    pool: &PgPool,
    object_data: &[CarpenterObjectData],
    batch_size: usize,
) -> (usize, usize) {
    let mut inserted = 0;
    let mut errors = 0;

    for batch in object_data.chunks(batch_size) {
        match insert_carpenter_objects_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                error!(
                    "Error insertando lote de objetos de carpintero: {}",
                    e
                );
                errors += batch.len();
            }
        }
    }

    (inserted, errors)
}
