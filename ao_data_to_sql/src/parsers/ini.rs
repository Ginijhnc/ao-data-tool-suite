//! INI file parser with legacy encoding support.
//!
//! Handles `[SECTION]` headers and `KEY=VALUE` pairs.
//! Tries UTF-8 first, falls back to Windows-1252 for legacy VB6 servers.

use std::collections::HashMap;

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

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section_name = trimmed[1..trimmed.len() - 1].to_uppercase();
            current_section = Some(section_name.clone());
            data.entry(section_name).or_default();
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
