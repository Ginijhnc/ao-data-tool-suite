//! Shared test infrastructure for `ao_sql_to_static_files` E2E tests.
//!
//! These row insertion helpers exist only to seed test fixtures so that the export
//! pipeline (`build_all_dat_exports`, etc.) has data to read.

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

/// Inserts a test row into a named dat table (id, name, data).
///
/// The data should be a flat JSONB object matching the table's schema.
pub async fn insert_named_dat_row(
    pool: &PgPool,
    table: &str,
    id: i32,
    name: &str,
    data: serde_json::Value,
) {
    let sql = format!(
        "INSERT INTO {table} (id, name, data) VALUES ($1, $2, $3) \
         ON CONFLICT (id) DO UPDATE SET name = $2, data = $3"
    );
    sqlx::query(&sql)
        .bind(id)
        .bind(name)
        .bind(data)
        .execute(pool)
        .await
        .expect("Failed to insert named dat row");
}

/// Inserts a test row into an id-only dat table (id, data).
///
/// The data should be a flat JSONB object matching the table's schema.
pub async fn insert_id_only_dat_row(
    pool: &PgPool,
    table: &str,
    id: i32,
    data: serde_json::Value,
) {
    let sql = format!(
        "INSERT INTO {table} (id, data) VALUES ($1, $2) \
         ON CONFLICT (id) DO UPDATE SET data = $2"
    );
    sqlx::query(&sql)
        .bind(id)
        .bind(data)
        .execute(pool)
        .await
        .expect("Failed to insert id-only dat row");
}

/// Inserts a test balance row (section, data).
///
/// The data should be a flat JSONB object matching the balance table's schema.
pub async fn insert_balance_row(
    pool: &PgPool,
    section: &str,
    data: serde_json::Value,
) {
    sqlx::query(
        "INSERT INTO balance (section, data) VALUES ($1, $2) \
         ON CONFLICT (section) DO UPDATE SET data = $2",
    )
    .bind(section)
    .bind(data)
    .execute(pool)
    .await
    .expect("Failed to insert balance row");
}

/// Inserts a test map row (id, name, data).
///
/// The data should be a nested JSONB object: { "MAPA{n}": {...}, "SONIDOS": {...}, ... }
pub async fn insert_map_row(
    pool: &PgPool,
    id: i32,
    name: &str,
    data: serde_json::Value,
) {
    sqlx::query(
        "INSERT INTO maps (id, name, data) VALUES ($1, $2, $3) \
         ON CONFLICT (id) DO UPDATE SET name = $2, data = $3",
    )
    .bind(id)
    .bind(name)
    .bind(data)
    .execute(pool)
    .await
    .expect("Failed to insert map row");
}
