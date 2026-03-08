//! Object database operations.
//!
//! Handles batch inserts and upserts for object data stored as JSONB.

use sqlx::PgPool;
use tracing::error;

use crate::parsers::dat::objects::ParsedObject;

/// Object data ready for database insertion: (id, name, `json_data`).
pub type ObjectData = (i32, String, serde_json::Value);

/// Inserts objects in a batch using multi-row INSERT with upsert.
pub async fn insert_objects_batch(
    pool: &PgPool,
    objects: &[ObjectData],
) -> Result<usize, sqlx::Error> {
    if objects.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (0..objects.len())
        .map(|i| {
            let p1 = i * 3 + 1;
            let p2 = i * 3 + 2;
            let p3 = i * 3 + 3;
            format!("(${p1}, ${p2}, ${p3})")
        })
        .collect();

    let query = format!(
        r"
        INSERT INTO objects (id, name, data)
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
    for &(id, ref name, ref data) in objects {
        query_builder = query_builder.bind(id).bind(name).bind(data);
    }

    query_builder.execute(pool).await?;

    Ok(objects.len())
}

/// Converts parsed objects to database-ready format.
#[must_use]
pub fn prepare_object_data(objects: Vec<ParsedObject>) -> Vec<ObjectData> {
    objects
        .into_iter()
        .filter_map(|obj| {
            serde_json::to_value(&obj.data)
                .ok()
                .map(|json| (obj.id, obj.name, json))
        })
        .collect()
}

/// Inserts all objects, returning inserted count and error count.
pub async fn insert_objects(
    pool: &PgPool,
    object_data: &[ObjectData],
    batch_size: usize,
) -> (usize, usize) {
    let mut inserted = 0;
    let mut errors = 0;

    for batch in object_data.chunks(batch_size) {
        match insert_objects_batch(pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                error!("Error insertando lote de objetos: {}", e);
                errors += batch.len();
            }
        }
    }

    (inserted, errors)
}
