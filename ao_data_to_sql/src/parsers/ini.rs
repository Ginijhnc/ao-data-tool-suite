use encoding_rs::WINDOWS_1252;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IniParseError {
    #[error("Error de codificación en archivo")]
    EncodingError,
}

pub type IniSection = HashMap<String, String>;
pub type IniData = HashMap<String, IniSection>;

pub fn parse_ini_bytes(bytes: &[u8]) -> Result<IniData, IniParseError> {
    let content = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (decoded, _, had_errors) = WINDOWS_1252.decode(bytes);
            if had_errors {
                return Err(IniParseError::EncodingError);
            }
            decoded.into_owned()
        }
    };

    Ok(parse_ini_string(&content))
}

fn parse_ini_string(content: &str) -> IniData {
    let mut data: IniData = HashMap::new();
    let mut current_section: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line[1..line.len() - 1].to_uppercase();
            current_section = Some(section_name.clone());
            data.entry(section_name).or_default();
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_uppercase();
            let value = line[eq_pos + 1..].trim().to_string();

            if let Some(ref section) = current_section {
                data.get_mut(section).unwrap().insert(key, value);
            }
        }
    }

    data
}
