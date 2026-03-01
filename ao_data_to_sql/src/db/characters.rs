use serde_json::Value;
use sqlx::PgPool;

pub async fn insert_characters_batch(
    pool: &PgPool,
    characters: &[(String, Value)],
) -> Result<usize, sqlx::Error> {
    let mut tx = pool.begin().await?;

    for (name, data) in characters {
        sqlx::query(
            r#"
            INSERT INTO characters (name, data)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET data = EXCLUDED.data
            "#,
        )
        .bind(name)
        .bind(data)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(characters.len())
}
