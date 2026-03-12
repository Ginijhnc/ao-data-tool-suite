//! DAT file parsers.
//!
//! ## Implemented Modules
//!
//! - [`npcs`] - NPCs.dat
//! - [`objects`] - Obj.dat
//! - [`spells`] - Hechizos.dat
//! - [`carpenter`] - ObjCarpintero.dat
//! - [`blacksmith_armors`] - ArmadurasHerrero.dat
//! - [`blacksmith_weapons`] - ArmasHerrero.dat
//! - [`faction_armors`] - ArmadurasFaccionarias.dat
//! - [`balance`] - Balance.dat
//! - [`maps`] - mapa*.dat

pub mod balance;
pub mod blacksmith_armors;
pub mod blacksmith_weapons;
pub mod carpenter;
pub mod faction_armors;
pub mod maps;
pub mod npcs;
pub mod objects;
pub mod spells;
