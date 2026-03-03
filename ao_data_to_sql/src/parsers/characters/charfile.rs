//! Character file (.CHR) parser implementation.
//!
//! Extracts character name from filename and filters sensitive fields before storage.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use thiserror::Error;
use walkdir::WalkDir;

use crate::parsers::ini::{IniData, IniParseError, parse_ini_bytes};

/// Sensitive fields to remove from parsed character data (privacy protection).
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

/// INI sections to completely remove from parsed data.
const SKIP_SECTIONS: &[&str] = &["CONTACTO"];

/// Character file parsing errors.
#[derive(Error, Debug)]
#[non_exhaustive]
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
    /// File modification time could not be retrieved.
    #[error("Error obteniendo tiempo de modificación: {0}")]
    MetadataError(String),
}

/// A parsed character with name extracted from filename.
#[non_exhaustive]
pub struct ParsedCharfile {
    /// Character name (from filename, without .chr extension).
    pub name: String,
    /// Character data as INI sections.
    pub data: IniData,
}

/// Parser for .CHR character files with privacy filtering.
pub struct CharfileParser {
    /// Field names to remove from all sections.
    skip_fields: HashSet<String>,
    /// Section names to completely remove.
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
            skip_fields: SKIP_FIELDS
                .iter()
                .copied()
                .map(String::from)
                .collect(),
            skip_sections: SKIP_SECTIONS
                .iter()
                .copied()
                .map(String::from)
                .collect(),
        }
    }

    /// Parses a .CHR file, extracting name from filename and filtering sensitive data.
    pub fn parse_file(
        &self,
        path: &Path,
    ) -> Result<ParsedCharfile, CharfileError> {
        let bytes = std::fs::read(path)?;
        let filename =
            path.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
                CharfileError::InvalidFilename(path.display().to_string())
            })?;

        let name = filename
            .strip_suffix(".chr")
            .or_else(|| filename.strip_suffix(".CHR"))
            .ok_or_else(|| {
                CharfileError::InvalidFilename(filename.to_owned())
            })?
            .to_owned();

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

/// Recursively finds all `.chr` files in a directory, excluding `.chr.bk` backups.
pub fn discover_chr_files(
    dir: &Path,
) -> Result<Vec<(PathBuf, SystemTime)>, CharfileError> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        let is_chr = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("chr"));
        let is_backup = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.to_lowercase().ends_with(".chr.bk"));

        if is_chr && !is_backup {
            let metadata = std::fs::metadata(path)?;
            let mtime = metadata.modified().map_err(|e| {
                CharfileError::MetadataError(format!(
                    "{}: {}",
                    path.display(),
                    e
                ))
            })?;
            files.push((path.to_path_buf(), mtime));
        }
    }

    Ok(files)
}
