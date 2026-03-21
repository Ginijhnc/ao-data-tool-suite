//! Character ranking queries for `PostgreSQL`.
//!
//! Provides functions to fetch top characters ranked by level and `PvP` kills.

use anyhow::{Context, Result};
use serde::Serialize;
use sqlx::PgPool;

/// Ranked character data extracted from the database.
///
/// Contains all fields needed for frontend ranking display.
#[derive(Debug, Serialize, sqlx::FromRow)]
#[non_exhaustive]
pub struct RankedCharacter {
    pub name: String,
    pub level: i32,
    pub exp: i32,
    pub user_kills: i32,
    pub class: i32,
    pub race: i32,
    pub gold: i32,
}

/// Fetches the top characters ranked by level.
///
/// Returns characters ordered by level descending, with experience as tiebreaker.
pub async fn fetch_top_level(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<RankedCharacter>> {
    sqlx::query_as::<_, RankedCharacter>(
        r"
        SELECT
            name,
            COALESCE((data->'STATS'->>'ELV')::int, 0) AS level,
            COALESCE((data->'STATS'->>'EXP')::int, 0) AS exp,
            COALESCE((data->'MUERTES'->>'USERMUERTES')::int, 0) AS user_kills,
            COALESCE((data->'INIT'->>'CLASE')::int, 0) AS class,
            COALESCE((data->'INIT'->>'RAZA')::int, 0) AS race,
            COALESCE((data->'STATS'->>'GLD')::int, 0) AS gold
        FROM characters
        WHERE is_gm = FALSE
        ORDER BY level DESC, exp DESC
        LIMIT $1
        ",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("Error al consultar ranking por nivel")
}

/// Fetches the top characters ranked by `PvP` kills.
///
/// Returns characters ordered by user kills descending.
pub async fn fetch_top_pvp_kills(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<RankedCharacter>> {
    sqlx::query_as::<_, RankedCharacter>(
        r"
        SELECT
            name,
            COALESCE((data->'STATS'->>'ELV')::int, 0) AS level,
            COALESCE((data->'STATS'->>'EXP')::int, 0) AS exp,
            COALESCE((data->'MUERTES'->>'USERMUERTES')::int, 0) AS user_kills,
            COALESCE((data->'INIT'->>'CLASE')::int, 0) AS class,
            COALESCE((data->'INIT'->>'RAZA')::int, 0) AS race,
            COALESCE((data->'STATS'->>'GLD')::int, 0) AS gold
        FROM characters
        WHERE is_gm = FALSE
        ORDER BY user_kills DESC
        LIMIT $1
        ",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("Error al consultar ranking por asesinatos PvP")
}
