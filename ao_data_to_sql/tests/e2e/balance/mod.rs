//! End-to-end tests for Balance.dat parsing and database import.
//!
//! Tests verify balance section count, data integrity, and incremental update behavior.

use std::process::Command;

use tempfile::TempDir;

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, setup_test_db,
    with_test_db_env,
};
use crate::e2e::entity_configs::{
    BALANCE_FIXTURES, verify_balance_count,
    verify_balance_data_matches_fixture,
};

/// Verifies the number of balance sections matches expected count.
#[tokio::test]
async fn balance_count_matches_expected() {
    verify_balance_count(14).await;
}

/// Verifies MODEVASION section data matches expected fixture.
#[tokio::test]
async fn balance_modevasion_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("MODEVASION.json");
    verify_balance_data_matches_fixture("MODEVASION", fixture).await;
}

/// Verifies DISTRIBUCION section data matches expected fixture.
#[tokio::test]
async fn balance_distribucion_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("DISTRIBUCION.json");
    verify_balance_data_matches_fixture("DISTRIBUCION", fixture).await;
}

/// Verifies PARTY section data matches expected fixture.
#[tokio::test]
async fn balance_party_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("PARTY.json");
    verify_balance_data_matches_fixture("PARTY", fixture).await;
}

/// Verifies RECOMPENSAFACCION section data matches expected fixture.
#[tokio::test]
async fn balance_recompensafaccion_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("RECOMPENSAFACCION.json");
    verify_balance_data_matches_fixture("RECOMPENSAFACCION", fixture).await;
}

/// Verifies incremental update: parser detects modified Balance.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_balance_section() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_balance_dat = temp_dir.path().join("Balance.dat");

    let original_balance = crate_dir().join("dat").join("Balance.dat");
    std::fs::copy(&original_balance, &temp_balance_dat)
        .expect("failed to copy Balance.dat");

    let temp_dats_dir = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM balance")
        .fetch_one(&pool)
        .await
        .expect("failed to count balance sections");
    assert_eq!(count, 14, "initial import should have 14 balance sections");

    std::thread::sleep(core::time::Duration::from_secs(2));

    let balance_content = std::fs::read_to_string(&temp_balance_dat)
        .expect("failed to read Balance.dat");
    let modified_content = balance_content
        .replace("[MODEVASION]", "[MODEVASION] ' Balance Modificado");
    std::fs::write(&temp_balance_dat, modified_content)
        .expect("failed to write modified Balance.dat");

    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    let (updated_data,): (serde_json::Value,) = sqlx::query_as(
        "SELECT data FROM balance WHERE section = 'MODEVASION'",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to fetch MODEVASION section");

    let comment = updated_data
        .get("_COMMENT")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert_eq!(
        comment, "Balance Modificado",
        "MODEVASION comment should be updated after modification"
    );

    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM balance")
            .fetch_one(&pool)
            .await
            .expect("failed to count balance sections");
    assert_eq!(final_count, 14, "count should remain 14 after modification");
}

/// Verifies incremental update: parser detects new balance section added.
#[tokio::test]
async fn incremental_update_detects_new_balance_section() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_balance_dat = temp_dir.path().join("Balance.dat");

    let original_balance = crate_dir().join("dat").join("Balance.dat");
    std::fs::copy(&original_balance, &temp_balance_dat)
        .expect("failed to copy Balance.dat");

    let temp_dats_dir = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM balance")
        .fetch_one(&pool)
        .await
        .expect("failed to count balance sections");
    assert_eq!(count, 14, "initial import should have 14 balance sections");

    let (exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM balance WHERE section = 'MODREGENERACION')",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check MODREGENERACION existence");
    assert!(!exists, "MODREGENERACION should not exist initially");

    std::thread::sleep(core::time::Duration::from_secs(2));

    let balance_content = std::fs::read_to_string(&temp_balance_dat)
        .expect("failed to read Balance.dat");
    let new_section_fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_balance_modregeneracion.txt");
    let new_section = std::fs::read_to_string(&new_section_fixture)
        .expect("failed to read test MODREGENERACION fixture");
    let modified_content = format!("{balance_content}\n{new_section}");
    std::fs::write(&temp_balance_dat, modified_content)
        .expect("failed to write modified Balance.dat");

    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    let (section_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM balance WHERE section = 'MODREGENERACION')",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check MODREGENERACION existence");
    assert!(
        section_exists,
        "MODREGENERACION should exist after addition"
    );

    let (section_data,): (serde_json::Value,) = sqlx::query_as(
        "SELECT data FROM balance WHERE section = 'MODREGENERACION'",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to fetch MODREGENERACION");

    let guerrero = section_data
        .get("GUERRERO")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(
        guerrero, "1.05",
        "MODREGENERACION should have correct GUERRERO value"
    );

    let comment = section_data
        .get("_COMMENT")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(
        comment, "Regeneracion De Prueba",
        "MODREGENERACION should have correct _COMMENT"
    );

    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM balance")
            .fetch_one(&pool)
            .await
            .expect("failed to count balance sections");
    assert_eq!(
        final_count, 15,
        "count should be 15 after adding new section"
    );
}
