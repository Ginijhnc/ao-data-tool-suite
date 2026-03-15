use std::path::PathBuf;
use std::process::Command;

use pretty_assertions::assert_eq;
use sqlx::PgPool;
use tempfile::TempDir;
use testcontainers::ContainerAsync;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

use crate::e2e::entity_configs::SERVER_INI_FIXTURES;

/// Base path for all test fixtures
pub const FIXTURES_DIR: &str = "tests/fixtures";

/// Fixtures for the Dakara C++ server
pub const DAKARA_CPP_FIXTURES: &str = "dakara_cpp";

/// Fixtures for the Alkon VB6 server
pub const ALKON_VB6_FIXTURES: &str = "alkon_vb6";

/// Configuration for entity test helpers.
///
/// Defines which source directories to enable when running tests for a specific entity type.
pub struct EntityTestConfig {
    /// The database table name for this entity type
    pub table_name: &'static str,
    /// The column name used as primary key for data lookups
    pub id_column: &'static str,
    /// Display name for error messages
    pub display_name: &'static str,
    /// Source directory configuration
    pub source_dir: SourceDir,
}

/// Source directory configuration for test entities.
pub enum SourceDir {
    /// Use `DATS_DIR` (most DAT-based entities)
    Dats,
    /// Use `CHARFILE_DIR` (character files)
    Charfile,
    /// Use `MAPS_DIR` (map files)
    Maps,
}

impl EntityTestConfig {
    /// Configures the command with the appropriate source directory environment variables.
    fn configure_source_dir(&self, cmd: &mut Command) {
        let server_ini = test_server_ini_path();

        match self.source_dir {
            SourceDir::Dats => {
                let dats_dir = crate_dir().join("dat");
                cmd.env("DATS_DIR", dats_dir.to_str().unwrap())
                    .env("CHARFILE_DIR", "/nonexistent")
                    .env("MAPS_DIR", "/nonexistent")
                    .env("SERVER_INI_PATH", server_ini.to_str().unwrap());
            }
            SourceDir::Charfile => {
                let charfile_dir = crate_dir().join("Charfile");
                cmd.env("CHARFILE_DIR", charfile_dir.to_str().unwrap())
                    .env("SERVER_INI_PATH", server_ini.to_str().unwrap())
                    .env("DATS_DIR", "/nonexistent")
                    .env("MAPS_DIR", "/nonexistent");
            }
            SourceDir::Maps => {
                let maps_dir = crate_dir().join("Maps");
                cmd.env("MAPS_DIR", maps_dir.to_str().unwrap())
                    .env("DATS_DIR", "/nonexistent")
                    .env("CHARFILE_DIR", "/nonexistent")
                    .env("SERVER_INI_PATH", server_ini.to_str().unwrap());
            }
        }
    }
}

/// Generic helper that verifies the number of entities in the database.
pub async fn verify_entity_count(
    config: &EntityTestConfig,
    expected_count: i64,
) {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port);
    config.configure_source_dir(&mut cmd);
    cmd.env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "ao_data_to_sql terminated with error");

    let query = format!("SELECT COUNT(*) FROM {}", config.table_name);
    let count: (i64,) = sqlx::query_as(&query)
        .fetch_one(&pool)
        .await
        .expect("failed to execute COUNT(*) query");

    assert_eq!(
        count.0, expected_count,
        "Expected {} {} in {} table, found {}",
        expected_count, config.display_name, config.table_name, count.0
    );
}

/// Generic helper that verifies entity data matches a JSON fixture.
///
/// Uses generics to support different primary key types (i32, &str, etc.).
pub async fn verify_entity_data_matches_fixture<T>(
    config: &EntityTestConfig,
    id_value: T,
    fixture_path: PathBuf,
) where
    T: for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>
        + core::fmt::Display
        + Send,
{
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port);
    config.configure_source_dir(&mut cmd);
    cmd.env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "ao_data_to_sql terminated with error");

    let query = format!(
        "SELECT data FROM {} WHERE {} = $1",
        config.table_name, config.id_column
    );
    let (actual_data,): (serde_json::Value,) = sqlx::query_as(&query)
        .bind(id_value)
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| {
            panic!("failed to fetch {} data", config.display_name)
        });

    let fixture_content = std::fs::read_to_string(&fixture_path)
        .expect("failed to read fixture file");
    let expected_data: serde_json::Value =
        serde_json::from_str(&fixture_content)
            .expect("failed to parse fixture JSON");

    assert_eq!(
        actual_data, expected_data,
        "{} data does not match fixture",
        config.display_name
    );
}

fn get_docker_host() -> &'static str {
    if std::env::var("TESTCONTAINERS_RYUK_DISABLED").is_ok() {
        "host.docker.internal"
    } else {
        "localhost"
    }
}

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub async fn setup_test_db() -> (ContainerAsync<Postgres>, PgPool) {
    let container = Postgres::default().start().await.unwrap();
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();
    let host = get_docker_host();

    let url =
        format!("postgres://postgres:postgres@{host}:{host_port}/postgres");
    let pool = PgPool::connect(&url).await.unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    (container, pool)
}

pub fn with_test_db_env(cmd: &mut Command, host_port: u16) -> &mut Command {
    cmd.current_dir(workspace_root())
        .env("DB_HOST", get_docker_host())
        .env("DB_PORT", host_port.to_string())
        .env("DB_NAME", "postgres")
        .env("DB_USER", "postgres")
        .env("DB_PASSWORD", "postgres")
}

/// Configures source directory environment variables for the parser command.
fn configure_parser_env(
    cmd: &mut Command,
    config: &EntityTestConfig,
    temp_source_dir: &std::path::Path,
    last_exec_file: &std::path::Path,
) {
    match config.source_dir {
        SourceDir::Dats => {
            cmd.env("DATS_DIR", temp_source_dir.to_str().unwrap());
        }
        SourceDir::Charfile => {
            cmd.env("CHARFILE_DIR", temp_source_dir.to_str().unwrap());
        }
        SourceDir::Maps => {
            cmd.env("MAPS_DIR", temp_source_dir.to_str().unwrap());
        }
    }
    cmd.env("LAST_EXECUTION_FILE", last_exec_file.to_str().unwrap());
}

/// Runs the parser and verifies initial import succeeded with expected count.
async fn run_initial_import(
    config: &EntityTestConfig,
    host_port: u16,
    temp_source_dir: &std::path::Path,
    last_exec_file: &std::path::Path,
    expected_count: i64,
    pool: &PgPool,
) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port);
    config.configure_source_dir(&mut cmd);
    configure_parser_env(&mut cmd, config, temp_source_dir, last_exec_file);

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "first run failed");

    let query = format!("SELECT COUNT(*) FROM {}", config.table_name);
    let (count,): (i64,) = sqlx::query_as(&query)
        .fetch_one(pool)
        .await
        .expect("failed to count entities");
    assert_eq!(
        count, expected_count,
        "initial import should have {} {}",
        expected_count, config.display_name
    );
}

/// Runs the parser for the second time (after modification) and verifies success.
fn run_reimport(
    config: &EntityTestConfig,
    host_port: u16,
    temp_source_dir: &std::path::Path,
    last_exec_file: &std::path::Path,
) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port);
    config.configure_source_dir(&mut cmd);
    configure_parser_env(&mut cmd, config, temp_source_dir, last_exec_file);

    let status = cmd.status().expect("failed to execute ao_data_to_sql");
    assert!(status.success(), "second run failed");
}

/// Generic helper for testing incremental updates when modifying existing entities.
///
/// This function tests that the parser detects file modifications and re-imports data correctly.
pub async fn test_incremental_modification<F>(
    config: &EntityTestConfig,
    source_subdir: &str,
    dat_filename: &str,
    expected_count: i64,
    search_string: &str,
    replace_string: &str,
    verify_fn: F,
) where
    F: for<'a> FnOnce(
            &'a PgPool,
        ) -> core::pin::Pin<
            Box<dyn core::future::Future<Output = ()> + Send + 'a>,
        > + Send,
{
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_dat_file = temp_dir.path().join(dat_filename);

    let original_dat = crate_dir().join(source_subdir).join(dat_filename);
    std::fs::copy(&original_dat, &temp_dat_file)
        .expect("failed to copy DAT file");

    let temp_source_dir = temp_dir.path();

    run_initial_import(
        config,
        host_port,
        temp_source_dir,
        &last_exec_file,
        expected_count,
        &pool,
    )
    .await;

    std::thread::sleep(core::time::Duration::from_secs(2));

    let content = std::fs::read_to_string(&temp_dat_file)
        .expect("failed to read DAT file");
    let modified_content = content.replace(search_string, replace_string);
    std::fs::write(&temp_dat_file, modified_content)
        .expect("failed to write modified DAT file");

    run_reimport(config, host_port, temp_source_dir, &last_exec_file);

    verify_fn(&pool).await;

    let query = format!("SELECT COUNT(*) FROM {}", config.table_name);
    let (final_count,): (i64,) = sqlx::query_as(&query)
        .fetch_one(&pool)
        .await
        .expect("failed to count entities");
    assert_eq!(
        final_count, expected_count,
        "count should remain {} after modification",
        expected_count
    );
}

/// Verifies that a new entity does not exist before addition.
async fn verify_entity_not_exists<T>(
    config: &EntityTestConfig,
    pool: &PgPool,
    entity_id: T,
) where
    T: for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>
        + core::fmt::Display,
{
    let exists_query = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)",
        config.table_name, config.id_column
    );
    let (exists,): (bool,) = sqlx::query_as(&exists_query)
        .bind(entity_id)
        .fetch_one(pool)
        .await
        .expect("failed to check entity existence");
    assert!(
        !exists,
        "new {} should not exist initially",
        config.display_name
    );
}

/// Verifies that a new entity exists after addition and checks final count.
async fn verify_entity_added<T>(
    config: &EntityTestConfig,
    pool: &PgPool,
    entity_id: T,
    initial_count: i64,
) where
    T: for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>
        + core::fmt::Display,
{
    let exists_query = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)",
        config.table_name, config.id_column
    );
    let (entity_exists,): (bool,) = sqlx::query_as(&exists_query)
        .bind(entity_id)
        .fetch_one(pool)
        .await
        .expect("failed to check entity existence");
    assert!(
        entity_exists,
        "{} should exist after addition",
        config.display_name
    );

    let query = format!("SELECT COUNT(*) FROM {}", config.table_name);
    let expected_final_count = initial_count + 1;
    let (final_count,): (i64,) = sqlx::query_as(&query)
        .fetch_one(pool)
        .await
        .expect("failed to count entities");
    assert_eq!(
        final_count, expected_final_count,
        "count should be {} after adding new {}",
        expected_final_count, config.display_name
    );
}

/// Generic helper for testing incremental updates when adding new entities.
///
/// This function tests that the parser detects new entities added to a file and imports them.
pub async fn test_incremental_addition<T, F>(
    config: &EntityTestConfig,
    source_subdir: &str,
    dat_filename: &str,
    initial_count: i64,
    new_entity_id: T,
    fixture_path: PathBuf,
    verify_fn: F,
) where
    T: for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>
        + core::fmt::Display
        + Clone
        + Send,
    F: for<'a> FnOnce(
            &'a PgPool,
        ) -> core::pin::Pin<
            Box<dyn core::future::Future<Output = ()> + Send + 'a>,
        > + Send,
{
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let last_exec_file = temp_dir.path().join("last_execution");
    let temp_dat_file = temp_dir.path().join(dat_filename);

    let original_dat = crate_dir().join(source_subdir).join(dat_filename);
    std::fs::copy(&original_dat, &temp_dat_file)
        .expect("failed to copy DAT file");

    let temp_source_dir = temp_dir.path();

    run_initial_import(
        config,
        host_port,
        temp_source_dir,
        &last_exec_file,
        initial_count,
        &pool,
    )
    .await;

    verify_entity_not_exists(config, &pool, new_entity_id.clone()).await;

    std::thread::sleep(core::time::Duration::from_secs(2));

    let content = std::fs::read_to_string(&temp_dat_file)
        .expect("failed to read DAT file");
    let new_entity = std::fs::read_to_string(&fixture_path)
        .expect("failed to read fixture");
    let modified_content = format!("{content}\n{new_entity}");
    std::fs::write(&temp_dat_file, modified_content)
        .expect("failed to write modified DAT file");

    run_reimport(config, host_port, temp_source_dir, &last_exec_file);

    verify_fn(&pool).await;

    verify_entity_added(config, &pool, new_entity_id, initial_count).await;
}

/// Returns the path to the test Server.ini fixture.
pub fn test_server_ini_path() -> PathBuf {
    crate_dir()
        .join(FIXTURES_DIR)
        .join(SERVER_INI_FIXTURES)
        .join(ALKON_VB6_FIXTURES)
        .join("Server.ini")
}
