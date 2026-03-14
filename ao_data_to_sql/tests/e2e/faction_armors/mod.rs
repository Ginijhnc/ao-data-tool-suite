//! End-to-end tests for ArmadurasFaccionarias.dat parsing and database import.
//!
//! Tests verify faction armor count, data integrity, and incremental update behavior.

use std::process::Command;

use tempfile::TempDir;

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, setup_test_db,
    with_test_db_env,
};
use crate::e2e::entity_configs::{
    FACTION_ARMORS_FIXTURES, verify_faction_armor_count,
    verify_faction_armor_data_matches_fixture,
};

/// Verifies the number of faction armor classes matches expected count.
#[tokio::test]
async fn faction_armor_count_matches_expected() {
    verify_faction_armor_count(12).await;
}

/// Verifies faction armor 1 (Mago) data matches expected fixture.
#[tokio::test]
async fn faction_armor_1_mago_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(FACTION_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("CLASE1.json");
    verify_faction_armor_data_matches_fixture(1, fixture).await;
}

/// Verifies faction armor 3 (Guerrero) data matches expected fixture.
#[tokio::test]
async fn faction_armor_3_guerrero_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(FACTION_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("CLASE3.json");
    verify_faction_armor_data_matches_fixture(3, fixture).await;
}

/// Verifies incremental update: parser detects modified ArmadurasFaccionarias.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_faction_armor() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_armors_dat = temp_dir.path().join("ArmadurasFaccionarias.dat");

    let original_armors =
        crate_dir().join("dat").join("ArmadurasFaccionarias.dat");
    std::fs::copy(&original_armors, &temp_armors_dat)
        .expect("failed to copy ArmadurasFaccionarias.dat");

    let temp_dats_dir = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM faction_armors")
            .fetch_one(&pool)
            .await
            .expect("failed to count faction armors");
    assert_eq!(count, 12, "initial import should have 12 faction armors");

    std::thread::sleep(core::time::Duration::from_secs(2));

    let armors_content = std::fs::read_to_string(&temp_armors_dat)
        .expect("failed to read ArmadurasFaccionarias.dat");
    let modified_content = armors_content
        .replace("[CLASE3] ' Guerrero", "[CLASE3] ' Guerrero MODIFICADO");
    std::fs::write(&temp_armors_dat, modified_content)
        .expect("failed to write modified ArmadurasFaccionarias.dat");

    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    let (updated_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM faction_armors WHERE id = 3")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch faction armor 3");

    let comment = updated_data
        .get("_COMMENT")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert_eq!(
        comment, "Guerrero MODIFICADO",
        "Faction armor 3 comment should be updated after modification"
    );

    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM faction_armors")
            .fetch_one(&pool)
            .await
            .expect("failed to count faction armors");
    assert_eq!(final_count, 12, "count should remain 12 after modification");
}

/// Verifies incremental update: parser detects new faction armor class added.
#[tokio::test]
async fn incremental_update_detects_new_faction_armor() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_armors_dat = temp_dir.path().join("ArmadurasFaccionarias.dat");

    let original_armors =
        crate_dir().join("dat").join("ArmadurasFaccionarias.dat");
    std::fs::copy(&original_armors, &temp_armors_dat)
        .expect("failed to copy ArmadurasFaccionarias.dat");

    let temp_dats_dir = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM faction_armors")
            .fetch_one(&pool)
            .await
            .expect("failed to count faction armors");
    assert_eq!(count, 12, "initial import should have 12 faction armors");

    let (exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM faction_armors WHERE id = 13)",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check CLASE13 existence");
    assert!(!exists, "CLASE13 should not exist initially");

    std::thread::sleep(core::time::Duration::from_secs(2));

    let armors_content = std::fs::read_to_string(&temp_armors_dat)
        .expect("failed to read ArmadurasFaccionarias.dat");
    let new_armor_fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(FACTION_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_faction_armor_13.txt");
    let new_armor = std::fs::read_to_string(&new_armor_fixture)
        .expect("failed to read test CLASE13 fixture");
    let modified_content = format!("{armors_content}\n{new_armor}");
    std::fs::write(&temp_armors_dat, modified_content)
        .expect("failed to write modified ArmadurasFaccionarias.dat");

    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    let (armor_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM faction_armors WHERE id = 13)",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check CLASE13 existence");
    assert!(armor_exists, "CLASE13 should exist after addition");

    let (armor_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM faction_armors WHERE id = 13")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch CLASE13");

    let def_min = armor_data
        .get("DEFMINARMYALTO")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(def_min, "675", "CLASE13 should have correct DEFMINARMYALTO");

    assert!(
        armor_data.get("_COMMENT").is_none(),
        "CLASE13 should not have _COMMENT field (no comment in source)"
    );

    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM faction_armors")
            .fetch_one(&pool)
            .await
            .expect("failed to count faction armors");
    assert_eq!(final_count, 13, "count should be 13 after adding new class");
}
