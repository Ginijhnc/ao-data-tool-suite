//! Map file import orchestration.
//!
//! Discovers, filters, parses, and inserts map data files into the database.
//! Uses parallel parsing via rayon for throughput on large map sets.

use std::path::Path;
use std::time::{Instant, SystemTime};

use anyhow::Result;
use rayon::prelude::*;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::db::{insert_maps, prepare_map_data};
use crate::execution_tracking::filter_modified_since;
use crate::parsers::dat::maps::{discover_map_files, parse_map_file};

/// Use a fixed batch size for maps since the total count is typically small
/// (around 500 maps on average). Not worth making this configurable via .env.
const MAP_BATCH_SIZE: usize = 100;

/// Parses and imports map .dat files into the database.
pub async fn import_maps(
    pool: &PgPool,
    maps_dir: &Path,
    last_exec: Option<SystemTime>,
    strip_comments: bool,
) -> Result<()> {
    if !maps_dir.exists() {
        warn!("Directorio de mapas no encontrado: {}", maps_dir.display());
        return Ok(());
    }

    let map_file_paths = discover_map_files(maps_dir);

    if map_file_paths.is_empty() {
        warn!(
            "No se encontraron archivos de mapas en {}",
            maps_dir.display()
        );
        return Ok(());
    }

    let map_files: Vec<_> = map_file_paths
        .into_iter()
        .filter_map(|path| {
            std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|modified| (path, modified))
        })
        .collect();

    let modified_files = filter_modified_since(map_files, last_exec);

    if modified_files.is_empty() {
        info!("Mapas: ninguno modificado desde la ultima ejecucion");
        return Ok(());
    }

    let start = Instant::now();

    let parsed_maps: Vec<_> = modified_files
        .par_iter()
        .filter_map(|path| match parse_map_file(path, strip_comments) {
            Ok(map) => Some(map),
            Err(e) => {
                warn!("Error parseando {:?}: {}", path, e);
                None
            }
        })
        .collect();

    let map_count = parsed_maps.len();
    let map_data = prepare_map_data(parsed_maps);

    let (inserted, errors) =
        insert_maps(pool, &map_data, MAP_BATCH_SIZE).await;

    let elapsed = start.elapsed().as_secs_f64();

    info!(
        "Mapas: {} parseados, {} insertados, {} errores, {:.3}s",
        map_count, inserted, errors, elapsed
    );

    Ok(())
}
