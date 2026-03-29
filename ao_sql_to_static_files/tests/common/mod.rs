//! Shared test infrastructure for `ao_sql_to_static_files` E2E tests.
//!
//! Provides character insertion helpers for E2E tests.

use sqlx::PgPool;

/// Inserts a test character directly into the database.
///
/// Builds the JSONB structure that the ranking queries expect.
pub async fn insert_test_character(
    pool: &PgPool,
    name: &str,
    level: i32,
    exp: i64,
    user_kills: i32,
    class: i32,
    race: i32,
    gold: i64,
    is_gm: bool,
) {
    let data = serde_json::json!({
        "INIT": { "CLASE": class, "RAZA": race },
        "STATS": { "ELV": level, "EXP": exp, "GLD": gold },
        "MUERTES": { "USERMUERTES": user_kills }
    });

    sqlx::query(
        "INSERT INTO characters (name, data, is_gm) VALUES ($1, $2, $3)
         ON CONFLICT (name) DO UPDATE SET data = $2, is_gm = $3",
    )
    .bind(name)
    .bind(data)
    .bind(is_gm)
    .execute(pool)
    .await
    .expect("Failed to insert test character");
}
