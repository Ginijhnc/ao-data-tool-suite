//! # `ao_data_to_sql`
//!
//! Imports Argentum Online game data files into `PostgreSQL`.
//!
//! ## Supported File Types
//!
//! - `.CHR` - Character save files (implemented)
//! - `.DAT` - NPCs, objects, spells (implemented); cities, balance, crafting (planned)
//! - `.map/.inf` - Map tiles and metadata (planned)

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use sqlx::PgPool;
use tracing::info;

use ao_data_to_sql::execution_tracking::{
    read_last_execution, write_last_execution,
};
use ao_data_to_sql::importers::{
    import_balance, import_blacksmith_armors, import_blacksmith_weapons,
    import_carpenter_objects, import_characters, import_faction_armors,
    import_maps, import_npcs, import_objects, import_spells,
};

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

    /// Directory containing map .dat files (mapa1.dat, mapa2.dat, etc.).
    #[arg(long, env = "MAPS_DIR", default_value = "./Server/Maps")]
    maps_dir: PathBuf,

    /// Path to Server.ini file for GM detection.
    #[arg(long, env = "SERVER_INI_PATH")]
    server_ini_path: PathBuf,

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

    /// Strip inline comments (text after ') from DAT file fields.
    #[arg(
        long,
        env = "STRIP_DAT_INLINE_COMMENTS",
        default_value = "true",
        help = "Strip inline comments (text after ') from DAT fields (though it intentionally preserves section header comments)"
    )]
    strip_dat_inline_comments: bool,
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
    let batch = args.batch_size;
    let strip = args.strip_dat_inline_comments;

    import_characters(
        &pool,
        &args.charfile_dir,
        &args.server_ini_path,
        last_exec,
        batch,
        args.enable_profiling,
    )
    .await?;

    import_npcs(&pool, &args.dats_dir, last_exec, strip, batch).await?;
    import_objects(&pool, &args.dats_dir, last_exec, strip, batch).await?;
    import_spells(&pool, &args.dats_dir, last_exec, strip, batch).await?;
    import_carpenter_objects(&pool, &args.dats_dir, last_exec, strip, batch)
        .await?;
    import_blacksmith_armors(&pool, &args.dats_dir, last_exec, strip, batch)
        .await?;
    import_blacksmith_weapons(&pool, &args.dats_dir, last_exec, strip, batch)
        .await?;
    import_faction_armors(&pool, &args.dats_dir, last_exec, strip, batch)
        .await?;
    import_balance(&pool, &args.dats_dir, last_exec, strip, batch).await?;
    import_maps(&pool, &args.maps_dir, last_exec, strip).await?;

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
