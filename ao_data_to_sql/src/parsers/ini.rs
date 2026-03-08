//! INI file parser with legacy encoding support.
//!
//! Handles `[SECTION]` headers and `KEY=VALUE` pairs.
//! Supports section comments like `[NPC52] 'Propiedades Bander`.
//! Tries UTF-8 first, falls back to Windows-1252 for legacy VB6 servers.
//!
//! Also provides generic DAT file parsing for files following the
//! `[PREFIX{id}]` section pattern (NPCs.dat, Obj.dat, Hechizos.dat, etc.).

use std::collections::HashMap;
use std::path::Path;

use encoding_rs::WINDOWS_1252;
use thiserror::Error;

/// INI parsing errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum IniParseError {
    /// File encoding could not be determined.
    #[error("Error de codificación en archivo")]
    EncodingError,
}

/// DAT file parsing errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DatParseError {
    /// File could not be read.
    #[error("Error leyendo archivo: {0}")]
    IoError(#[from] std::io::Error),
    /// INI parsing failed.
    #[error("Error parseando archivo: {0}")]
    ParseError(#[from] IniParseError),
}

/// Result type for DAT parsing operations.
pub type DatParseResult<T> = core::result::Result<T, DatParseError>;

/// A parsed DAT entry with ID extracted from section name.
#[derive(Debug)]
#[non_exhaustive]
pub struct ParsedDatEntry {
    /// Entry ID from section name (e.g., `[NPC34]` -> 34).
    pub id: i32,
    /// Entry name from the specified name field (empty if not found).
    pub name: String,
    /// All entry data as key-value pairs.
    pub data: IniSection,
}

/// A single INI section: key-value pairs.
pub type IniSection = HashMap<String, String>;

/// Complete INI file: section name -> section data.
pub type IniData = HashMap<String, IniSection>;

/// Parses raw bytes as INI, detecting encoding automatically.
pub fn parse_ini_bytes(bytes: &[u8]) -> Result<IniData, IniParseError> {
    let content = if let Ok(s) = core::str::from_utf8(bytes) {
        s.to_owned()
    } else {
        let (decoded, _, had_errors) = WINDOWS_1252.decode(bytes);
        if had_errors {
            return Err(IniParseError::EncodingError);
        }
        decoded.into_owned()
    };

    Ok(parse_ini_string(&content))
}

/// Parses an INI string into sections and key-value pairs.
fn parse_ini_string(content: &str) -> IniData {
    let mut data: IniData = HashMap::new();
    let mut current_section: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty()
            || trimmed.starts_with(';')
            || trimmed.starts_with('#')
        {
            continue;
        }

        if trimmed.starts_with('[')
            && let Some((section_name, comment)) =
                parse_section_header(trimmed)
        {
            current_section = Some(section_name.clone());
            let section_data = data.entry(section_name).or_default();
            if let Some(c) = comment {
                section_data.insert("_COMMENT".to_owned(), c);
            }
            continue;
        }

        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_uppercase();
            let value = trimmed[eq_pos + 1..].trim().to_owned();

            if let Some(ref section) = current_section
                && let Some(section_data) = data.get_mut(section)
            {
                section_data.insert(key, value);
            }
        }
    }

    data
}

/// Parses a section header line like `[NPC52] 'Propiedades Bander`.
///
/// Returns the section name (uppercase) and an optional comment.
fn parse_section_header(line: &str) -> Option<(String, Option<String>)> {
    let close_bracket = line.find(']')?;
    let section_name = line[1..close_bracket].to_uppercase();

    let remainder = line[close_bracket + 1..].trim();
    let comment = remainder.strip_prefix('\'').and_then(|comment_text| {
        let trimmed = comment_text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    });

    Some((section_name, comment))
}

/// Extracts numeric ID from section name by stripping the given prefix.
///
/// For example, `extract_section_id("NPC34", "NPC")` returns `Some(34)`.
#[must_use]
pub fn extract_section_id(section: &str, prefix: &str) -> Option<i32> {
    section.strip_prefix(prefix).and_then(|n| n.parse().ok())
}

/// Parses a DAT file into individual entries.
///
/// The `section_prefix` determines which sections to extract (e.g., `"NPC"`, `"OBJ"`).
/// The `name_field` specifies which field contains the entry name (e.g., `"NAME"`, `"Nombre"`).
/// Pass an empty string for `name_field` if entries have no name field.
pub fn parse_dat_file(
    path: &Path,
    section_prefix: &str,
    name_field: &str,
) -> DatParseResult<Vec<ParsedDatEntry>> {
    let bytes = std::fs::read(path)?;
    let ini_data = parse_ini_bytes(&bytes)?;

    let mut entries = Vec::new();

    for (section_name, section_data) in ini_data {
        let Some(id) = extract_section_id(&section_name, section_prefix)
        else {
            continue;
        };

        let name = if name_field.is_empty() {
            String::new()
        } else {
            section_data.get(name_field).cloned().unwrap_or_default()
        };

        entries.push(ParsedDatEntry {
            id,
            name,
            data: section_data,
        });
    }

    entries.sort_by_key(|entry| entry.id);

    Ok(entries)
}
