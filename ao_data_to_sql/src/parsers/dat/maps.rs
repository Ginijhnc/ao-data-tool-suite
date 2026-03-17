//! Map data file (mapa*.dat) parser.
//!
//! Parses individual map .dat files. Unlike other DAT files, maps are stored
//! as individual files (mapa1.dat, mapa2.dat) rather than a single file.
//! Consolidates SONIDO{n} sections into a nested SONIDOS array inside the map object.

use std::path::Path;

use serde_json::{Map, Value};

use crate::parsers::ini::{
    DatParseError, DatParseResult, IniData, IniSection, coerce_ini_section,
    parse_ini_bytes,
};

/// Map parsing errors.
pub type MapError = DatParseError;

/// Result type for map operations.
pub type Result<T> = DatParseResult<T>;

/// A parsed map with ID, name, and consolidated JSON data.
#[derive(Debug)]
#[non_exhaustive]
pub struct ParsedMap {
    /// Map ID extracted from filename (e.g., mapa1.dat -> 1).
    pub id: i32,
    /// Map display name from NAME field (may be empty).
    pub name: String,
    /// Map data as JSON with SONIDOS nested as an array.
    pub data: Value,
}

/// Parses a single map .dat file (e.g., mapa1.dat, mapa113.dat).
///
/// Extracts map ID from filename and consolidates SONIDO sections into
/// a nested SONIDOS array within the main map object.
pub fn parse_map_file(
    path: &Path,
    strip_inline_comments: bool,
) -> Result<ParsedMap> {
    let id = extract_map_id(path)?;
    let bytes = std::fs::read(path)?;
    let ini_data = parse_ini_bytes(&bytes, strip_inline_comments)?;

    // Find the MAPA* section key
    #[allow(
        clippy::pattern_type_mismatch,
        reason = "BTreeMap::iter() returns (&K, &V) references"
    )]
    let map_key = ini_data
        .iter()
        .find(|(section_name, _)| {
            section_name.to_uppercase().starts_with("MAPA")
        })
        .map(|(k, _)| k.clone());

    let name = map_key
        .as_deref()
        .and_then(|k| ini_data.get(k))
        .and_then(|section| section.get("NAME"))
        .cloned()
        .unwrap_or_default();

    let data = consolidate_map_sections(&ini_data, map_key.as_deref());

    Ok(ParsedMap { id, name, data })
}

/// Consolidates INI sections into a single JSON object with SONIDOS as an array.
///
/// Merges MAPA* fields as top-level keys and SONIDO{n} sections into a nested array.
fn consolidate_map_sections(
    ini_data: &IniData,
    map_key: Option<&str>,
) -> Value {
    let mut map_obj = Map::new();

    if let Some(k) = map_key
        && let Some(section) = ini_data.get(k)
        && let Value::Object(coerced) = coerce_ini_section(section.clone())
    {
        map_obj.extend(coerced);
    }

    // Collect SONIDO{n} sections (exclude SONIDOS metadata), sorted by number
    let mut sounds: Vec<(i32, &IniSection)> = ini_data
        .iter()
        .filter_map(|(name, data)| {
            name.strip_prefix("SONIDO")
                .and_then(|suffix| suffix.parse::<i32>().ok())
                .map(|n| (n, data))
        })
        .collect();
    #[allow(
        clippy::pattern_type_mismatch,
        reason = "Vec::sort_by_key closure receives &(i32, _) reference"
    )]
    sounds.sort_by_key(|&(n, _)| n);

    if !sounds.is_empty() {
        let sound_array: Vec<Value> = sounds
            .into_iter()
            .map(|(_, section)| coerce_ini_section(section.clone()))
            .collect();
        map_obj.insert("SONIDOS".to_owned(), Value::Array(sound_array));
    }

    let section_key = map_key.unwrap_or("MAPA").to_owned();
    let mut root = Map::new();
    root.insert(section_key, Value::Object(map_obj));
    Value::Object(root)
}

/// Extracts map ID from filename (e.g., "mapa1.dat" -> 1, "mapa113.dat" -> 113).
fn extract_map_id(path: &Path) -> Result<i32> {
    let filename =
        path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "nombre de archivo invalido",
            )
        })?;

    let filename_lower = filename.to_lowercase();
    let id_str = filename_lower.strip_prefix("mapa").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "nombre de archivo debe comenzar con 'mapa'",
        )
    })?;

    id_str.parse::<i32>().map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "ID de mapa invalido",
        )
        .into()
    })
}

/// Discovers all map .dat files in a directory.
#[must_use]
pub fn discover_map_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    #[allow(
        clippy::std_instead_of_core,
        reason = "Result::ok ambiguous without std"
    )]
    entries
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let path = entry.path();
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| {
                    let lower = name.to_lowercase();
                    lower.starts_with("mapa")
                        && path
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("dat"))
                })
        })
        .map(|entry| entry.path())
        .collect()
}
