//! File parsers for Argentum Online data formats.
//!
//! - [`ini`] - INI file parser with encoding detection
//! - [`characters`] - Character file (.CHR) parser
//! - [`dat`] - Binary DAT file parsers (planned)
//! - [`maps`] - Map file parsers (planned)

pub mod characters;
pub mod dat;
pub mod ini;
pub mod maps;
