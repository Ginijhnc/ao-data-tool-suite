//! Database operations for storing parsed game data.
//!
//! ## Implemented
//!
//! - Character files (.CHR)
//! - NPC definitions (NPCs.dat)
//!
//! ## Planned
//!
//! - Objects (obj.dat)
//! - Spells (Hechizos.dat)
//! - Crafting (`ObjCarpintero`, `ArmadurasHerrero`, `ArmasHerrero`)

mod characters;
mod npcs;

pub use characters::insert_charfiles;
pub use npcs::{NpcData, insert_npcs, prepare_npc_data};
