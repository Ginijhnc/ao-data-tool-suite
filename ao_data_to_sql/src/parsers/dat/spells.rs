//! Spell data file (Hechizos.dat) parser.
//!
//! Extracts spell ID from section name and stores all fields as JSONB.

use std::path::Path;

use crate::parsers::ini::{
    DatParseError, DatParseResult, ParsedDatEntry, parse_dat_file,
};

/// Spell parsing errors.
pub type SpellError = DatParseError;

/// Result type for spell operations.
pub type Result<T> = DatParseResult<T>;

/// A parsed spell with ID extracted from section name.
pub type ParsedSpell = ParsedDatEntry;

/// Parses Hechizos.dat file into individual spell records.
pub fn parse_spells_file(path: &Path) -> Result<Vec<ParsedSpell>> {
    parse_dat_file(path, "HECHIZO", "NOMBRE")
}
