//! Character file (.CHR) parser implementation.
//!
//! Extracts character name from filename and filters sensitive fields before storage.

use crate::parsers::ini::{parse_ini_bytes, IniData, IniParseError};
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

const SKIP_FIELDS: &[&str] = &[
    "EMAIL",
    "PASSWORD",
    "PASSWORDHASH",
    "PASSWORDSALT",
    "LASTIP1",
    "LASTIP2",
    "LASTIP3",
    "LASTIP4",
    "LASTIP5",
];

const SKIP_SECTIONS: &[&str] = &["CONTACTO"];

/// Character file parsing errors.
#[derive(Error, Debug)]
pub enum CharfileError {
    /// File could not be read.
    #[error("Error leyendo archivo: {0}")]
    IoError(#[from] std::io::Error),
    /// INI parsing failed.
    #[error("Error parseando archivo: {0}")]
    ParseError(#[from] IniParseError),
    /// Filename is not a valid .chr file.
    #[error("Nombre de archivo inválido: {0}")]
    InvalidFilename(String),
}

/// A parsed character with name extracted from filename.
pub struct ParsedCharfile {
    /// Character name (from filename, without .chr extension).
    pub name: String,
    /// Character data as INI sections.
    pub data: IniData,
}

/// Parser for .CHR character files with privacy filtering.
pub struct CharfileParser {
    skip_fields: HashSet<String>,
    skip_sections: HashSet<String>,
}

impl Default for CharfileParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CharfileParser {
    /// Creates a new parser with default privacy filters.
    pub fn new() -> Self {
        Self {
            skip_fields: SKIP_FIELDS.iter().map(|s| s.to_string()).collect(),
            skip_sections: SKIP_SECTIONS.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Parses a .CHR file, extracting name from filename and filtering sensitive data.
    pub fn parse_file(&self, path: &Path) -> Result<ParsedCharfile, CharfileError> {
        let bytes = std::fs::read(path)?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CharfileError::InvalidFilename(path.display().to_string()))?;

        let name = filename
            .strip_suffix(".chr")
            .or_else(|| filename.strip_suffix(".CHR"))
            .ok_or_else(|| CharfileError::InvalidFilename(filename.to_string()))?
            .to_string();

        let mut data = parse_ini_bytes(&bytes)?;

        for section_name in &self.skip_sections {
            data.remove(section_name);
        }

        for section in data.values_mut() {
            for field in &self.skip_fields {
                section.remove(field);
            }
        }

        Ok(ParsedCharfile { name, data })
    }
}
