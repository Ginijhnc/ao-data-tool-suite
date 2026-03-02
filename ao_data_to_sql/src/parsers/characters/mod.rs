//! Character file (.CHR) parsing.
//!
//! Parses character save files and filters sensitive data (passwords, IPs, emails).

mod charfile;

pub use charfile::{CharfileError, CharfileParser, ParsedCharfile};
