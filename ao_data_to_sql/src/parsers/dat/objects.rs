//! Object data file (Obj.dat) parser.
//!
//! Extracts object ID from section name and stores all fields as JSONB.

use std::path::Path;

use crate::parsers::ini::{
    DatParseError, DatParseResult, ParsedDatEntry, parse_dat_file,
};

/// Object parsing errors.
pub type ObjectError = DatParseError;

/// Result type for object operations.
pub type Result<T> = DatParseResult<T>;

/// A parsed object with ID extracted from section name.
pub type ParsedObject = ParsedDatEntry;

/// Parses Obj.dat file into individual object records.
pub fn parse_objects_file(path: &Path) -> Result<Vec<ParsedObject>> {
    parse_dat_file(path, "OBJ", "NAME")
}
