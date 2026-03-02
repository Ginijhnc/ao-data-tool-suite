//! # ao_data_to_sql
//!
//! Imports Argentum Online game data files into PostgreSQL.
//!
//! ## Supported File Types
//!
//! - `.CHR` - Character save files (implemented)
//! - `.DAT` - Objects, NPCs, spells, cities, etc. (planned)
//! - `.map/.inf` - Map tiles and metadata (planned)

mod db;
mod parsers;

use anyhow::{Context, Result};
use parsers::characters::{CharfileParser, ParsedCharfile};
use clap::Parser;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tracing::{error, info, warn};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "ao_data_to_sql")]
#[command(about = "Importa archivos de Argentum Online a PostgreSQL")]
struct Args {
    #[arg(short, long, env = "CHARFILE_DIR", default_value = "./Charfile")]
    charfile_dir: PathBuf,

    #[arg(short, long, env = "BATCH_SIZE", default_value = "100")]
    batch_size: usize,

    #[arg(short, long, env = "THREADS", default_value = "0")]
    threads: usize,

    #[arg(
        long,
        default_value = "false",
        help = "Revertir migraciones en lugar de aplicarlas"
    )]
    rollback: bool,

    #[arg(
        long,
        default_value = "0",
        help = "Version objetivo para rollback (0 = revertir todas)"
    )]
    rollback_target: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ao_data_to_sql=info".parse()?),
        )
        .init();

    dotenvy::dotenv().ok();

    let args = Args::parse();

    let total_start = Instant::now();

    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .context("Error configurando pool de hilos")?;
    }

    let pool = ao_shared::create_pool(10)
        .await
        .context("Error conectando a la base de datos")?;

    let migrations_path = Path::new("./ao_data_to_sql/migrations");
    let migrator = sqlx::migrate::Migrator::new(migrations_path)
        .await
        .context("Error cargando migraciones")?;

    if args.rollback {
        migrator
            .undo(&pool, args.rollback_target)
            .await
            .context("Error revirtiendo migraciones")?;
        info!(
            "Migraciones revertidas hasta version {}",
            args.rollback_target
        );
        return Ok(());
    }

    migrator
        .run(&pool)
        .await
        .context("Error ejecutando migraciones")?;

    let chr_files = discover_chr_files(&args.charfile_dir)?;

    if chr_files.is_empty() {
        warn!(
            "No se encontraron archivos .CHR en {}",
            args.charfile_dir.display()
        );
        return Ok(());
    }

    let parser = CharfileParser::new();
    let error_count = AtomicUsize::new(0);

    let parsed: Vec<ParsedCharfile> = chr_files
        .par_iter()
        .filter_map(|path| match parser.parse_file(path) {
            Ok(charfile) => Some(charfile),
            Err(e) => {
                error_count.fetch_add(1, Ordering::Relaxed);
                error!("Error parseando {}: {}", path.display(), e);
                None
            }
        })
        .collect();

    let mut inserted = 0;
    let mut insert_errors = 0;

    let char_data: Vec<(String, serde_json::Value)> = parsed
        .into_iter()
        .filter_map(|c| {
            serde_json::to_value(&c.data)
                .ok()
                .map(|json| (c.name, json))
        })
        .collect();

    for batch in char_data.chunks(args.batch_size) {
        match db::insert_characters_batch(&pool, batch).await {
            Ok(count) => inserted += count,
            Err(e) => {
                insert_errors += batch.len();
                error!("Error insertando lote: {}", e);
            }
        }
    }

    let parse_errors = error_count.load(Ordering::Relaxed);
    let total_secs = total_start.elapsed().as_secs_f64();

    info!(
        "Importación: {} archivos, {} insertados, {} errores parseo, {} errores inserción, {:.3}s",
        chr_files.len(),
        inserted,
        parse_errors,
        insert_errors,
        total_secs
    );

    Ok(())
}

/// Recursively finds all .chr files in a directory, excluding .chr.bk backups.
fn discover_chr_files(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let name_lower = name.to_lowercase();
            if name_lower.ends_with(".chr") && !name_lower.ends_with(".chr.bk") {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}

/// Logs a formatted summary of the import process with counts and timings.
fn print_summary(
    found: usize,
    parsed: usize,
    parse_errors: usize,
    inserted: usize,
    insert_errors: usize,
    scan_secs: f64,
    parse_secs: f64,
    db_secs: f64,
    insert_secs: f64,
    total_secs: f64,
) {
    info!("========================================");
    info!("RESUMEN DE IMPORTACIÓN");
    info!("========================================");
    info!("Archivos encontrados:      {}", found);
    info!("Archivos parseados:        {}", parsed);
    info!("Errores de parseo:         {}", parse_errors);
    info!("Registros insertados:      {}", inserted);
    info!("Errores de inserción:      {}", insert_errors);
    info!("----------------------------------------");
    info!("Búsqueda de archivos .CHR: {:.3}s", scan_secs);
    info!("Parseo de archivos:        {:.3}s", parse_secs);
    info!("Conexión a DB:             {:.3}s", db_secs);
    info!("Inserción en DB:           {:.3}s", insert_secs);
    info!("----------------------------------------");
    info!("TIEMPO TOTAL:              {:.3} segundos", total_secs);
    info!("========================================");
}
