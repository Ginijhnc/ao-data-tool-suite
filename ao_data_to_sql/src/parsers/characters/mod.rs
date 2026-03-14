//! Character file (.CHR) parsing.
//!
//! Parses character save files and filters sensitive data (passwords, IPs, emails).

mod charfile;

use core::hash::BuildHasher;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use tracing::error;

#[allow(
    unused_imports,
    reason = "CharfileError re-exported for public API"
)]
pub use charfile::{
    CharfileError, CharfileParser, ParsedCharfile, discover_chr_files,
};

/// Character name paired with its parsed JSONB data and GM flag.
pub type CharacterData = (String, serde_json::Value, bool);

/// Parses charfiles in parallel, returning parsed data and error count.
///
/// Enriches each character with a GM flag by checking if their name appears in the provided Server.ini.
#[must_use]
pub fn parse_charfiles<S: BuildHasher + Sync>(
    chr_files: &[PathBuf],
    gm_names: &HashSet<String, S>,
) -> (Vec<CharacterData>, usize) {
    let parser = CharfileParser::new();
    let error_count = AtomicUsize::new(0);

    let char_data: Vec<CharacterData> = chr_files
        .par_iter()
        .filter_map(|path| try_parse_file(&parser, path, &error_count))
        .filter_map(|c| {
            serde_json::to_value(&c.data).ok().map(|json| {
                let is_gm = gm_names.contains(&c.name.to_uppercase());
                (c.name, json, is_gm)
            })
        })
        .collect();

    (char_data, error_count.load(Ordering::Relaxed))
}

/// Attempts to parse a single charfile, logging errors and incrementing the counter on failure.
fn try_parse_file(
    parser: &CharfileParser,
    path: &Path,
    error_count: &AtomicUsize,
) -> Option<ParsedCharfile> {
    match parser.parse_file(path) {
        Ok(charfile) => Some(charfile),
        Err(e) => {
            error_count.fetch_add(1, Ordering::Relaxed);
            error!("Error parseando {}: {e}", path.display());
            None
        }
    }
}
