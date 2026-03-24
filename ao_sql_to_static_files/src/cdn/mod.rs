//! Cloudflare R2 CDN upload functionality.
//!
//! Provides R2 client and configuration for uploading static files.

mod client;
mod config;

pub use client::R2Client;
pub use config::R2Config;
