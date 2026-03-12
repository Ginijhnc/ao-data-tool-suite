//! Helper functions for balance E2E tests.
//!
//! Provides utilities to verify balance section count and data against fixtures.

use std::path::PathBuf;
use std::process::Command;

use pretty_assertions::assert_eq;
use tempfile::TempDir;

use crate::common::{crate_dir, setup_test_db, with_test_db_env};

/// Fixture subdirectory for balance sections.
pub const BALANCE_FIXTURES: &str = "balance";

/// Helper that verifies the number of balance sections in the database.
pub async fn verify_balance_count(expected_count: i64) {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");

    let dats_dir = crate_dir().join("dat");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "ao_data_to_sql terminated with error");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM balance")
        .fetch_one(&pool)
        .await
        .expect("failed to execute COUNT(*) query");

    assert_eq!(
        count.0, expected_count,
        "Expected {} balance sections, found {}",
        expected_count, count.0
    );
}

/// Helper that verifies balance section data matches a JSON fixture.
pub async fn verify_balance_data_matches_fixture(
    section: &str,
    fixture_path: PathBuf,
) {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");

    let dats_dir = crate_dir().join("dat");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("DATS_DIR", dats_dir.to_str().unwrap())
        .env("CHARFILE_DIR", "/nonexistent")
        .env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "ao_data_to_sql terminated with error");

    let (actual_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM balance WHERE section = $1")
            .bind(section)
            .fetch_one(&pool)
            .await
            .expect("failed to fetch balance section data");

    let fixture_content = std::fs::read_to_string(&fixture_path)
        .expect("failed to read fixture file");
    let expected_data: serde_json::Value =
        serde_json::from_str(&fixture_content)
            .expect("failed to parse fixture JSON");

    assert_eq!(
        actual_data, expected_data,
        "Balance section {section} data does not match fixture"
    );
}
