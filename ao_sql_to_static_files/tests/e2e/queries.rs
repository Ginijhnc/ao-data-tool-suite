//! Tests for character ranking SQL query edge cases.
//!
//! Validates GM filtering, limit, empty table behavior, and tiebreaker ordering
//! against a real `PostgreSQL` instance with JSONB character data.

use ao_sql_to_static_files::queries::{fetch_top_level, fetch_top_pvp_kills};

use crate::common::insert_test_character;
use ao_shared::testing::setup_test_db;

// Verifies that GM characters are never included in the level ranking output.
// A GM with higher level than the regular player must not appear in results,
// ensuring game masters cannot pollute public leaderboards.
#[tokio::test]
async fn fetch_top_level_excludes_gm_characters() {
    let (_container, pool) = setup_test_db().await;

    insert_test_character(&pool, "Player", 45, 999_999, 0, 1, 1, 0, false)
        .await;
    insert_test_character(&pool, "Admin", 50, 9_999_999, 0, 1, 1, 0, true)
        .await;

    let result = fetch_top_level(&pool, 50).await.expect("query failed");

    assert_eq!(result.len(), 1, "only non-GM characters should appear");
    assert_eq!(result.first().expect("expected one result").name, "Player");
}

// Verifies that the limit parameter is honored by the query.
// With 10 characters in the database, requesting 3 must return exactly 3,
// preventing unbounded result sets in production exports.
#[tokio::test]
async fn fetch_top_level_respects_limit() {
    let (_container, pool) = setup_test_db().await;

    for i in 0..10 {
        let name = format!("Player{i}");
        insert_test_character(&pool, &name, i + 1, 0, 0, 1, 1, 0, false).await;
    }

    let result = fetch_top_level(&pool, 3).await.expect("query failed");
    assert_eq!(
        result.len(),
        3,
        "result count should match the requested limit"
    );
}

// Verifies that querying an empty table returns an empty vector rather than
// an error. This is the baseline state on a freshly deployed server before
// any characters have been created.
#[tokio::test]
async fn fetch_top_level_empty_table_returns_empty_vec() {
    let (_container, pool) = setup_test_db().await;

    let result = fetch_top_level(&pool, 50).await.expect("query failed");
    assert!(result.is_empty(), "empty table should return empty result");
}

// When two characters share the same level, the one with higher experience
// must appear first. This verifies the secondary sort key (exp DESC) works
// before the final name tiebreaker is needed.
#[tokio::test]
async fn fetch_top_level_ties_broken_by_exp() {
    let (_container, pool) = setup_test_db().await;

    insert_test_character(&pool, "LowExp", 40, 100_000, 0, 1, 1, 0, false)
        .await;
    insert_test_character(&pool, "HighExp", 40, 900_000, 0, 1, 1, 0, false)
        .await;

    let result = fetch_top_level(&pool, 50).await.expect("query failed");

    assert_eq!(result.len(), 2);
    assert_eq!(
        result.first().expect("expected first result").name,
        "HighExp",
        "higher exp must rank above lower exp at equal level"
    );
}

// When two characters share both level and experience, they must be ordered
// alphabetically by name. This verifies the final tiebreaker (name ASC).
#[tokio::test]
async fn fetch_top_level_ties_broken_by_name() {
    let (_container, pool) = setup_test_db().await;

    insert_test_character(&pool, "Zara", 40, 500_000, 0, 1, 1, 0, false).await;
    insert_test_character(&pool, "Aaron", 40, 500_000, 0, 1, 1, 0, false)
        .await;

    let result = fetch_top_level(&pool, 50).await.expect("query failed");

    assert_eq!(result.len(), 2);
    assert_eq!(
        result.first().expect("expected first result").name,
        "Aaron",
        "alphabetically earlier name must rank first at equal level and exp"
    );
}

// When two characters share the same kill count, they must be ordered
// alphabetically by name. This verifies the PvP tiebreaker (name ASC).
#[tokio::test]
async fn fetch_top_pvp_kills_ties_broken_by_name() {
    let (_container, pool) = setup_test_db().await;

    insert_test_character(&pool, "Zara", 30, 0, 100, 1, 1, 0, false).await;
    insert_test_character(&pool, "Aaron", 30, 0, 100, 1, 1, 0, false).await;

    let result = fetch_top_pvp_kills(&pool, 50).await.expect("query failed");

    assert_eq!(result.len(), 2);
    assert_eq!(
        result.first().expect("expected first result").name,
        "Aaron",
        "alphabetically earlier name must rank first at equal kill count"
    );
}
