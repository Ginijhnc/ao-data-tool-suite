use std::process::Command;

use tempfile::TempDir;

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, setup_test_db,
    test_server_ini_path, with_test_db_env,
};
use crate::e2e::entity_configs::{
    OBJECT_FIXTURES as OBJ_FIXTURES, verify_object_count,
    verify_object_data_matches_fixture,
};

/// Verifies the number of objects in the database matches expected count.
#[tokio::test]
async fn object_count_matches_expected() {
    verify_object_count(1122).await;
}

/// Verifies OBJ1 (Manzana Roja - food item) data matches expected fixture.
#[tokio::test]
async fn object_1_manzana_roja_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(OBJ_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("OBJ1.json");
    verify_object_data_matches_fixture(1, fixture).await;
}

/// Verifies OBJ129 (Hacha de Guerra Dos Filos - weapon with class prohibitions) data matches expected fixture.
#[tokio::test]
async fn object_129_hacha_guerra_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(OBJ_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("OBJ129.json");
    verify_object_data_matches_fixture(129, fixture).await;
}

/// Verifies OBJ142 (Puerta de madera cerrada - door with key/lock properties) data matches expected fixture.
#[tokio::test]
async fn object_142_puerta_madera_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(OBJ_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("OBJ142.json");
    verify_object_data_matches_fixture(142, fixture).await;
}

/// Verifies incremental update: parser detects modified Obj.dat and updates existing object.
#[tokio::test]
async fn incremental_update_detects_modified_object() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_obj_dat = temp_dir.path().join("Obj.dat");

    // Copy original Obj.dat to temp location
    let original_obj = crate_dir().join("dat").join("Obj.dat");
    std::fs::copy(&original_obj, &temp_obj_dat)
        .expect("failed to copy Obj.dat");

    let temp_dats_dir = temp_dir.path();

    let server_ini = test_server_ini_path();

    // First run: import all objects
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("MAPS_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", server_ini.to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    // Verify initial count
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM objects")
        .fetch_one(&pool)
        .await
        .expect("failed to count objects");
    assert_eq!(count, 1122, "initial import should have 1122 objects");

    // Sleep briefly to ensure filesystem mtime resolution
    std::thread::sleep(core::time::Duration::from_secs(2));

    // Modify Obj.dat: change OBJ1's name
    let obj_content = std::fs::read_to_string(&temp_obj_dat)
        .expect("failed to read Obj.dat");
    let modified_content =
        obj_content.replace("Name=Manzana Roja", "Name=Manzana MODIFICADA");
    std::fs::write(&temp_obj_dat, modified_content)
        .expect("failed to write modified Obj.dat");

    // Second run: should detect modification and re-import
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("MAPS_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", server_ini.to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    // Verify OBJ1's name was updated
    let (updated_name,): (String,) =
        sqlx::query_as("SELECT name FROM objects WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch OBJ1");

    assert_eq!(
        updated_name, "Manzana MODIFICADA",
        "OBJ1 name should be updated after modification"
    );

    // Count should still be 1125 (no new objects added)
    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM objects")
            .fetch_one(&pool)
            .await
            .expect("failed to count objects");
    assert_eq!(
        final_count, 1122,
        "count should remain 1122 after modification"
    );
}

/// Verifies incremental update: parser detects new object added to Obj.dat.
#[tokio::test]
async fn incremental_update_detects_new_object() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_obj_dat = temp_dir.path().join("Obj.dat");

    // Copy original Obj.dat to temp location
    let original_obj = crate_dir().join("dat").join("Obj.dat");
    std::fs::copy(&original_obj, &temp_obj_dat)
        .expect("failed to copy Obj.dat");

    let temp_dats_dir = temp_dir.path();

    let server_ini = test_server_ini_path();

    // First run: import all objects
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("MAPS_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", server_ini.to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    // Verify initial count
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM objects")
        .fetch_one(&pool)
        .await
        .expect("failed to count objects");
    assert_eq!(count, 1122, "initial import should have 1122 objects");

    // Verify OBJ1126 doesn't exist yet
    let (exists,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM objects WHERE id = 1126)")
            .fetch_one(&pool)
            .await
            .expect("failed to check OBJ1126 existence");
    assert!(!exists, "OBJ1126 should not exist initially");

    // Sleep briefly to ensure filesystem mtime resolution
    std::thread::sleep(core::time::Duration::from_secs(2));

    // Add a new object to Obj.dat
    let obj_content = std::fs::read_to_string(&temp_obj_dat)
        .expect("failed to read Obj.dat");
    let new_obj_fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(OBJ_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_obj_1126.txt");
    let new_obj = std::fs::read_to_string(&new_obj_fixture)
        .expect("failed to read test OBJ1126 fixture");
    let modified_content = format!("{obj_content}\n{new_obj}");
    std::fs::write(&temp_obj_dat, modified_content)
        .expect("failed to write modified Obj.dat");

    // Second run: should detect modification and re-import
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd2, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("MAPS_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", server_ini.to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status2 = cmd2.status().expect("failed to execute ao_data_to_sql");
    assert!(status2.success(), "second run failed");

    // Verify OBJ1126 now exists
    let (obj_exists,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM objects WHERE id = 1126)")
            .fetch_one(&pool)
            .await
            .expect("failed to check OBJ1126 existence");
    assert!(obj_exists, "OBJ1126 should exist after addition");

    // Verify OBJ1126's name
    let (obj_name,): (String,) =
        sqlx::query_as("SELECT name FROM objects WHERE id = 1126")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch OBJ1126");
    assert_eq!(
        obj_name, "Teleport a Dungeon Newbie",
        "OBJ1126 should have correct name"
    );

    // Count should now be 1126 (1 new object added)
    let (final_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM objects")
            .fetch_one(&pool)
            .await
            .expect("failed to count objects");
    assert_eq!(
        final_count, 1123,
        "count should be 1123 after adding new object"
    );
}
