//! NPC data file (NPCs.dat) parser.
//!
//! Extracts NPC ID from section name and stores all fields as JSONB.

use std::path::Path;

use thiserror::Error;

use crate::parsers::ini::{IniParseError, IniSection, parse_ini_bytes};

/// NPC parsing errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum NpcError {
    /// File could not be read.
    #[error("Error leyendo archivo: {0}")]
    IoError(#[from] std::io::Error),
    /// INI parsing failed.
    #[error("Error parseando archivo: {0}")]
    ParseError(#[from] IniParseError),
}

/// Result type for NPC operations.
pub type Result<T> = core::result::Result<T, NpcError>;

/// A parsed NPC with ID extracted from section name.
#[derive(Debug)]
#[non_exhaustive]
pub struct ParsedNpc {
    /// NPC ID from section name (e.g., `[NPC34]` -> 34).
    pub id: i32,
    /// NPC name from the `Name=` field.
    pub name: String,
    /// All NPC data as key-value pairs.
    pub data: IniSection,
}

/// Parses NPCs.dat file into individual NPC records.
pub fn parse_npcs_file(path: &Path) -> Result<Vec<ParsedNpc>> {
    let bytes = std::fs::read(path)?;
    let ini_data = parse_ini_bytes(&bytes)?;

    let mut npcs = Vec::new();

    for (section_name, section_data) in ini_data {
        let Some(id) = extract_npc_id(&section_name) else {
            continue;
        };

        let name = section_data.get("NAME").cloned().unwrap_or_default();

        npcs.push(ParsedNpc {
            id,
            name,
            data: section_data,
        });
    }

    npcs.sort_by_key(|npc| npc.id);

    Ok(npcs)
}

/// Extracts NPC ID from section name (e.g., `NPC34` -> 34).
fn extract_npc_id(section: &str) -> Option<i32> {
    section.strip_prefix("NPC").and_then(|n| n.parse().ok())
}
