//! # `ao_data_to_sql`
//!
//! Imports Argentum Online game data files into `PostgreSQL`.
//!
//! ## Supported File Types
//!
//! - `.CHR` - Character save files (implemented)
//! - `NPCs.dat` - NPC definitions (implemented)
//! - `.DAT` - Objects, spells, cities, etc. (planned)
//! - `.map/.inf` - Map tiles and metadata (planned)

#![allow(
    clippy::std_instead_of_alloc,
    reason = "Arc/Mutex from std required for profiling module interop"
)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{Context, Result};
use clap::Parser;
use sqlx::PgPool;
use tracing::{info, warn};

use ao_data_to_sql::db::{insert_charfiles, insert_npcs, prepare_npc_data};
use ao_data_to_sql::execution_tracking::{
    filter_modified_since, read_last_execution, was_modified_since,
    write_last_execution,
};
use ao_data_to_sql::parsers::characters::{
    discover_chr_files, parse_charfiles,
};
use ao_data_to_sql::parsers::dat::npcs::parse_npcs_file;
use ao_data_to_sql::profiling;

/// CLI arguments for the import tool.
#[derive(Parser, Debug)]
#[command(name = "ao_data_to_sql")]
#[command(about = "Importa archivos de Argentum Online a PostgreSQL")]
struct Args {
    /// Directory containing .CHR character files.
    #[arg(short, long, env = "CHARFILE_DIR", default_value = "./Charfile")]
    charfile_dir: PathBuf,

    /// Directory containing .DAT files (NPCs.dat, Obj.dat, etc.).
    #[arg(long, env = "DATS_DIR", default_value = "./Server/Dat")]
    dats_dir: PathBuf,

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

    /// Enable detailed performance profiling (CPU/RAM tracking).
    #[arg(
        long,
        env = "ENABLE_PROFILING",
        default_value = "false",
        help = "Habilitar profiling detallado de rendimiento"
    )]
    enable_profiling: bool,
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

    let last_exec = read_last_execution();

    import_characters(&pool, &args, last_exec).await?;
    import_npcs(&pool, &args, last_exec).await?;

    write_last_execution()?;

    Ok(())
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
async fn import_characters(
    pool: &PgPool,
    args: &Args,
    last_exec: Option<std::time::SystemTime>,
) -> Result<()> {
    let total_start = Instant::now();

    let (peak_cpu, peak_memory_mb, cpu_monitor_handle) =
        if args.enable_profiling {
            profiling::start_cpu_monitoring()
        } else {
            (Arc::new(Mutex::new(0.0)), Arc::new(Mutex::new(0.0)), None)
        };

    let scan_start = Instant::now();
    let all_files = discover_chr_files(&args.charfile_dir)
        .context("Error descubriendo archivos .CHR")?;
    let scan_secs = scan_start.elapsed().as_secs_f64();

    if all_files.is_empty() {
        warn!(
            "No se encontraron archivos .CHR en {}",
            args.charfile_dir.display()
        );
        return Ok(());
    }

    let filter_start = Instant::now();
    let chr_files = filter_modified_since(all_files, last_exec);
    let filter_secs = filter_start.elapsed().as_secs_f64();

    if chr_files.is_empty() {
        info!(
            "Personajes: no hay archivos modificados desde la última ejecución"
        );
        return Ok(());
    }

    let parse_start = Instant::now();
    let (char_data, parse_errors) = parse_charfiles(&chr_files);
    let parse_secs = parse_start.elapsed().as_secs_f64();

    let insert_start = Instant::now();
    let (inserted, insert_errors) =
        insert_charfiles(pool, &char_data, args.batch_size).await;
    let insert_secs = insert_start.elapsed().as_secs_f64();

    let total_secs = total_start.elapsed().as_secs_f64();

    #[allow(
        clippy::unwrap_used,
        reason = "mutex poisoning handled by monitoring task design"
    )]
    let (final_peak_cpu, final_peak_memory_mb) = if args.enable_profiling {
        if let Some(handle) = cpu_monitor_handle {
            handle.abort();
        }
        let cpu = *peak_cpu.lock().unwrap();
        let mem = *peak_memory_mb.lock().unwrap();
        (cpu, mem)
    } else {
        (0.0, 0.0)
    };

    if args.enable_profiling {
        print_summary(
            chr_files.len(),
            char_data.len(),
            parse_errors,
            inserted,
            insert_errors,
            scan_secs,
            filter_secs,
            parse_secs,
            insert_secs,
            total_secs,
            args.batch_size,
            final_peak_memory_mb,
            final_peak_cpu,
        );
    } else {
        info!(
            "Personajes: {} archivos modificados, {} insertados, {} errores parseo, {} errores inserción, {:.3}s",
            chr_files.len(),
            inserted,
            parse_errors,
            insert_errors,
            total_secs
        );
    }

    Ok(())
}

/// Parses and imports NPCs.dat into the database.
async fn import_npcs(
    pool: &PgPool,
    args: &Args,
    last_exec: Option<std::time::SystemTime>,
) -> Result<()> {
    let npcs_path = args.dats_dir.join("NPCs.dat");

    if !npcs_path.exists() {
        warn!("NPCs.dat no encontrado en {}", npcs_path.display());
        return Ok(());
    }

    if !was_modified_since(&npcs_path, last_exec) {
        info!("NPCs: no modificado desde la última ejecución");
        return Ok(());
    }

    let start = Instant::now();

    let parsed_npcs =
        parse_npcs_file(&npcs_path).context("Error parseando NPCs.dat")?;

    let npc_count = parsed_npcs.len();
    let npc_data = prepare_npc_data(parsed_npcs);

    let (inserted, errors) =
        insert_npcs(pool, &npc_data, args.batch_size).await;

    let elapsed = start.elapsed().as_secs_f64();

    info!(
        "NPCs: {} parseados, {} insertados, {} errores, {:.3}s",
        npc_count, inserted, errors, elapsed
    );

    Ok(())
}

/// Logs a formatted summary of the import process with counts and timings.
#[allow(
    clippy::cognitive_complexity,
    clippy::cast_precision_loss,
    clippy::as_conversions,
    clippy::too_many_arguments,
    reason = "precision loss acceptable for performance metrics display, many args needed for comprehensive metrics"
)]
fn print_summary(
    found: usize,
    parsed: usize,
    parse_errors: usize,
    inserted: usize,
    insert_errors: usize,
    scan_secs: f64,
    filter_secs: f64,
    parse_secs: f64,
    insert_secs: f64,
    total_secs: f64,
    batch_size: usize,
    peak_memory_mb: f64,
    cpu_usage_pct: f32,
) {
    let scan_pct = (scan_secs / total_secs) * 100.0;
    let filter_pct = (filter_secs / total_secs) * 100.0;
    let parse_pct = (parse_secs / total_secs) * 100.0;
    let insert_pct = (insert_secs / total_secs) * 100.0;

    let parse_rate = if parse_secs > 0.0 {
        parsed as f64 / parse_secs
    } else {
        0.0
    };

    let insert_rate = if insert_secs > 0.0 {
        inserted as f64 / insert_secs
    } else {
        0.0
    };

    info!("========================================");
    info!("RESUMEN DE IMPORTACIÓN");
    info!("========================================");
    info!("Archivos encontrados:      {}", found);
    info!("Archivos parseados:        {}", parsed);
    info!("Errores de parseo:         {}", parse_errors);
    info!("Registros insertados:      {}", inserted);
    info!("Errores de inserción:      {}", insert_errors);
    info!("Batch size:                {}", batch_size);
    info!("RAM pico (script):         {:.1} MB", peak_memory_mb);
    info!("CPU pico (script):         {:.1}%", cpu_usage_pct);
    info!("----------------------------------------");
    info!("DESGLOSE DE TIEMPOS:");
    info!(
        "  Búsqueda archivos:       {:.3}s ({:.1}%)",
        scan_secs, scan_pct
    );
    info!(
        "  Filtrado modificados:    {:.3}s ({:.1}%)",
        filter_secs, filter_pct
    );
    info!(
        "  Parseo:                  {:.3}s ({:.1}%) - {:.0} archivos/s",
        parse_secs, parse_pct, parse_rate
    );
    info!(
        "  Inserción DB:            {:.3}s ({:.1}%) - {:.0} registros/s",
        insert_secs, insert_pct, insert_rate
    );
    info!("----------------------------------------");
    info!("TIEMPO TOTAL:              {:.3} segundos", total_secs);
    info!(
        "THROUGHPUT GENERAL:        {:.0} archivos/s",
        found as f64 / total_secs
    );
    info!("========================================");
}
