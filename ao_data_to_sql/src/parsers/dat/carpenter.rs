//! Carpenter object file (ObjCarpintero.dat) parser.
//!
//! Extracts object ID from section name and stores all fields as JSONB.
//! Renames the `INDEX` field to `OBJ_INDEX` for clarity.

use std::path::Path;

use crate::parsers::ini::{
    DatParseError, DatParseResult, ParsedDatEntry, parse_dat_file,
};

/// Carpenter object parsing errors.
pub type CarpenterError = DatParseError;

/// Result type for carpenter object operations.
pub type Result<T> = DatParseResult<T>;

/// A parsed carpenter object with ID extracted from section name.
pub type ParsedCarpenterObject = ParsedDatEntry;

/// Parses ObjCarpintero.dat file into individual object records.
///
/// Renames the `INDEX` field to `OBJ_INDEX` for clarity when stored in the database.
pub fn parse_carpenter_file(
    path: &Path,
) -> Result<Vec<ParsedCarpenterObject>> {
    let mut entries = parse_dat_file(path, "OBJ", "")?;

    for entry in &mut entries {
        if let Some(index_value) = entry.data.remove("INDEX") {
            entry.data.insert("OBJ_INDEX".to_owned(), index_value);
        }
    }

    Ok(entries)
}
