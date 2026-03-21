//! Database query functions for fetching ranking data.
//!
//! This module provides functions to retrieve character data from `PostgreSQL`
//! ordered by different ranking criteria.

mod characters;

pub use characters::{RankedCharacter, fetch_top_level, fetch_top_pvp_kills};
