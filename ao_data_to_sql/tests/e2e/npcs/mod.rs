use std::process::Command;

use tempfile::TempDir;

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, setup_test_db,
    test_server_ini_path, with_test_db_env,
};
use crate::e2e::entity_configs::{
    NPC_FIXTURES, verify_npc_count, verify_npc_data_matches_fixture,
};

/// Verifies the number of NPCs in the database matches expected count.
#[tokio::test]
async fn npc_count_matches_expected() {
    verify_npc_count(337).await;
}

/// Verifies NPC138 (Abel - non-interactive decorative NPC) data matches expected fixture
#[tokio::test]
async fn npc_138_abel_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(NPC_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("NPC138.json");
    verify_npc_data_matches_fixture(138, fixture).await;
}

/// Verifies NPC61 (Mago - spell merchant) data matches expected fixture
#[tokio::test]
async fn npc_61_mago_spell_merchant_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(NPC_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("NPC61.json");
    verify_npc_data_matches_fixture(61, fixture).await;
}

/// Verifies NPC517 (Djinn de Viento - Hostile NPC - no 'comment after [NPC517]) data matches expected fixture
#[tokio::test]
async fn npc_517_djinn_hostile_npc_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(NPC_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("NPC517.json");
    verify_npc_data_matches_fixture(517, fixture).await;
}

/// Verifies NPC61 with `STRIP_DAT_INLINE_COMMENTS=false` preserves comments
#[tokio::test]
async fn npc_61_preserves_inline_comments_when_disabled() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(NPC_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("NPC61_with_comments.json");

    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_npcs_dat = temp_dir.path().join("NPCs.dat");

    // Copy original NPCs.dat to temp location
    let original_npcs = crate_dir().join("dat").join("NPCs.dat");
    std::fs::copy(&original_npcs, &temp_npcs_dat)
        .expect("failed to copy NPCs.dat");

    let temp_dats_dir = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("STRIP_DAT_INLINE_COMMENTS", "false")
        .env("CHARFILE_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", test_server_ini_path().to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "import failed");

    let (data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM npcs WHERE id = 61")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch NPC61");

    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&fixture).unwrap())
            .expect("failed to parse fixture");

    assert_eq!(
        data, expected,
        "NPC 61 data does not match fixture with comments preserved"
    );
}

/// Verifies NPC517 with `STRIP_DAT_INLINE_COMMENTS=false` preserves comments
#[tokio::test]
async fn npc_517_preserves_inline_comments_when_disabled() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(NPC_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("NPC517_with_comments.json");

    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_npcs_dat = temp_dir.path().join("NPCs.dat");

    // Copy original NPCs.dat to temp location
    let original_npcs = crate_dir().join("dat").join("NPCs.dat");
    std::fs::copy(&original_npcs, &temp_npcs_dat)
        .expect("failed to copy NPCs.dat");

    let temp_dats_dir = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", temp_dats_dir.to_str().unwrap())
        .env("STRIP_DAT_INLINE_COMMENTS", "false")
        .env("CHARFILE_DIR", "/nonexistent")
        .env("SERVER_INI_PATH", test_server_ini_path().to_str().unwrap())
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "import failed");

    let (data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM npcs WHERE id = 517")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch NPC517");

    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&fixture).unwrap())
            .expect("failed to parse fixture");

    assert_eq!(
        data, expected,
        "NPC 517 data does not match fixture with comments preserved"
    );
}

/// Verifies incremental update: parser detects modified NPCs.dat and updates existing NPC
#[tokio::test]
async fn incremental_update_detects_modified_npc() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_npcs_dat = temp_dir.path().join("NPCs.dat");

    // Copy original NPCs.dat to temp location
    let original_npcs = crate_dir().join("dat").join("NPCs.dat");
    std::fs::copy(&original_npcs, &temp_npcs_dat)
        .expect("failed to copy NPCs.dat");

    let temp_dats_dir = temp_dir.path();

    let server_ini = test_server_ini_path();

    // First run: import all NPCs
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
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM npcs")
        .fetch_one(&pool)
        .await
        .expect("failed to count NPCs");
    assert_eq!(count, 337, "initial import should have 337 NPCs");

    // Sleep briefly to ensure filesystem mtime resolution
    std::thread::sleep(core::time::Duration::from_secs(2));

    // Modify NPCs.dat: change NPC138's name
    let npcs_content = std::fs::read_to_string(&temp_npcs_dat)
        .expect("failed to read NPCs.dat");
    let modified_content = npcs_content
        .replace("Name=Abel, el monje pensador", "Name=Abel MODIFICADO");
    std::fs::write(&temp_npcs_dat, modified_content)
        .expect("failed to write modified NPCs.dat");

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

    // Verify NPC138's name was updated
    let (updated_name,): (String,) =
        sqlx::query_as("SELECT name FROM npcs WHERE id = 138")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch NPC138");

    assert_eq!(
        updated_name, "Abel MODIFICADO",
        "NPC138 name should be updated after modification"
    );

    // Count should still be 337 (no new NPCs added)
    let (final_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM npcs")
        .fetch_one(&pool)
        .await
        .expect("failed to count NPCs");
    assert_eq!(
        final_count, 337,
        "count should remain 337 after modification"
    );
}

/// Verifies incremental update: parser detects new NPC added to NPCs.dat
#[tokio::test]
async fn incremental_update_detects_new_npc() {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_npcs_dat = temp_dir.path().join("NPCs.dat");

    // Copy original NPCs.dat to temp location
    let original_npcs = crate_dir().join("dat").join("NPCs.dat");
    std::fs::copy(&original_npcs, &temp_npcs_dat)
        .expect("failed to copy NPCs.dat");

    let temp_dats_dir = temp_dir.path();

    let server_ini = test_server_ini_path();

    // First run: import all NPCs
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
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM npcs")
        .fetch_one(&pool)
        .await
        .expect("failed to count NPCs");
    assert_eq!(count, 337, "initial import should have 337 NPCs");

    // Verify NPC924 doesn't exist yet
    let (exists,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM npcs WHERE id = 924)")
            .fetch_one(&pool)
            .await
            .expect("failed to check NPC924 existence");
    assert!(!exists, "NPC924 should not exist initially");

    // Sleep briefly to ensure filesystem mtime resolution
    std::thread::sleep(core::time::Duration::from_secs(2));

    // Add a new NPC to NPCs.dat
    let npcs_content = std::fs::read_to_string(&temp_npcs_dat)
        .expect("failed to read NPCs.dat");
    let new_npc_fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(NPC_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_npc_924.txt");
    let new_npc = std::fs::read_to_string(&new_npc_fixture)
        .expect("failed to read test NPC924 fixture");
    let modified_content = format!("{npcs_content}{new_npc}");
    std::fs::write(&temp_npcs_dat, modified_content)
        .expect("failed to write modified NPCs.dat");

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

    // Verify NPC924 now exists
    let (npc_exists,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM npcs WHERE id = 924)")
            .fetch_one(&pool)
            .await
            .expect("failed to check NPC924 existence");
    assert!(npc_exists, "NPC924 should exist after addition");

    // Verify NPC924's name
    let (npc_name,): (String,) =
        sqlx::query_as("SELECT name FROM npcs WHERE id = 924")
            .fetch_one(&pool)
            .await
            .expect("failed to fetch NPC924");
    assert_eq!(
        npc_name, "NPC de prueba incremental",
        "NPC924 should have correct name"
    );

    // Count should now be 338 (1 new NPC added)
    let (final_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM npcs")
        .fetch_one(&pool)
        .await
        .expect("failed to count NPCs");
    assert_eq!(final_count, 338, "count should be 338 after adding new NPC");
}
