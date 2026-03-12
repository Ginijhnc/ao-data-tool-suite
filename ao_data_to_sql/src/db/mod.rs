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
//! - Blacksmith weapons (ArmasHerrero.dat)
//! - Faction armors (ArmadurasFaccionarias.dat)
//! - Balance (Balance.dat)
//! - Maps (mapa*.dat)

mod balance;
mod blacksmith_armors;
mod blacksmith_weapons;
mod carpenter;
mod characters;
mod faction_armors;
mod maps;
mod npcs;
mod objects;
mod spells;

pub use balance::{BalanceData, insert_balance, prepare_balance_data};
pub use blacksmith_armors::{
    BlacksmithArmorData, insert_blacksmith_armors,
    prepare_blacksmith_armor_data,
};
pub use blacksmith_weapons::{
    BlacksmithWeaponData, insert_blacksmith_weapons,
    prepare_blacksmith_weapon_data,
};
pub use carpenter::{
    CarpenterObjectData, insert_carpenter_objects,
    prepare_carpenter_object_data,
};
pub use characters::insert_charfiles;
pub use faction_armors::{
    FactionArmorData, insert_faction_armors, prepare_faction_armor_data,
};
pub use maps::{MapData, insert_maps, prepare_map_data};
pub use npcs::{NpcData, insert_npcs, prepare_npc_data};
pub use objects::{ObjectData, insert_objects, prepare_object_data};
pub use spells::{SpellData, insert_spells, prepare_spell_data};
