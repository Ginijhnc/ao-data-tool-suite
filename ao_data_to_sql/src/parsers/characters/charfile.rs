//! Character file (.CHR) parser implementation.
//!
//! Extracts character name from filename and filters sensitive fields before storage.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use ignore::WalkBuilder;
use thiserror::Error;

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

        let mut data = parse_ini_bytes(&bytes, false)?;

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

/// Checks if a path is a valid .chr file (not a backup).
fn is_valid_chr_file(path: &Path) -> bool {
    let is_chr = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("chr"));
    let is_backup = path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.to_lowercase().ends_with(".chr.bk"));

    is_chr && !is_backup
}

/// Recursively finds all `.chr` files in a directory using parallel traversal.
///
/// Uses the `ignore` crate for parallel directory walking, which provides
/// significant speedup on directories with many files.
pub fn discover_chr_files(
    dir: &Path,
) -> Result<Vec<(PathBuf, SystemTime)>, CharfileError> {
    let files: Mutex<Vec<(PathBuf, SystemTime)>> = Mutex::new(Vec::new());
    let errors: Mutex<Option<CharfileError>> = Mutex::new(None);

    WalkBuilder::new(dir)
        .follow_links(false)
        .standard_filters(false)
        .build_parallel()
        .run(|| {
            let files = &files;
            let errors = &errors;

            Box::new(move |result| {
                let Ok(entry) = result else {
                    return ignore::WalkState::Continue;
                };

                let path = entry.path();

                if !is_valid_chr_file(path) {
                    return ignore::WalkState::Continue;
                }

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(e) => {
                        if let Ok(mut err_guard) = errors.lock()
                            && err_guard.is_none()
                        {
                            *err_guard = Some(CharfileError::IoError(
                                std::io::Error::other(e.to_string()),
                            ));
                        }
                        return ignore::WalkState::Continue;
                    }
                };

                let mtime = match metadata.modified() {
                    Ok(t) => t,
                    Err(e) => {
                        if let Ok(mut err_guard) = errors.lock()
                            && err_guard.is_none()
                        {
                            *err_guard = Some(CharfileError::MetadataError(
                                format!("{}: {}", path.display(), e),
                            ));
                        }
                        return ignore::WalkState::Continue;
                    }
                };

                if let Ok(mut files_guard) = files.lock() {
                    files_guard.push((path.to_path_buf(), mtime));
                }

                ignore::WalkState::Continue
            })
        });

    if let Ok(err_guard) = errors.lock()
        && let Some(e) = err_guard.as_ref()
    {
        return Err(CharfileError::MetadataError(e.to_string()));
    }

    let result = files
        .into_inner()
        .map_err(|e| CharfileError::MetadataError(e.to_string()))?;

    Ok(result)
}
