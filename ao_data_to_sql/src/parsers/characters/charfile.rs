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

#[derive(Error, Debug)]
pub enum CharfileError {
    #[error("Error leyendo archivo: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Error parseando archivo: {0}")]
    ParseError(#[from] IniParseError),
    #[error("Nombre de archivo inválido: {0}")]
    InvalidFilename(String),
}

pub struct ParsedCharfile {
    pub name: String,
    pub data: IniData,
}

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
    pub fn new() -> Self {
        Self {
            skip_fields: SKIP_FIELDS.iter().map(|s| s.to_string()).collect(),
            skip_sections: SKIP_SECTIONS.iter().map(|s| s.to_string()).collect(),
        }
    }

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
