//! Balance file (Balance.dat) parser.
//!
//! Parses named configuration sections (MODEVASION, DISTRIBUCION, etc.) into
//! individual records keyed by section name. Unlike other DAT files, sections
//! are not numerically indexed.

use std::path::Path;

use crate::parsers::ini::{
    DatParseError, DatParseResult, IniSection, parse_ini_bytes,
};

/// Balance parsing errors.
pub type BalanceError = DatParseError;

/// Result type for balance operations.
pub type Result<T> = DatParseResult<T>;

/// A parsed balance section with its name and key-value data.
#[derive(Debug)]
#[non_exhaustive]
pub struct ParsedBalanceSection {
    /// Section name, e.g. "MODEVASION", "EXTRA".
    pub section: String,
    /// All section data as key-value pairs.
    pub data: IniSection,
}

/// Parses Balance.dat into individual named section records.
///
/// Unlike other DAT files, Balance.dat sections are semantic names
/// (`MODEVASION`, `DISTRIBUCION`, etc.) with no numeric ID to extract.
/// `parse_dat_file` cannot be used here because it requires a shared numeric
/// prefix per section (e.g. `[NPC34]`) and maps to an `INTEGER` primary key.
/// The balance table instead uses `TEXT` keyed by section name.
pub fn parse_balance_file(
    path: &Path,
    strip_inline_comments: bool,
) -> Result<Vec<ParsedBalanceSection>> {
    let bytes = std::fs::read(path)?;
    let ini_data = parse_ini_bytes(&bytes, strip_inline_comments)?;

    let mut sections: Vec<ParsedBalanceSection> = ini_data
        .into_iter()
        .map(|(section, data)| ParsedBalanceSection { section, data })
        .collect();

    sections.sort_by(|a, b| a.section.cmp(&b.section));

    Ok(sections)
}
