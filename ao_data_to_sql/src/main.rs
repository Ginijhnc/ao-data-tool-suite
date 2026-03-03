//! # `ao_data_to_sql`
//!
//! Imports Argentum Online game data files into `PostgreSQL`.
//!
//! ## Supported File Types
//!
//! - `.CHR` - Character save files (implemented)
//! - `.DAT` - Objects, NPCs, spells, cities, etc. (planned)
//! - `.map/.inf` - Map tiles and metadata (planned)

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use clap::Parser;
use sqlx::PgPool;
use tracing::{info, warn};

use ao_data_to_sql::db::insert_charfiles;
use ao_data_to_sql::execution_tracking::{
    filter_modified_since, read_last_execution, write_last_execution,
};
use ao_data_to_sql::parsers::characters::{
    discover_chr_files, parse_charfiles,
};

/// CLI arguments for the import tool.
#[derive(Parser, Debug)]
#[command(name = "ao_data_to_sql")]
#[command(about = "Importa archivos de Argentum Online a PostgreSQL")]
struct Args {
    /// Directory containing .CHR character files.
    #[arg(short, long, env = "CHARFILE_DIR", default_value = "./Charfile")]
    charfile_dir: PathBuf,

    /// Number of records per database batch insert.
    #[arg(short, long, env = "BATCH_SIZE", default_value = "100")]
    batch_size: usize,

    /// Number of parsing threads (0 = auto-detect CPU cores).
    #[arg(short, long, env = "THREADS", default_value = "0")]
    threads: usize,

    /// Rollback migrations instead of applying them.
    #[arg(
        long,
        default_value = "false",
        help = "Revertir migraciones en lugar de aplicarlas"
    )]
    rollback: bool,

    /// Target migration version for rollback (0 = rollback all).
    #[arg(
        long,
        default_value = "0",
        help = "Version objetivo para rollback (0 = revertir todas)"
    )]
    rollback_target: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

/// Main application entry point that orchestrates the import process.
async fn run() -> Result<()> {
    let args = init()?;
    let pool = setup_database(&args).await?;

    if args.rollback {
        return Ok(());
    }

    import_characters(&pool, &args).await
}

/// Initializes logging, loads environment, and parses CLI arguments.
fn init() -> Result<Args> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ao_data_to_sql=info".parse()?),
        )
        .init();

    dotenvy::dotenv().ok();

    let args = Args::parse();

    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .context("Error configurando pool de hilos")?;
    }

    Ok(args)
}

/// Connects to the database and runs or rolls back migrations.
async fn setup_database(args: &Args) -> Result<PgPool> {
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
    } else {
        migrator
            .run(&pool)
            .await
            .context("Error ejecutando migraciones")?;
    }

    Ok(pool)
}

/// Discovers, parses, and inserts character files into the database.
async fn import_characters(pool: &PgPool, args: &Args) -> Result<()> {
    let total_start = Instant::now();

    let all_files = discover_chr_files(&args.charfile_dir)
        .context("Error descubriendo archivos .CHR")?;

    if all_files.is_empty() {
        warn!(
            "No se encontraron archivos .CHR en {}",
            args.charfile_dir.display()
        );
        return Ok(());
    }

    let last_exec = read_last_execution();
    let chr_files = filter_modified_since(all_files, last_exec);

    if chr_files.is_empty() {
        info!("No hay archivos modificados desde la última ejecución");
        write_last_execution()?;
        return Ok(());
    }

    let (char_data, parse_errors) = parse_charfiles(&chr_files);
    let (inserted, insert_errors) =
        insert_charfiles(pool, &char_data, args.batch_size).await;

    write_last_execution()?;

    let total_secs = total_start.elapsed().as_secs_f64();

    info!(
        "Importación: {} archivos modificados, {} insertados, {} errores parseo, {} errores inserción, {:.3}s",
        chr_files.len(),
        inserted,
        parse_errors,
        insert_errors,
        total_secs
    );

    Ok(())
}

/// Logs a formatted summary of the import process with counts and timings.
#[allow(
    dead_code,
    clippy::cognitive_complexity,
    reason = "kept for future detailed logging"
)]
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
