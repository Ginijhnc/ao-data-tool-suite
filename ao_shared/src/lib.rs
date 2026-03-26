//! # `ao_shared`
//!
//! Shared utilities for the AO Tool Suite.
//!
//! This crate provides common database connection management used across
//! all tools in the workspace for interacting with `PostgreSQL`.

#![deny(missing_docs)]

use core::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};
use thiserror::Error;

#[cfg(feature = "testing")]
pub mod testing;

/// Errors that can occur during database operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DbError {
    /// A required environment variable is not defined.
    #[error("Variable de entorno no definida: {0}")]
    EnvVarMissing(String),

    /// A database connection error occurred.
    #[error("Error de conexión: {0}")]
    ConnectionError(#[from] sqlx::Error),
}

/// Constructs a `PostgreSQL` connection URL from environment variables.
///
/// Reads `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, and `DB_PASSWORD` from the
/// environment and assembles them into a connection string.
pub fn build_db_url() -> Result<String, DbError> {
    let host = std::env::var("DB_HOST")
        .map_err(|_| DbError::EnvVarMissing("DB_HOST".into()))?;
    let port = std::env::var("DB_PORT")
        .map_err(|_| DbError::EnvVarMissing("DB_PORT".into()))?;
    let name = std::env::var("DB_NAME")
        .map_err(|_| DbError::EnvVarMissing("DB_NAME".into()))?;
    let user = std::env::var("DB_USER")
        .map_err(|_| DbError::EnvVarMissing("DB_USER".into()))?;
    let password = std::env::var("DB_PASSWORD")
        .map_err(|_| DbError::EnvVarMissing("DB_PASSWORD".into()))?;

    Ok(format!("postgres://{user}:{password}@{host}:{port}/{name}"))
}

/// Creates a `PostgreSQL` connection pool.
///
/// Uses [`build_db_url`] for configuration. Pool has a 30-second acquire timeout.
pub async fn create_pool(max_connections: u32) -> Result<PgPool, DbError> {
    let url = build_db_url()?;

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&url)
        .await?;

    Ok(pool)
}
