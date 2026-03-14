//! End-to-end tests for ObjCarpintero.dat parsing and database import.
//!
//! Tests verify carpenter object count, data integrity, and incremental update behavior.

use std::process::Command;

use tempfile::TempDir;

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, setup_test_db,
    with_test_db_env,
};
use crate::e2e::entity_configs::{
    CARPENTER_FIXTURES, verify_carpenter_object_count,
    verify_carpenter_object_data_matches_fixture,
};

/// Verifies the number of carpenter objects matches expected count.
#[tokio::test]
async fn carpenter_object_count_matches_expected() {
    verify_carpenter_object_count(34).await;
}

/// Verifies carpenter object 1 (Flecha - Index 480) data matches expected fixture.
#[tokio::test]
async fn carpenter_object_1_flecha_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(CARPENTER_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("CARPENTEROBJ1.json");
    verify_carpenter_object_data_matches_fixture(1, fixture).await;
}

/// Verifies carpenter object 10 (Arco de Cazador - Index 665) data matches expected fixture.
#[tokio::test]
async fn carpenter_object_10_arco_cazador_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(CARPENTER_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("CARPENTEROBJ10.json");
    verify_carpenter_object_data_matches_fixture(10, fixture).await;
}

/// Verifies incremental update: parser detects modified ObjCarpintero.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_carpenter_object() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_carpenter_dat = temp_dir.path().join("ObjCarpintero.dat");

    // Copy original ObjCarpintero.dat to temp location
    let original_carpenter = crate_dir().join("dat").join("ObjCarpintero.dat");
    std::fs::copy(&original_carpenter, &temp_carpenter_dat)
        .expect("failed to copy ObjCarpintero.dat");

    let temp_dats_dir = temp_dir.path();

    // First run: import all carpenter objects
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    // Verify initial count
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM carpenter_objects")
            .fetch_one(&pool)
            .await
            .expect("failed to count carpenter objects");
    assert_eq!(count, 34, "initial import should have 34 carpenter objects");

    // Sleep briefly to ensure filesystem mtime resolution
    std::thread::sleep(core::time::Duration::from_secs(2));

    // Modify ObjCarpintero.dat: change OBJ1's comment
    let carpenter_content = std::fs::read_to_string(&temp_carpenter_dat)
        .expect("failed to read ObjCarpintero.dat");
    let modified_content = carpenter_content
        .replace("[OBJ1] 'Flecha", "[OBJ1] 'Flecha MODIFICADA");
    std::fs::write(&temp_carpenter_dat, modified_content)
        .expect("failed to write modified ObjCarpintero.dat");

    // Second run: should detect modification and re-import
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    // Verify OBJ1's comment was updated
    let (updated_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM carpenter_objects WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch carpenter object 1");

    let comment = updated_data
        .get("_COMMENT")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert_eq!(
        comment, "Flecha MODIFICADA",
        "Carpenter object 1 comment should be updated after modification"
    );

    // Count should still be 34 (no new objects added)
    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM carpenter_objects")
            .fetch_one(&pool)
            .await
            .expect("failed to count carpenter objects");
    assert_eq!(final_count, 34, "count should remain 34 after modification");
}

/// Verifies incremental update: parser detects new carpenter object added.
#[tokio::test]
async fn incremental_update_detects_new_carpenter_object() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_carpenter_dat = temp_dir.path().join("ObjCarpintero.dat");

    // Copy original ObjCarpintero.dat to temp location
    let original_carpenter = crate_dir().join("dat").join("ObjCarpintero.dat");
    std::fs::copy(&original_carpenter, &temp_carpenter_dat)
        .expect("failed to copy ObjCarpintero.dat");

    let temp_dats_dir = temp_dir.path();

    // First run: import all carpenter objects
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    // Verify initial count
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM carpenter_objects")
            .fetch_one(&pool)
            .await
            .expect("failed to count carpenter objects");
    assert_eq!(count, 34, "initial import should have 34 carpenter objects");

    // Verify OBJ35 doesn't exist yet
    let (exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM carpenter_objects WHERE id = 35)",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check OBJ35 existence");
    assert!(!exists, "OBJ35 should not exist initially");

    // Sleep briefly to ensure filesystem mtime resolution
    std::thread::sleep(core::time::Duration::from_secs(2));

    // Add a new carpenter object to ObjCarpintero.dat
    let carpenter_content = std::fs::read_to_string(&temp_carpenter_dat)
        .expect("failed to read ObjCarpintero.dat");
    let new_object_fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(CARPENTER_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_carpenter_obj_35.txt");
    let new_object = std::fs::read_to_string(&new_object_fixture)
        .expect("failed to read test OBJ35 fixture");
    let modified_content = format!("{carpenter_content}\n{new_object}");
    std::fs::write(&temp_carpenter_dat, modified_content)
        .expect("failed to write modified ObjCarpintero.dat");

    // Second run: should detect modification and re-import
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    // Verify OBJ35 now exists
    let (obj_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM carpenter_objects WHERE id = 35)",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check OBJ35 existence");
    assert!(obj_exists, "OBJ35 should exist after addition");

    // Verify OBJ35's data
    let (obj_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM carpenter_objects WHERE id = 35")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch OBJ35");

    let index = obj_data
        .get("OBJ_INDEX")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(index, "999", "OBJ35 should have correct OBJ_INDEX");

    // Count should now be 35 (1 new object added)
    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM carpenter_objects")
            .fetch_one(&pool)
            .await
            .expect("failed to count carpenter objects");
    assert_eq!(
        final_count, 35,
        "count should be 35 after adding new object"
    );
}
