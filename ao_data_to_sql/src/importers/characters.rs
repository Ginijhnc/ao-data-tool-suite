//! Character file import orchestration.
//!
//! Discovers, filters, parses, and inserts character save files into the
//! database. Includes detailed performance profiling when enabled.

#![allow(
    clippy::std_instead_of_alloc,
    reason = "Arc/Mutex from std required for profiling module interop"
)]

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};

use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::{info, warn};

use crate::db::insert_charfiles;
use crate::execution_tracking::filter_modified_since;
use crate::parsers::characters::{discover_chr_files, parse_charfiles};
use crate::parsers::server_ini::parse_gm_names;
use crate::profiling;

/// Discovers, parses, and inserts character files into the database.
pub async fn import_characters(
    pool: &PgPool,
    charfile_dir: &Path,
    server_ini_path: &Path,
    last_exec: Option<SystemTime>,
    batch_size: usize,
    enable_profiling: bool,
) -> Result<()> {
    let total_start = Instant::now();

    let (peak_cpu, peak_memory_mb, cpu_monitor_handle) = if enable_profiling {
        profiling::start_cpu_monitoring()
    } else {
        (Arc::new(Mutex::new(0.0)), Arc::new(Mutex::new(0.0)), None)
    };

    let scan_start = Instant::now();
    let all_files = discover_chr_files(charfile_dir)
        .context("Error descubriendo archivos .CHR")?;
    let scan_secs = scan_start.elapsed().as_secs_f64();

    if all_files.is_empty() {
        warn!(
            "No se encontraron archivos .CHR en {}",
            charfile_dir.display()
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

    let gm_names = parse_gm_names(server_ini_path)
        .context("Error procesando Server.ini")?;

    let parse_start = Instant::now();
    let (char_data, parse_errors) = parse_charfiles(&chr_files, &gm_names);
    let parse_secs = parse_start.elapsed().as_secs_f64();

    let insert_start = Instant::now();
    let (inserted, insert_errors) =
        insert_charfiles(pool, &char_data, batch_size).await;
    let insert_secs = insert_start.elapsed().as_secs_f64();

    let total_secs = total_start.elapsed().as_secs_f64();

    #[allow(
        clippy::unwrap_used,
        reason = "mutex poisoning handled by monitoring task design"
    )]
    let (final_peak_cpu, final_peak_memory_mb) = if enable_profiling {
        if let Some(handle) = cpu_monitor_handle {
            handle.abort();
        }
        let cpu = *peak_cpu.lock().unwrap();
        let mem = *peak_memory_mb.lock().unwrap();
        (cpu, mem)
    } else {
        (0.0, 0.0)
    };

    if enable_profiling {
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
            batch_size,
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
    info!("RESUMEN DE IMPORTACIÓN - PERSONAJES");
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
