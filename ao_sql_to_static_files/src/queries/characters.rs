//! Character data queries for `PostgreSQL`.
//!
//! Provides functions to fetch character data ranked by level and `PvP` kills.

use anyhow::{Context, Result};
use serde::Serialize;
use sqlx::PgPool;

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

/// Fetches the top characters ranked by level.
///
/// Returns characters ordered by level descending, with experience and name as tiebreakers.
pub async fn fetch_top_level(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<RankedCharacter>> {
    sqlx::query_as::<_, RankedCharacter>(
        r"
        SELECT
            name,
            COALESCE((data->'STATS'->>'ELV')::int, 0)::smallint AS level,
            COALESCE((data->'STATS'->>'EXP')::bigint, 0) AS exp,
            COALESCE((data->'MUERTES'->>'USERMUERTES')::int, 0) AS user_kills,
            COALESCE((data->'INIT'->>'CLASE')::int, 0)::smallint AS class,
            COALESCE((data->'INIT'->>'RAZA')::int, 0)::smallint AS race,
            COALESCE((data->'STATS'->>'GLD')::bigint, 0) AS gold
        FROM characters
        WHERE is_gm = FALSE
        ORDER BY level DESC, exp DESC, name ASC
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
/// Returns characters ordered by user kills descending, with name as tiebreaker.
pub async fn fetch_top_pvp_kills(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<RankedCharacter>> {
    sqlx::query_as::<_, RankedCharacter>(
        r"
        SELECT
            name,
            COALESCE((data->'STATS'->>'ELV')::int, 0)::smallint AS level,
            COALESCE((data->'STATS'->>'EXP')::bigint, 0) AS exp,
            COALESCE((data->'MUERTES'->>'USERMUERTES')::int, 0) AS user_kills,
            COALESCE((data->'INIT'->>'CLASE')::int, 0)::smallint AS class,
            COALESCE((data->'INIT'->>'RAZA')::int, 0)::smallint AS race,
            COALESCE((data->'STATS'->>'GLD')::bigint, 0) AS gold
        FROM characters
        WHERE is_gm = FALSE
        ORDER BY user_kills DESC, name ASC
        LIMIT $1
        ",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("Error al consultar ranking por asesinatos PvP")
}
