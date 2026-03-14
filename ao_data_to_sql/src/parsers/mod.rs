//! File parsers for Argentum Online data formats.
//!
//! - [`ini`] - INI file parser with encoding detection
//! - [`characters`] - Character file (.CHR) parser
//! - [`dat`] - Binary DAT file parsers (planned)
//! - [`server_ini`] - Server.ini parser for GM detection

pub mod characters;
pub mod dat;
pub mod ini;
pub mod server_ini;
