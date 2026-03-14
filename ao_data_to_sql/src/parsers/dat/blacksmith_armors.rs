//! Blacksmith armor file (ArmadurasHerrero.dat) parser.
//!
//! Extracts armor ID from section name and stores all fields as JSONB.
//! Renames the `INDEX` field to `OBJ_INDEX` for clarity.

use std::path::Path;

use crate::parsers::ini::{
    DatParseError, DatParseResult, ParsedDatEntry, parse_dat_file,
};

/// Blacksmith armor parsing errors.
pub type BlacksmithArmorError = DatParseError;

/// Result type for blacksmith armor operations.
pub type Result<T> = DatParseResult<T>;

/// A parsed blacksmith armor with ID extracted from section name.
pub type ParsedBlacksmithArmor = ParsedDatEntry;

/// Parses ArmadurasHerrero.dat file into individual armor records.
///
/// Renames the `INDEX` field to `OBJ_INDEX` for clarity when stored in the database.
pub fn parse_blacksmith_armors_file(
    path: &Path,
    strip_inline_comments: bool,
) -> Result<Vec<ParsedBlacksmithArmor>> {
    let mut entries =
        parse_dat_file(path, "ARMADURA", "", strip_inline_comments)?;

    for entry in &mut entries {
        if let Some(index_value) = entry.data.remove("INDEX") {
            entry.data.insert("OBJ_INDEX".to_owned(), index_value);
        }
    }

    Ok(entries)
}
