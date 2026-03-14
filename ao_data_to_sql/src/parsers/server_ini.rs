//! Server.ini parser for extracting Game Master names.
//!
//! Reads GM character names from configured sections (Admines, Dioses, etc.).
//! These names are used to enrich character data with a GM flag during import.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result};

use crate::parsers::ini::parse_ini_bytes;

/// GM section names in Server.ini (uppercased as the INI parser uppercases all section names).
const GM_SECTIONS: &[&str] = &[
    "ADMINES",
    "DIOSES",
    "SEMIDIOSES",
    "CONSEJEROS",
    "ROLESMASTERS",
];

/// Extracts all GM character names from Server.ini.
///
/// Collects names from all GM sections, uppercased for case-insensitive matching.
/// Missing sections or empty values are silently skipped.
pub fn parse_gm_names(path: &Path) -> Result<HashSet<String>> {
    let bytes = std::fs::read(path).context("Error leyendo Server.ini")?;
    let data = parse_ini_bytes(&bytes, false)
        .context("Error parseando Server.ini")?;

    let mut gm_names = HashSet::new();

    for section_name in GM_SECTIONS {
        if let Some(fields) = data.get(*section_name) {
            for value in fields.values() {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    gm_names.insert(trimmed.to_uppercase());
                }
            }
        }
    }

    Ok(gm_names)
}
