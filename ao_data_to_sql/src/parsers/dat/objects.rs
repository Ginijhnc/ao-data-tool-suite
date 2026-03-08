//! Object data file (Obj.dat) parser.
//!
//! Extracts object ID from section name and stores all fields as JSONB.

use std::path::Path;

use thiserror::Error;

use crate::parsers::ini::{IniParseError, IniSection, parse_ini_bytes};

/// Object parsing errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ObjectError {
    /// File could not be read.
    #[error("Error leyendo archivo: {0}")]
    IoError(#[from] std::io::Error),
    /// INI parsing failed.
    #[error("Error parseando archivo: {0}")]
    ParseError(#[from] IniParseError),
}

/// Result type for object operations.
pub type Result<T> = core::result::Result<T, ObjectError>;

/// A parsed object with ID extracted from section name.
#[derive(Debug)]
#[non_exhaustive]
pub struct ParsedObject {
    /// Object ID from section name (e.g., `[OBJ34]` -> 34).
    pub id: i32,
    /// Object name from the `Name=` field.
    pub name: String,
    /// All object data as key-value pairs.
    pub data: IniSection,
}

/// Parses Obj.dat file into individual object records.
pub fn parse_objects_file(path: &Path) -> Result<Vec<ParsedObject>> {
    let bytes = std::fs::read(path)?;
    let ini_data = parse_ini_bytes(&bytes)?;

    let mut objects = Vec::new();

    for (section_name, section_data) in ini_data {
        let Some(id) = extract_object_id(&section_name) else {
            continue;
        };

        let name = section_data.get("NAME").cloned().unwrap_or_default();

        objects.push(ParsedObject {
            id,
            name,
            data: section_data,
        });
    }

    objects.sort_by_key(|obj| obj.id);

    Ok(objects)
}

/// Extracts object ID from section name (e.g., `OBJ34` -> 34).
fn extract_object_id(section: &str) -> Option<i32> {
    section.strip_prefix("OBJ").and_then(|n| n.parse().ok())
}
