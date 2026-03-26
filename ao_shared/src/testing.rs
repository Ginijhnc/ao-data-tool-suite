//! Shared test infrastructure for E2E tests across the workspace.
//!
//! Provides `PostgreSQL` container setup via testcontainers and
//! environment variable configuration for running crate binaries in tests.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Test infrastructure intentionally panics on setup failures"
)]

use std::process::Command;

use sqlx::PgPool;
use testcontainers::ContainerAsync;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

/// Path to the shared migrations directory, relative to the workspace root.
pub const MIGRATIONS_PATH: &str = "../ao_shared/migrations";

/// Returns the Docker host address.
///
/// Uses `host.docker.internal` when Ryuk is disabled (Docker-in-Docker),
/// otherwise uses `localhost`.
#[must_use]
pub fn get_docker_host() -> &'static str {
    if std::env::var("TESTCONTAINERS_RYUK_DISABLED").is_ok() {
        "host.docker.internal"
    } else {
        "localhost"
    }
}

/// Spawns a `PostgreSQL` container and runs workspace migrations.
///
/// Returns the container (must be kept alive) and a connected pool.
pub async fn setup_test_db() -> (ContainerAsync<Postgres>, PgPool) {
    let container = Postgres::default().start().await.unwrap();
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();
    let host = get_docker_host();

    let url =
        format!("postgres://postgres:postgres@{host}:{host_port}/postgres");
    let pool = PgPool::connect(&url).await.unwrap();

    sqlx::migrate::Migrator::new(std::path::Path::new(MIGRATIONS_PATH))
        .await
        .expect("Failed to load migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    (container, pool)
}

/// Configures a Command with database environment variables for test containers.
///
/// Sets `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, and `DB_PASSWORD`.
pub fn with_test_db_env(cmd: &mut Command, host_port: u16) -> &mut Command {
    cmd.env("DB_HOST", get_docker_host())
        .env("DB_PORT", host_port.to_string())
        .env("DB_NAME", "postgres")
        .env("DB_USER", "postgres")
        .env("DB_PASSWORD", "postgres")
}
