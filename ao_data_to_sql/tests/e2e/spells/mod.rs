//! End-to-end tests for Hechizos.dat parsing and database import.
//!
//! Tests verify spell count, data integrity, and incremental update behavior.

use std::process::Command;

use tempfile::TempDir;

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, setup_test_db,
    with_test_db_env,
};
use crate::e2e::entity_configs::{
    SPELL_FIXTURES, verify_spell_count, verify_spell_data_matches_fixture,
};

/// Verifies the number of spells in the database matches expected count.
#[tokio::test]
async fn spell_count_matches_expected() {
    verify_spell_count(50).await;
}

/// Verifies HECHIZO9 (Paralizar - status effect spell) data matches expected fixture.
#[tokio::test]
async fn spell_9_paralizar_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(SPELL_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("HECHIZO9.json");
    verify_spell_data_matches_fixture(9, fixture).await;
}

/// Verifies HECHIZO36 (Tormenta Pretoriana - damage spell) data matches expected fixture.
#[tokio::test]
async fn spell_36_tormenta_pretoriana_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(SPELL_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("HECHIZO36.json");
    verify_spell_data_matches_fixture(36, fixture).await;
}

/// Verifies incremental update: parser detects modified Hechizos.dat and updates existing spell.
#[tokio::test]
async fn incremental_update_detects_modified_spell() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_spells_dat = temp_dir.path().join("Hechizos.dat");

    // Copy original Hechizos.dat to temp location
    let original_spells = crate_dir().join("dat").join("Hechizos.dat");
    std::fs::copy(&original_spells, &temp_spells_dat)
        .expect("failed to copy Hechizos.dat");

    let temp_dats_dir = temp_dir.path();

    // First run: import all spells
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    // Verify initial count
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM spells")
        .fetch_one(&pool)
        .await
        .expect("failed to count spells");
    assert_eq!(count, 50, "initial import should have 50 spells");

    // Sleep briefly to ensure filesystem mtime resolution
    std::thread::sleep(core::time::Duration::from_secs(2));

    // Modify Hechizos.dat: change HECHIZO1's name
    let spells_content = std::fs::read_to_string(&temp_spells_dat)
        .expect("failed to read Hechizos.dat");
    let modified_content = spells_content
        .replace("Nombre=Antídoto Mágico", "Nombre=Antidoto MODIFICADO");
    std::fs::write(&temp_spells_dat, modified_content)
        .expect("failed to write modified Hechizos.dat");

    // Second run: should detect modification and re-import
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    // Verify HECHIZO1's name was updated
    let (updated_name,): (String,) =
        sqlx::query_as("SELECT name FROM spells WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch HECHIZO1");

    assert_eq!(
        updated_name, "Antidoto MODIFICADO",
        "HECHIZO1 name should be updated after modification"
    );

    // Count should still be 50 (no new spells added)
    let (final_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM spells")
        .fetch_one(&pool)
        .await
        .expect("failed to count spells");
    assert_eq!(final_count, 50, "count should remain 50 after modification");
}

/// Verifies incremental update: parser detects new spell added to Hechizos.dat.
#[tokio::test]
async fn incremental_update_detects_new_spell() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_spells_dat = temp_dir.path().join("Hechizos.dat");

    // Copy original Hechizos.dat to temp location
    let original_spells = crate_dir().join("dat").join("Hechizos.dat");
    std::fs::copy(&original_spells, &temp_spells_dat)
        .expect("failed to copy Hechizos.dat");

    let temp_dats_dir = temp_dir.path();

    // First run: import all spells
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    // Verify initial count
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM spells")
        .fetch_one(&pool)
        .await
        .expect("failed to count spells");
    assert_eq!(count, 50, "initial import should have 50 spells");

    // Verify HECHIZO51 doesn't exist yet
    let (exists,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM spells WHERE id = 51)")
            .fetch_one(&pool)
            .await
            .expect("failed to check HECHIZO51 existence");
    assert!(!exists, "HECHIZO51 should not exist initially");

    // Sleep briefly to ensure filesystem mtime resolution
    std::thread::sleep(core::time::Duration::from_secs(2));

    // Add a new spell to Hechizos.dat
    let spells_content = std::fs::read_to_string(&temp_spells_dat)
        .expect("failed to read Hechizos.dat");
    let new_spell_fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(SPELL_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_spell_51.txt");
    let new_spell = std::fs::read_to_string(&new_spell_fixture)
        .expect("failed to read test HECHIZO51 fixture");
    let modified_content = format!("{spells_content}\n{new_spell}");
    std::fs::write(&temp_spells_dat, modified_content)
        .expect("failed to write modified Hechizos.dat");

    // Second run: should detect modification and re-import
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    // Verify HECHIZO51 now exists
    let (spell_exists,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM spells WHERE id = 51)")
            .fetch_one(&pool)
            .await
            .expect("failed to check HECHIZO51 existence");
    assert!(spell_exists, "HECHIZO51 should exist after addition");

    // Verify HECHIZO51's name
    let (spell_name,): (String,) =
        sqlx::query_as("SELECT name FROM spells WHERE id = 51")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch HECHIZO51");
    assert_eq!(
        spell_name, "Hechizo de prueba incremental",
        "HECHIZO51 should have correct name"
    );

    // Count should now be 51 (1 new spell added)
    let (final_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM spells")
        .fetch_one(&pool)
        .await
        .expect("failed to count spells");
    assert_eq!(final_count, 51, "count should be 51 after adding new spell");
}
