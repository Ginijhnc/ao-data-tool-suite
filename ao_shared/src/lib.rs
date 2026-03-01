use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Variable de entorno no definida: {0}")]
    EnvVarMissing(String),
    #[error("Error de conexión: {0}")]
    ConnectionError(#[from] sqlx::Error),
}

pub fn build_db_url() -> Result<String, DbError> {
    let host = std::env::var("DB_HOST").map_err(|_| DbError::EnvVarMissing("DB_HOST".into()))?;
    let port = std::env::var("DB_PORT").map_err(|_| DbError::EnvVarMissing("DB_PORT".into()))?;
    let name = std::env::var("DB_NAME").map_err(|_| DbError::EnvVarMissing("DB_NAME".into()))?;
    let user = std::env::var("DB_USER").map_err(|_| DbError::EnvVarMissing("DB_USER".into()))?;
    let password =
        std::env::var("DB_PASSWORD").map_err(|_| DbError::EnvVarMissing("DB_PASSWORD".into()))?;

    Ok(format!(
        "postgres://{}:{}@{}:{}/{}",
        user, password, host, port, name
    ))
}

pub async fn create_pool(max_connections: u32) -> Result<PgPool, DbError> {
    let url = build_db_url()?;

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&url)
        .await?;

    Ok(pool)
}
