//! Cloudflare R2 client for uploading files.
//!
//! Provides S3-compatible upload functionality with content-type detection.

use anyhow::{Context, Result};
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Region, SharedCredentialsProvider};
use aws_sdk_s3::primitives::ByteStream;

use super::config::R2Config;

/// R2 client for CDN uploads.
///
/// Wraps AWS S3 SDK configured for Cloudflare R2.
pub struct R2Client {
    /// S3 client
    client: Client,
    /// R2 bucket name
    bucket_name: String,
}

impl R2Client {
    /// Creates a new R2 client from configuration.
    ///
    /// Initializes S3 client with R2 endpoint and credentials.
    pub fn new(config: &R2Config) -> Result<Self> {
        // Create static credentials
        let credentials = Credentials::new(
            &config.access_key_id,
            &config.secret_access_key,
            None,
            None,
            "r2-static-credentials",
        );

        // Configure S3 client for R2
        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .endpoint_url(&config.endpoint)
            .region(Region::new("auto"))
            .credentials_provider(SharedCredentialsProvider::new(credentials))
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket_name: config.bucket_name.clone(),
        })
    }

    /// Uploads raw bytes to R2 with specified content type.
    ///
    /// Generic upload method for any file format.
    pub async fn upload(
        &self,
        key: &str,
        content: Vec<u8>,
        content_type: &str,
    ) -> Result<()> {
        let byte_stream = ByteStream::from(content);

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(byte_stream)
            .content_type(content_type)
            .cache_control("public, max-age=1800")
            .send()
            .await
            .with_context(|| format!("Error al subir archivo a R2: {key}"))?;

        tracing::info!("Archivo subido a R2: {key}");

        Ok(())
    }

    /// Uploads JSON content to R2.
    ///
    /// Convenience wrapper for JSON uploads.
    pub async fn upload_json(
        &self,
        key: &str,
        json_content: String,
    ) -> Result<()> {
        self.upload(key, json_content.into_bytes(), "application/json")
            .await
    }

    /// Uploads file content with auto-detected content type.
    ///
    /// Maps file extension to MIME type.
    pub async fn upload_file(
        &self,
        key: &str,
        content: Vec<u8>,
        file_extension: &str,
    ) -> Result<()> {
        let content_type = match file_extension {
            "json" => "application/json",
            "map" | "dat" => "text/plain",
            _ => "application/octet-stream",
        };

        self.upload(key, content, content_type).await
    }
}
