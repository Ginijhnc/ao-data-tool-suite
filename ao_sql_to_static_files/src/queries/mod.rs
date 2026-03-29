//! Database query functions for fetching game data.
//!
//! This module provides functions to retrieve game data from `PostgreSQL`
//! ordered by various criteria for export to static JSON files.

mod characters;
mod dats;
mod ranking_builder;

pub use characters::{
    CHARACTER_CLASSES, RankedCharacter, fetch_top_level,
    fetch_top_level_by_class, fetch_top_pvp_kills,
    fetch_top_pvp_kills_by_class,
};
pub use dats::build_all_dat_exports;
pub use ranking_builder::{ExportEntry, build_all_ranking_exports};
