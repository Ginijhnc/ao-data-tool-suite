//! Character data queries for `PostgreSQL`.
//!
//! Provides functions to fetch character data ranked by level and `PvP` kills,
//! with variants for filtering by class. A shared query builder handles all variants
//! to keep SQL logic in one place.

use anyhow::{Context, Result};
use serde::Serialize;
use sqlx::PgPool;

/// Character class definitions mapping database IDs to URL-safe names.
///
/// Both Dakara C++ and Alkon VB6 share this enum (1=Mage through 12=Pirate).
pub const CHARACTER_CLASSES: &[(i16, &str)] = &[
    (1, "mage"),
    (2, "cleric"),
    (3, "warrior"),
    (4, "assassin"),
    (5, "thief"),
    (6, "bard"),
    (7, "druid"),
    (8, "bandit"),
    (9, "paladin"),
    (10, "hunter"),
    (11, "worker"),
    (12, "pirate"),
];

/// SQL ORDER BY clause for level-based ranking.
const ORDER_BY_LEVEL: &str = "level DESC, exp DESC, name ASC";

/// SQL ORDER BY clause for `PvP` kills ranking.
const ORDER_BY_KILLS: &str = "user_kills DESC, name ASC";

/// Character data extracted from the database.
///
/// Contains all fields needed for frontend display.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[non_exhaustive]
pub struct RankedCharacter {
    pub name: String,
    pub level: i16,
    pub exp: i64,
    pub user_kills: i32,
    pub class: i16,
    pub race: i16,
    pub gold: i64,
}

/// Builds and executes a ranking query with optional class filter.
///
/// Constructs the SQL dynamically to include a class filter when provided.
async fn fetch_ranked_characters(
    pool: &PgPool,
    limit: i32,
    order_by: &str,
    class_filter: Option<i16>,
) -> Result<Vec<RankedCharacter>> {
    let class_clause = if class_filter.is_some() {
        "AND COALESCE((data->'INIT'->>'CLASE')::int, 0) = $2"
    } else {
        ""
    };

    let sql = format!(
        r"SELECT
            name,
            COALESCE((data->'STATS'->>'ELV')::int, 0)::smallint AS level,
            COALESCE((data->'STATS'->>'EXP')::bigint, 0) AS exp,
            COALESCE((data->'MUERTES'->>'USERMUERTES')::int, 0) AS user_kills,
            COALESCE((data->'INIT'->>'CLASE')::int, 0)::smallint AS class,
            COALESCE((data->'INIT'->>'RAZA')::int, 0)::smallint AS race,
            COALESCE((data->'STATS'->>'GLD')::bigint, 0) AS gold
        FROM characters
        WHERE is_gm = FALSE {class_clause}
        ORDER BY {order_by}
        LIMIT $1"
    );

    let base_query = sqlx::query_as::<_, RankedCharacter>(&sql).bind(limit);

    let rows = if let Some(class_id) = class_filter {
        base_query.bind(class_id).fetch_all(pool).await
    } else {
        base_query.fetch_all(pool).await
    };

    rows.context("Error al consultar ranking")
}

/// Fetches the top characters ranked by level.
///
/// Returns characters ordered by level descending, with experience and name as tiebreakers.
pub async fn fetch_top_level(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<RankedCharacter>> {
    fetch_ranked_characters(pool, limit, ORDER_BY_LEVEL, None)
        .await
        .context("Error al consultar ranking por nivel")
}

/// Fetches the top characters ranked by `PvP` kills.
///
/// Returns characters ordered by user kills descending, with name as tiebreaker.
pub async fn fetch_top_pvp_kills(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<RankedCharacter>> {
    fetch_ranked_characters(pool, limit, ORDER_BY_KILLS, None)
        .await
        .context("Error al consultar ranking por asesinatos PvP")
}

/// Fetches the top characters of a specific class ranked by level.
pub async fn fetch_top_level_by_class(
    pool: &PgPool,
    limit: i32,
    class_id: i16,
) -> Result<Vec<RankedCharacter>> {
    fetch_ranked_characters(pool, limit, ORDER_BY_LEVEL, Some(class_id))
        .await
        .context("Error al consultar ranking por nivel filtrado por clase")
}

/// Fetches the top characters of a specific class ranked by `PvP` kills.
pub async fn fetch_top_pvp_kills_by_class(
    pool: &PgPool,
    limit: i32,
    class_id: i16,
) -> Result<Vec<RankedCharacter>> {
    fetch_ranked_characters(pool, limit, ORDER_BY_KILLS, Some(class_id))
        .await
        .context(
            "Error al consultar ranking por asesinatos PvP filtrado por clase",
        )
}
