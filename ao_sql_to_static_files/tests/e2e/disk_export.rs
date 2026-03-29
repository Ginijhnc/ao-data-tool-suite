//! Tests for the full export pipeline in disk write mode.
//!
//! Exercises the complete flow from database query through
//! serialization to filesystem output. Also validates JSONB
//! field extraction and ordering since those are best tested
//! through the full pipeline rather than in isolation.

use ao_sql_to_static_files::queries::{
    build_all_ranking_exports, fetch_top_level, fetch_top_pvp_kills,
};
use ao_sql_to_static_files::serialization::{
    serialize_export_data, write_export_file,
};

use crate::common::insert_test_character;
use ao_shared::testing::setup_test_db;

// Verifies JSONB field extraction and level-descending ordering through the
// full pipeline. Hero (level 45) must appear before Warrior (level 40), and
// all eight character fields must be present and correctly typed.
#[tokio::test]
async fn disk_export_level_ranking_creates_expected_json_structure() {
    let (_container, pool) = setup_test_db().await;
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    // Hero: higher level, fewer kills
    insert_test_character(&pool, "Hero", 45, 999_999, 50, 1, 1, 50_000, false)
        .await;
    // Warrior: lower level, more kills
    insert_test_character(
        &pool, "Warrior", 40, 500_000, 200, 3, 2, 30_000, false,
    )
    .await;

    let level_data = fetch_top_level(&pool, 50).await.expect("query failed");
    let level_json =
        serialize_export_data(level_data).expect("serialization failed");

    write_export_file(
        output_dir.path(),
        "characters/top-by-level.json",
        &level_json,
    )
    .expect("write failed");

    let content = std::fs::read_to_string(
        output_dir.path().join("characters/top-by-level.json"),
    )
    .expect("read failed");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("invalid JSON");

    assert!(
        parsed
            .get("generated_at")
            .expect("missing generated_at")
            .is_string(),
        "level export should have generated_at"
    );
    let data = parsed
        .get("data")
        .expect("missing data")
        .as_array()
        .expect("data should be array");
    assert_eq!(data.len(), 2, "level export should contain both characters");

    // Hero (level 45) should be first in level ranking
    let first = data.first().expect("expected first entry");
    assert_eq!(
        first.get("rank").expect("missing rank"),
        1,
        "first entry rank should be 1"
    );
    assert_eq!(
        first.get("name").expect("missing name"),
        "Hero",
        "Hero should lead level ranking"
    );
    assert_eq!(
        first.get("level").expect("missing level"),
        45,
        "level should be extracted correctly"
    );
    assert_eq!(
        first.get("exp").expect("missing exp"),
        999_999,
        "exp should be extracted correctly"
    );
    assert_eq!(
        first.get("user_kills").expect("missing user_kills"),
        50,
        "user_kills should be extracted correctly"
    );
    assert_eq!(
        first.get("class").expect("missing class"),
        1,
        "class should be extracted correctly"
    );
    assert_eq!(
        first.get("race").expect("missing race"),
        1,
        "race should be extracted correctly"
    );
    assert_eq!(
        first.get("gold").expect("missing gold"),
        50_000,
        "gold should be extracted correctly"
    );

    let second = data.get(1).expect("expected second entry");
    assert_eq!(
        second.get("rank").expect("missing rank"),
        2,
        "second entry rank should be 2"
    );
    assert_eq!(
        second.get("name").expect("missing name"),
        "Warrior",
        "Warrior should be second"
    );
    assert_eq!(
        second.get("level").expect("missing level"),
        40,
        "Warrior level should be correct"
    );
}

// Verifies kills-descending ordering through the full pipeline.
// Warrior (200 kills) must appear before Hero (50 kills).
#[tokio::test]
async fn disk_export_pvp_ranking_creates_expected_json_structure() {
    let (_container, pool) = setup_test_db().await;
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    // Hero: higher level, fewer kills
    insert_test_character(&pool, "Hero", 45, 999_999, 50, 1, 1, 50_000, false)
        .await;
    // Warrior: lower level, more kills
    insert_test_character(
        &pool, "Warrior", 40, 500_000, 200, 3, 2, 30_000, false,
    )
    .await;

    let pvp_data = fetch_top_pvp_kills(&pool, 50).await.expect("query failed");
    let pvp_json =
        serialize_export_data(pvp_data).expect("serialization failed");

    write_export_file(
        output_dir.path(),
        "characters/top-by-kills.json",
        &pvp_json,
    )
    .expect("write failed");

    let content = std::fs::read_to_string(
        output_dir.path().join("characters/top-by-kills.json"),
    )
    .expect("read failed");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("invalid JSON");

    let entries = parsed
        .get("data")
        .expect("missing data")
        .as_array()
        .expect("data should be array");

    // Warrior (200 kills) should be first in PvP ranking
    let first = entries.first().expect("expected first PvP entry");
    assert_eq!(
        first.get("rank").expect("missing rank"),
        1,
        "first PvP entry rank should be 1"
    );
    assert_eq!(
        first.get("name").expect("missing name"),
        "Warrior",
        "Warrior should lead PvP ranking"
    );
    assert_eq!(
        first.get("user_kills").expect("missing user_kills"),
        200,
        "Warrior kills should be correct"
    );

    let second = entries.get(1).expect("expected second PvP entry");
    assert_eq!(
        second.get("rank").expect("missing rank"),
        2,
        "second PvP entry rank should be 2"
    );
    assert_eq!(
        second.get("name").expect("missing name"),
        "Hero",
        "Hero should be second in PvP"
    );
    assert_eq!(
        second.get("user_kills").expect("missing user_kills"),
        50,
        "Hero kills should be correct"
    );
}

// Verifies that GM characters are excluded from both ranking outputs through
// the full pipeline. Only the non-GM player must appear in both the level
// and PvP JSON files, regardless of the GM's superior stats.
#[tokio::test]
async fn disk_export_excludes_gm_characters_from_output() {
    let (_container, pool) = setup_test_db().await;
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    insert_test_character(&pool, "Player", 30, 100, 50, 1, 1, 0, false).await;
    insert_test_character(
        &pool,
        "GameMaster",
        50,
        999_999,
        9999,
        1,
        1,
        0,
        true,
    )
    .await;

    let level_data = fetch_top_level(&pool, 50).await.expect("query failed");
    let pvp_data = fetch_top_pvp_kills(&pool, 50).await.expect("query failed");

    let level_json =
        serialize_export_data(level_data).expect("serialization failed");
    let pvp_json =
        serialize_export_data(pvp_data).expect("serialization failed");

    write_export_file(
        output_dir.path(),
        "characters/top-by-level.json",
        &level_json,
    )
    .expect("write failed");
    write_export_file(
        output_dir.path(),
        "characters/top-by-kills.json",
        &pvp_json,
    )
    .expect("write failed");

    // Both files should only contain the non-GM player
    for filename in [
        "characters/top-by-level.json",
        "characters/top-by-kills.json",
    ] {
        let content =
            std::fs::read_to_string(output_dir.path().join(filename))
                .expect("read failed");
        let parsed: serde_json::Value =
            serde_json::from_str(&content).expect("invalid JSON");
        let data = parsed
            .get("data")
            .expect("missing data")
            .as_array()
            .expect("data should be array");

        assert_eq!(data.len(), 1, "GM should be excluded from {filename}");
        assert_eq!(
            data.first()
                .expect("expected one entry")
                .get("name")
                .expect("missing name"),
            "Player",
            "only Player should appear in {filename}"
        );
    }
}

// Verifies that build_all_ranking_exports produces one file per class per ranking type,
// plus two global files. Characters must appear only in their class-specific files,
// global files must contain all characters, and unrepresented classes must produce
// files with empty data arrays.
#[tokio::test]
async fn disk_export_per_class_creates_correct_files() {
    let (_container, pool) = setup_test_db().await;
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    // Class 1 (Mage): 2 characters
    insert_test_character(
        &pool, "Mage1", 45, 999_999, 50, 1, 1, 50_000, false,
    )
    .await;
    insert_test_character(
        &pool, "Mage2", 30, 100_000, 20, 1, 1, 10_000, false,
    )
    .await;
    // Class 3 (Warrior): 1 character
    insert_test_character(
        &pool, "Warrior1", 40, 500_000, 200, 3, 2, 30_000, false,
    )
    .await;

    let expected_count =
        2 + ao_sql_to_static_files::queries::CHARACTER_CLASSES.len() * 2;
    let entries = build_all_ranking_exports(&pool, 50)
        .await
        .expect("build failed");
    assert_eq!(
        entries.len(),
        expected_count,
        "should produce 2 global + 2 per class per ranking type"
    );

    for entry in &entries {
        write_export_file(
            output_dir.path(),
            &entry.file_key,
            &entry.json_content,
        )
        .expect("write failed");
    }

    // Per-class file for mage (class 1) should have 2 entries
    let mage_level =
        read_json_data(output_dir.path(), "characters/top-by-level/mage.json");
    assert_eq!(
        mage_level.len(),
        2,
        "mage level file should have 2 characters"
    );
    assert!(
        mage_level.iter().all(|c| c["class"] == 1),
        "mage file should only contain class 1"
    );

    // Per-class file for warrior (class 3) should have 1 entry
    let warrior_level = read_json_data(
        output_dir.path(),
        "characters/top-by-level/warrior.json",
    );
    assert_eq!(
        warrior_level.len(),
        1,
        "warrior level file should have 1 character"
    );

    // Unrepresented class (thief, class 5) should have empty data
    let thief_level = read_json_data(
        output_dir.path(),
        "characters/top-by-level/thief.json",
    );
    assert!(
        thief_level.is_empty(),
        "unrepresented class should have empty data"
    );

    // Global file should have all 3 characters
    let global_level = read_json_data(
        output_dir.path(),
        "characters/top-by-level/general.json",
    );
    assert_eq!(
        global_level.len(),
        3,
        "global level file should have all characters"
    );
}

/// Reads a JSON export file and returns the data array.
fn read_json_data(
    base: &std::path::Path,
    filename: &str,
) -> Vec<serde_json::Value> {
    let content = std::fs::read_to_string(base.join(filename))
        .unwrap_or_else(|_| panic!("failed to read {filename}"));
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|_| panic!("invalid JSON in {filename}"));
    parsed
        .get("data")
        .unwrap_or_else(|| panic!("missing data in {filename}"))
        .as_array()
        .unwrap_or_else(|| panic!("data not an array in {filename}"))
        .clone()
}
