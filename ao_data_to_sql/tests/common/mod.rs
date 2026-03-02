use std::path::PathBuf;
use std::process::Command;

use sqlx::PgPool;
use testcontainers::ContainerAsync;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

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
