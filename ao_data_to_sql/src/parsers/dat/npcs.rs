//! NPC data file (NPCs.dat) parser.
//!
//! Extracts NPC ID from section name and stores all fields as JSONB.

use std::path::Path;

use crate::parsers::ini::{
    DatParseError, DatParseResult, ParsedDatEntry, parse_dat_file,
};

/// NPC parsing errors.
pub type NpcError = DatParseError;

/// Result type for NPC operations.
pub type Result<T> = DatParseResult<T>;

/// A parsed NPC with ID extracted from section name.
pub type ParsedNpc = ParsedDatEntry;

/// Parses NPCs.dat file into individual NPC records.
pub fn parse_npcs_file(path: &Path) -> Result<Vec<ParsedNpc>> {
    parse_dat_file(path, "NPC", "NAME")
}
