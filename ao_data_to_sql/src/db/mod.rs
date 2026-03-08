//! Database operations for storing parsed game data.
//!
//! ## Implemented
//!
//! - Character files (.CHR)
//! - NPC definitions (NPCs.dat)
//! - Object definitions (Obj.dat)
//!
//! ## Planned
//!
//! - Spells (Hechizos.dat)
//! - Crafting (`ObjCarpintero`, `ArmadurasHerrero`, `ArmasHerrero`)

mod characters;
mod npcs;
mod objects;

pub use characters::insert_charfiles;
pub use npcs::{NpcData, insert_npcs, prepare_npc_data};
pub use objects::{ObjectData, insert_objects, prepare_object_data};
