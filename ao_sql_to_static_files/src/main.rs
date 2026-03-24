//! # `ao_sql_to_static_files`
//!
//! Exports Argentum Online game data from `PostgreSQL` to static JSON files.
//!
//! Queries the database for game data exports and writes them to JSON files
//! suitable for CDN distribution, with configurable limits and output directories.

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use tracing::info;

use ao_sql_to_static_files::cdn::{R2Config, upload_with_manifest};
use ao_sql_to_static_files::queries::{fetch_top_level, fetch_top_pvp_kills};
use ao_sql_to_static_files::serialization::{
    serialize_export_data, write_export_file,
};

/// Command-line arguments for the ranking exporter.
#[derive(Parser, Debug)]
#[command(
    name = "ao_sql_to_static_files",
    about = "Exporta rankings de Argentum Online de PostgreSQL a archivos JSON estáticos"
)]
struct Args {
    /// Maximum number of entries per ranking
    #[arg(
        long,
        env = "RANKING_LIMIT",
        default_value = "50",
        help = "Límite de entradas por ranking"
    )]
    ranking_limit: i32,

    /// Write JSON files to disk (for testing/debugging only)
    #[arg(
        long,
        env = "WRITE_TO_DISK",
        default_value = "false",
        help = "Escribir archivos JSON al disco (solo para testing/debugging)"
    )]
    write_to_disk: bool,

    /// Output directory for generated JSON files
    #[arg(
        long,
        env = "STATIC_JSON_OUTPUT_DIR",
        default_value = "./ao_sql_to_static_files/exported-json",
        help = "Directorio de salida para archivos JSON"
    )]
    output_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ao_sql_to_static_files=info".into()),
        )
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    if args.write_to_disk {
        info!(
            "Iniciando exportación de datos (límite: {}, salida: {})",
            args.ranking_limit,
            args.output_dir.display()
        );
    } else {
        info!(
            "Iniciando exportación de datos (límite: {}, modo: solo CDN)",
            args.ranking_limit
        );
    }

    let start = Instant::now();

    // Connect to database with single connection (batch job)
    let pool = ao_shared::create_pool(1).await?;

    // Fetch data
    info!("Consultando ranking por nivel...");
    let level_data = fetch_top_level(&pool, args.ranking_limit).await?;

    info!("Consultando ranking por asesinatos PvP...");
    let pvp_data = fetch_top_pvp_kills(&pool, args.ranking_limit).await?;

    if args.write_to_disk {
        // Disk mode: no manifest, no change detection
        let level_json = serialize_export_data(level_data)?;
        let pvp_json = serialize_export_data(pvp_data)?;

        write_export_file(
            &args.output_dir,
            "characters/top-by-level.json",
            &level_json,
        )?;
        write_export_file(
            &args.output_dir,
            "characters/top-by-kills.json",
            &pvp_json,
        )?;
    } else {
        // CDN mode: hash-based change detection via database
        let r2_config = R2Config::from_env()?;

        upload_with_manifest(&pool, &r2_config, level_data, pvp_data).await?;
    }

    let elapsed = start.elapsed();
    info!("Exportación completada en {:.2?}", elapsed);

    Ok(())
}
