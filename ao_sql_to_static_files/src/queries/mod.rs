//! Database query functions for fetching game data.
//!
//! This module provides functions to retrieve game data from `PostgreSQL`
//! ordered by various criteria for export to static JSON files.

mod characters;

pub use characters::{RankedCharacter, fetch_top_level, fetch_top_pvp_kills};
