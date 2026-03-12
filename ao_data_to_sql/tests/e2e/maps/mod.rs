//! End-to-end tests for map .dat file parsing and database import.
//!
//! Tests verify map count, data integrity, and incremental update behavior.

mod helpers;

use std::fs;
use std::process::Command;

use tempfile::TempDir;

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, setup_test_db,
    with_test_db_env,
};
use helpers::{
    MAP_FIXTURES, verify_map_count, verify_map_data_matches_fixture,
};

/// Verifies the number of maps matches expected count (5 test maps).
#[tokio::test]
async fn map_count_matches_expected() {
    verify_map_count(5).await;
}

/// Verifies MAPA1 data matches expected fixture.
#[tokio::test]
async fn map1_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(MAP_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("MAPA1.json");
    verify_map_data_matches_fixture(1, fixture).await;
}

/// Verifies MAPA113 (with sounds) data matches expected fixture.
#[tokio::test]
async fn map113_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(MAP_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("MAPA113.json");
    verify_map_data_matches_fixture(113, fixture).await;
}

/// Verifies MAPA167 (with uppercase PK) data matches expected fixture.
#[tokio::test]
async fn map167_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(MAP_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("MAPA167.json");
    verify_map_data_matches_fixture(167, fixture).await;
}

/// Verifies incremental update: parser detects modified map and updates.
#[tokio::test]
async fn incremental_update_detects_modified_map() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_maps_dir = temp_dir.path().join("Maps");
    fs::create_dir(&temp_maps_dir).expect("failed to create Maps dir");

    let original_map = crate_dir().join("Maps").join("mapa1.dat");
    let temp_map = temp_maps_dir.join("mapa1.dat");
    fs::copy(&original_map, &temp_map).expect("failed to copy mapa1.dat");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("MAPS_DIR", temp_maps_dir.to_str().unwrap())
        .env("DATS_DIR", "/nonexistent")
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM maps")
        .fetch_one(&pool)
        .await
        .expect("failed to count maps");
    assert_eq!(count, 1, "initial import should have 1 map");

    #[allow(
        clippy::std_instead_of_core,
        clippy::std_instead_of_alloc,
        reason = "std used for clarity in tests"
    )]
    {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let modified_map_fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(MAP_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_mapa1_modified.txt");
    let modified_content = std::fs::read_to_string(&modified_map_fixture)
        .expect("failed to read test_mapa1_modified.txt fixture");
    fs::write(&temp_map, modified_content)
        .expect("failed to modify mapa1.dat");

    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("MAPS_DIR", temp_maps_dir.to_str().unwrap())
        .env("DATS_DIR", "/nonexistent")
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute second run");
    assert!(status2.success(), "second run failed");

    let (name,): (String,) =
        sqlx::query_as("SELECT name FROM maps WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch map name");
    assert_eq!(name, "Modified Map", "map name should be updated");
}

/// Verifies parser skips non-map files in Maps directory.
#[tokio::test]
async fn parser_ignores_non_map_files() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_maps_dir = temp_dir.path().join("Maps");
    fs::create_dir(&temp_maps_dir).expect("failed to create Maps dir");

    let original_map = crate_dir().join("Maps").join("mapa1.dat");
    let temp_map = temp_maps_dir.join("mapa1.dat");
    fs::copy(&original_map, &temp_map).expect("failed to copy mapa1.dat");

    fs::write(temp_maps_dir.join("readme.txt"), "test file")
        .expect("failed to create readme.txt");
    fs::write(temp_maps_dir.join("config.ini"), "[CONFIG]\ntest=1")
        .expect("failed to create config.ini");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("MAPS_DIR", temp_maps_dir.to_str().unwrap())
        .env("DATS_DIR", "/nonexistent")
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "run failed");

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM maps")
        .fetch_one(&pool)
        .await
        .expect("failed to count maps");
    assert_eq!(count, 1, "should only import 1 map file");
}
