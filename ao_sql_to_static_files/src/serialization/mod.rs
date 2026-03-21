//! JSON file writing utilities for ranking data.
//!
//! This module handles serialization and file system operations for
//! exporting ranking data to static JSON files.

mod writer;

pub use writer::write_ranking_file;
