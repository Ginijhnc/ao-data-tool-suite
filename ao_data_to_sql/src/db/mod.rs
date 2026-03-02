//! Database operations for storing parsed game data.
//!
//! ## Planned Modules
//!
//! - `objects` - obj.dat
//! - `npcs` - NPCs.dat
//! - `spells` - Hechizos.dat
//! - `crafting` - ObjCarpintero, ArmadurasHerrero, ArmasHerrero

mod characters;

pub use characters::insert_characters_batch;
