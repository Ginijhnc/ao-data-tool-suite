//! Faction armor file (ArmadurasFaccionarias.dat) parser.
//!
//! Extracts class ID from section name and stores all fields as JSONB.
//! Section comments (e.g., `' Guerrero`) are stored as `_COMMENT`.

use std::path::Path;

use crate::parsers::ini::{
    DatParseError, DatParseResult, ParsedDatEntry, parse_dat_file,
};

/// Faction armor parsing errors.
pub type FactionArmorError = DatParseError;

/// Result type for faction armor operations.
pub type Result<T> = DatParseResult<T>;

/// A parsed faction armor class with ID extracted from section name.
pub type ParsedFactionArmor = ParsedDatEntry;

/// Parses ArmadurasFaccionarias.dat file into individual class records.
pub fn parse_faction_armors_file(
    path: &Path,
) -> Result<Vec<ParsedFactionArmor>> {
    parse_dat_file(path, "CLASE", "")
}
