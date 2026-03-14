//! Blacksmith weapon file (ArmasHerrero.dat) parser.
//!
//! Extracts weapon ID from section name and stores all fields as JSONB.
//! Renames the `INDEX` field to `OBJ_INDEX` for clarity.

use std::path::Path;

use crate::parsers::ini::{
    DatParseError, DatParseResult, ParsedDatEntry, parse_dat_file,
};

/// Blacksmith weapon parsing errors.
pub type BlacksmithWeaponError = DatParseError;

/// Result type for blacksmith weapon operations.
pub type Result<T> = DatParseResult<T>;

/// A parsed blacksmith weapon with ID extracted from section name.
pub type ParsedBlacksmithWeapon = ParsedDatEntry;

/// Parses ArmasHerrero.dat file into individual weapon records.
///
/// Renames the `INDEX` field to `OBJ_INDEX` for clarity when stored in the database.
pub fn parse_blacksmith_weapons_file(
    path: &Path,
    strip_inline_comments: bool,
) -> Result<Vec<ParsedBlacksmithWeapon>> {
    let mut entries = parse_dat_file(path, "ARMA", "", strip_inline_comments)?;

    for entry in &mut entries {
        if let Some(index_value) = entry.data.remove("INDEX") {
            entry.data.insert("OBJ_INDEX".to_owned(), index_value);
        }
    }

    Ok(entries)
}
