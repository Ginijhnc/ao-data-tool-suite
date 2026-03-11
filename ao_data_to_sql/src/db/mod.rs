//! Database operations for storing parsed game data.
//!
//! ## Implemented
//!
//! - Character files (.CHR)
//! - NPC definitions (NPCs.dat)
//! - Object definitions (Obj.dat)
//! - Spell definitions (Hechizos.dat)
//! - Carpenter objects (ObjCarpintero.dat)
//! - Blacksmith armors (ArmadurasHerrero.dat)
//!
//! ## Planned
//!
//! - Crafting (`ArmasHerrero`)

mod blacksmith_armors;
mod carpenter;
mod characters;
mod npcs;
mod objects;
mod spells;

pub use blacksmith_armors::{
    BlacksmithArmorData, insert_blacksmith_armors,
    prepare_blacksmith_armor_data,
};
pub use carpenter::{
    CarpenterObjectData, insert_carpenter_objects,
    prepare_carpenter_object_data,
};
pub use characters::insert_charfiles;
pub use npcs::{NpcData, insert_npcs, prepare_npc_data};
pub use objects::{ObjectData, insert_objects, prepare_object_data};
pub use spells::{SpellData, insert_spells, prepare_spell_data};
