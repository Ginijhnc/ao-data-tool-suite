//! End-to-end tests for ArmadurasHerrero.dat parsing and database import.
//!
//! Tests verify blacksmith armor count, data integrity, and incremental update behavior.

use std::process::Command;

use tempfile::TempDir;

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, setup_test_db,
    test_server_ini_path, with_test_db_env,
};
use crate::e2e::entity_configs::{
    BLACKSMITH_ARMORS_FIXTURES, verify_blacksmith_armor_count,
    verify_blacksmith_armor_data_matches_fixture,
};

/// Verifies the number of blacksmith armors matches expected count.
#[tokio::test]
async fn blacksmith_armor_count_matches_expected() {
    verify_blacksmith_armor_count(61).await;
}

/// Verifies blacksmith armor 1 (Casco de hierro - Index 132) data matches expected fixture.
#[tokio::test]
async fn blacksmith_armor_1_casco_hierro_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("BLACKSMITHARMOR1.json");
    verify_blacksmith_armor_data_matches_fixture(1, fixture).await;
}

/// Verifies blacksmith armor 41 (Anillo Magico - Index 697) data matches expected fixture.
#[tokio::test]
async fn blacksmith_armor_41_anillo_magico_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("BLACKSMITHARMOR41.json");
    verify_blacksmith_armor_data_matches_fixture(41, fixture).await;
}

/// Verifies incremental update: parser detects modified ArmadurasHerrero.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_blacksmith_armor() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_armors_dat = temp_dir.path().join("ArmadurasHerrero.dat");

    let original_armors = crate_dir().join("dat").join("ArmadurasHerrero.dat");
    std::fs::copy(&original_armors, &temp_armors_dat)
        .expect("failed to copy ArmadurasHerrero.dat");

    let temp_dats_dir = temp_dir.path();

    let server_ini = test_server_ini_path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("MAPS_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", server_ini.to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM blacksmith_armors")
            .fetch_one(&pool)
            .await
            .expect("failed to count blacksmith armors");
    assert_eq!(count, 61, "initial import should have 61 blacksmith armors");

    std::thread::sleep(core::time::Duration::from_secs(2));

    let armors_content = std::fs::read_to_string(&temp_armors_dat)
        .expect("failed to read ArmadurasHerrero.dat");
    let modified_content = armors_content.replace(
        "[Armadura1] ' Casco de hierro",
        "[Armadura1] ' Casco de hierro MODIFICADO",
    );
    std::fs::write(&temp_armors_dat, modified_content)
        .expect("failed to write modified ArmadurasHerrero.dat");

    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("MAPS_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", server_ini.to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    let (updated_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM blacksmith_armors WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch blacksmith armor 1");

    let comment = updated_data
        .get("_COMMENT")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert_eq!(
        comment, "Casco de hierro MODIFICADO",
        "Blacksmith armor 1 comment should be updated after modification"
    );

    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM blacksmith_armors")
            .fetch_one(&pool)
            .await
            .expect("failed to count blacksmith armors");
    assert_eq!(final_count, 61, "count should remain 61 after modification");
}

/// Verifies incremental update: parser detects new blacksmith armor added.
#[tokio::test]
async fn incremental_update_detects_new_blacksmith_armor() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_armors_dat = temp_dir.path().join("ArmadurasHerrero.dat");

    let original_armors = crate_dir().join("dat").join("ArmadurasHerrero.dat");
    std::fs::copy(&original_armors, &temp_armors_dat)
        .expect("failed to copy ArmadurasHerrero.dat");

    let temp_dats_dir = temp_dir.path();

    let server_ini = test_server_ini_path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("MAPS_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", server_ini.to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM blacksmith_armors")
            .fetch_one(&pool)
            .await
            .expect("failed to count blacksmith armors");
    assert_eq!(count, 61, "initial import should have 61 blacksmith armors");

    let (exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM blacksmith_armors WHERE id = 62)",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check Armadura62 existence");
    assert!(!exists, "Armadura62 should not exist initially");

    std::thread::sleep(core::time::Duration::from_secs(2));

    let armors_content = std::fs::read_to_string(&temp_armors_dat)
        .expect("failed to read ArmadurasHerrero.dat");
    let new_armor_fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_blacksmith_armor_62.txt");
    let new_armor = std::fs::read_to_string(&new_armor_fixture)
        .expect("failed to read test Armadura62 fixture");
    let modified_content = format!("{armors_content}\n{new_armor}");
    std::fs::write(&temp_armors_dat, modified_content)
        .expect("failed to write modified ArmadurasHerrero.dat");

    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("MAPS_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", server_ini.to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    let (armor_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM blacksmith_armors WHERE id = 62)",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check Armadura62 existence");
    assert!(armor_exists, "Armadura62 should exist after addition");

    let (armor_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM blacksmith_armors WHERE id = 62")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch Armadura62");

    let index = armor_data
        .get("OBJ_INDEX")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(index, "999", "Armadura62 should have correct OBJ_INDEX");

    assert!(
        armor_data.get("_COMMENT").is_none(),
        "Armadura62 should not have _COMMENT field (no comment in source)"
    );

    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM blacksmith_armors")
            .fetch_one(&pool)
            .await
            .expect("failed to count blacksmith armors");
    assert_eq!(final_count, 62, "count should be 62 after adding new armor");
}
