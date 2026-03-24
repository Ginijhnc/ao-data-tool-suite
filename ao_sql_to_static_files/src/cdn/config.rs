//! Cloudflare R2 configuration from environment variables.
//!
//! Loads and validates R2 credentials for CDN upload.

use anyhow::{Context, Result, bail};

/// Cloudflare R2 configuration.
///
/// Contains credentials and endpoint information for S3-compatible API.
#[non_exhaustive]
pub struct R2Config {
    /// Cloudflare account ID
    pub account_id: String,
    /// R2 access key ID (API token)
    pub access_key_id: String,
    /// R2 secret access key
    pub secret_access_key: String,
    /// R2 bucket name
    pub bucket_name: String,
    /// R2 endpoint URL
    pub endpoint: String,
}

impl R2Config {
    /// Loads R2 configuration from environment variables.
    ///
    /// Required variables: `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`,
    /// `R2_BUCKET_NAME`, `R2_ENDPOINT`
    pub fn from_env() -> Result<Self> {
        let account_id = std::env::var("R2_ACCOUNT_ID")
            .context("Variable de entorno R2_ACCOUNT_ID no definida")?;

        let access_key_id = std::env::var("R2_ACCESS_KEY_ID")
            .context("Variable de entorno R2_ACCESS_KEY_ID no definida")?;

        let secret_access_key = std::env::var("R2_SECRET_ACCESS_KEY")
            .context("Variable de entorno R2_SECRET_ACCESS_KEY no definida")?;

        let bucket_name = std::env::var("R2_BUCKET_NAME")
            .context("Variable de entorno R2_BUCKET_NAME no definida")?;

        let endpoint = std::env::var("R2_ENDPOINT")
            .context("Variable de entorno R2_ENDPOINT no definida")?;

        // Validate non-empty values
        if account_id.trim().is_empty() {
            bail!("R2_ACCOUNT_ID no puede estar vacío");
        }
        if access_key_id.trim().is_empty() {
            bail!("R2_ACCESS_KEY_ID no puede estar vacío");
        }
        if secret_access_key.trim().is_empty() {
            bail!("R2_SECRET_ACCESS_KEY no puede estar vacío");
        }
        if bucket_name.trim().is_empty() {
            bail!("R2_BUCKET_NAME no puede estar vacío");
        }
        if endpoint.trim().is_empty() {
            bail!("R2_ENDPOINT no puede estar vacío");
        }

        Ok(Self {
            account_id,
            access_key_id,
            secret_access_key,
            bucket_name,
            endpoint,
        })
    }
}
