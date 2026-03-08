//! Database operations for storing parsed game data.
//!
//! ## Implemented
//!
//! - Character files (.CHR)
//! - NPC definitions (NPCs.dat)
//! - Object definitions (Obj.dat)
//! - Spell definitions (Hechizos.dat)
//!
//! ## Planned
//!
//! - Crafting (`ObjCarpintero`, `ArmadurasHerrero`, `ArmasHerrero`)

mod characters;
mod npcs;
mod objects;
mod spells;

pub use characters::insert_charfiles;
pub use npcs::{NpcData, insert_npcs, prepare_npc_data};
pub use objects::{ObjectData, insert_objects, prepare_object_data};
pub use spells::{SpellData, insert_spells, prepare_spell_data};
