//! End-to-end tests for ArmasHerrero.dat parsing and database import.
//!
//! Tests verify blacksmith weapon count, data integrity, and incremental update behavior.

mod helpers;

use std::process::Command;

use tempfile::TempDir;

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, setup_test_db,
    with_test_db_env,
};
use helpers::{
    BLACKSMITH_WEAPONS_FIXTURES, verify_blacksmith_weapon_count,
    verify_blacksmith_weapon_data_matches_fixture,
};

/// Verifies the number of blacksmith weapons matches expected count.
#[tokio::test]
async fn blacksmith_weapon_count_matches_expected() {
    verify_blacksmith_weapon_count(26).await;
}

/// Verifies blacksmith weapon 1 (Daga - Index 15) data matches expected fixture.
#[tokio::test]
async fn blacksmith_weapon_1_daga_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_WEAPONS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("BLACKSMITHWEAPON1.json");
    verify_blacksmith_weapon_data_matches_fixture(1, fixture).await;
}

/// Verifies blacksmith weapon 16 (Hacha de Barbaro - Index 159) data matches expected fixture.
#[tokio::test]
async fn blacksmith_weapon_16_hacha_barbaro_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_WEAPONS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("BLACKSMITHWEAPON16.json");
    verify_blacksmith_weapon_data_matches_fixture(16, fixture).await;
}

/// Verifies incremental update: parser detects modified ArmasHerrero.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_blacksmith_weapon() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_weapons_dat = temp_dir.path().join("ArmasHerrero.dat");

    let original_weapons = crate_dir().join("dat").join("ArmasHerrero.dat");
    std::fs::copy(&original_weapons, &temp_weapons_dat)
        .expect("failed to copy ArmasHerrero.dat");

    let temp_dats_dir = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM blacksmith_weapons")
            .fetch_one(&pool)
            .await
            .expect("failed to count blacksmith weapons");
    assert_eq!(
        count, 26,
        "initial import should have 26 blacksmith weapons"
    );

    std::thread::sleep(core::time::Duration::from_secs(2));

    let weapons_content = std::fs::read_to_string(&temp_weapons_dat)
        .expect("failed to read ArmasHerrero.dat");
    let modified_content =
        weapons_content.replace("[Arma1] ' Daga", "[Arma1] ' Daga MODIFICADO");
    std::fs::write(&temp_weapons_dat, modified_content)
        .expect("failed to write modified ArmasHerrero.dat");

    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    let (updated_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM blacksmith_weapons WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch blacksmith weapon 1");

    let comment = updated_data
        .get("_COMMENT")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert_eq!(
        comment, "Daga MODIFICADO",
        "Blacksmith weapon 1 comment should be updated after modification"
    );

    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM blacksmith_weapons")
            .fetch_one(&pool)
            .await
            .expect("failed to count blacksmith weapons");
    assert_eq!(final_count, 26, "count should remain 26 after modification");
}

/// Verifies incremental update: parser detects new blacksmith weapon added.
#[tokio::test]
async fn incremental_update_detects_new_blacksmith_weapon() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_weapons_dat = temp_dir.path().join("ArmasHerrero.dat");

    let original_weapons = crate_dir().join("dat").join("ArmasHerrero.dat");
    std::fs::copy(&original_weapons, &temp_weapons_dat)
        .expect("failed to copy ArmasHerrero.dat");

    let temp_dats_dir = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM blacksmith_weapons")
            .fetch_one(&pool)
            .await
            .expect("failed to count blacksmith weapons");
    assert_eq!(
        count, 26,
        "initial import should have 26 blacksmith weapons"
    );

    let (exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM blacksmith_weapons WHERE id = 27)",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check Arma27 existence");
    assert!(!exists, "Arma27 should not exist initially");

    std::thread::sleep(core::time::Duration::from_secs(2));

    let weapons_content = std::fs::read_to_string(&temp_weapons_dat)
        .expect("failed to read ArmasHerrero.dat");
    let new_weapon_fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_WEAPONS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_blacksmith_weapon_27.txt");
    let new_weapon = std::fs::read_to_string(&new_weapon_fixture)
        .expect("failed to read test Arma27 fixture");
    let modified_content = format!("{weapons_content}\n{new_weapon}");
    std::fs::write(&temp_weapons_dat, modified_content)
        .expect("failed to write modified ArmasHerrero.dat");

    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    let (weapon_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM blacksmith_weapons WHERE id = 27)",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check Arma27 existence");
    assert!(weapon_exists, "Arma27 should exist after addition");

    let (weapon_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM blacksmith_weapons WHERE id = 27")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch Arma27");

    let index = weapon_data
        .get("OBJ_INDEX")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(index, "999", "Arma27 should have correct OBJ_INDEX");

    assert!(
        weapon_data.get("_COMMENT").is_none(),
        "Arma27 should not have _COMMENT field (no comment in source)"
    );

    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM blacksmith_weapons")
            .fetch_one(&pool)
            .await
            .expect("failed to count blacksmith weapons");
    assert_eq!(
        final_count, 27,
        "count should be 27 after adding new weapon"
    );
}
