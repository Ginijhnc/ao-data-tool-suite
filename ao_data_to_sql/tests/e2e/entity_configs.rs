//! Entity configurations for E2E tests.
//!
//! Centralizes all test entity configurations and provides helper functions
//! for verifying entity counts and data against fixtures.

use std::path::PathBuf;

use crate::common::{
    EntityTestConfig, SourceDir, verify_entity_count,
    verify_entity_data_matches_fixture,
};

// ============================================================================
// Fixture Directory Constants
// ============================================================================

/// Fixture subdirectory for spells.
pub const SPELL_FIXTURES: &str = "spells";

/// Fixture subdirectory for objects.
pub const OBJECT_FIXTURES: &str = "objects";

/// Fixture subdirectory for characters.
pub const CHARACTER_FIXTURES: &str = "characters";

/// Fixture subdirectory for NPCs.
pub const NPC_FIXTURES: &str = "npcs";

/// Fixture subdirectory for balance sections.
pub const BALANCE_FIXTURES: &str = "balance";

/// Fixture subdirectory for maps.
pub const MAP_FIXTURES: &str = "maps";

/// Fixture subdirectory for carpenter objects.
pub const CARPENTER_FIXTURES: &str = "carpenter_objects";

/// Fixture subdirectory for blacksmith weapons.
pub const BLACKSMITH_WEAPONS_FIXTURES: &str = "blacksmith_weapons";

/// Fixture subdirectory for blacksmith armors.
pub const BLACKSMITH_ARMORS_FIXTURES: &str = "blacksmith_armors";

/// Fixture subdirectory for faction armors.
pub const FACTION_ARMORS_FIXTURES: &str = "faction_armors";

/// Fixture subdirectory for Server.ini files.
pub const SERVER_INI_FIXTURES: &str = "server_ini";

// ============================================================================
// Entity Configurations
// ============================================================================

/// Configuration for spell entity tests.
pub const SPELL_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "spells",
    id_column: "id",
    display_name: "spell",
    source_dir: SourceDir::Dats,
};

/// Configuration for object entity tests.
pub const OBJECT_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "objects",
    id_column: "id",
    display_name: "object",
    source_dir: SourceDir::Dats,
};

/// Configuration for character entity tests.
pub const CHARACTER_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "characters",
    id_column: "name",
    display_name: "character",
    source_dir: SourceDir::Charfile,
};

/// Configuration for NPC entity tests.
pub const NPC_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "npcs",
    id_column: "id",
    display_name: "NPC",
    source_dir: SourceDir::Dats,
};

/// Configuration for balance entity tests.
pub const BALANCE_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "balance",
    id_column: "section",
    display_name: "balance section",
    source_dir: SourceDir::Dats,
};

/// Configuration for map entity tests.
pub const MAP_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "maps",
    id_column: "id",
    display_name: "map",
    source_dir: SourceDir::Maps,
};

/// Configuration for carpenter object entity tests.
pub const CARPENTER_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "carpenter_objects",
    id_column: "id",
    display_name: "carpenter object",
    source_dir: SourceDir::Dats,
};

/// Configuration for blacksmith weapon entity tests.
pub const BLACKSMITH_WEAPON_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "blacksmith_weapons",
    id_column: "id",
    display_name: "blacksmith weapon",
    source_dir: SourceDir::Dats,
};

/// Configuration for blacksmith armor entity tests.
pub const BLACKSMITH_ARMOR_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "blacksmith_armors",
    id_column: "id",
    display_name: "blacksmith armor",
    source_dir: SourceDir::Dats,
};

/// Configuration for faction armor entity tests.
pub const FACTION_ARMOR_CONFIG: EntityTestConfig = EntityTestConfig {
    table_name: "faction_armors",
    id_column: "id",
    display_name: "faction armor",
    source_dir: SourceDir::Dats,
};

// ============================================================================
// Helper Functions
// ============================================================================

// Spells
pub async fn verify_spell_count(expected_count: i64) {
    verify_entity_count(&SPELL_CONFIG, expected_count).await;
}

pub async fn verify_spell_data_matches_fixture(
    spell_id: i32,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(&SPELL_CONFIG, spell_id, fixture_path)
        .await;
}

// Objects
pub async fn verify_object_count(expected_count: i64) {
    verify_entity_count(&OBJECT_CONFIG, expected_count).await;
}

pub async fn verify_object_data_matches_fixture(
    object_id: i32,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(
        &OBJECT_CONFIG,
        object_id,
        fixture_path,
    )
    .await;
}

// Characters
pub async fn verify_character_count(expected_count: i64) {
    verify_entity_count(&CHARACTER_CONFIG, expected_count).await;
}

pub async fn verify_character_data_matches_fixture(
    character_name: &str,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(
        &CHARACTER_CONFIG,
        character_name,
        fixture_path,
    )
    .await;
}

// NPCs
pub async fn verify_npc_count(expected_count: i64) {
    verify_entity_count(&NPC_CONFIG, expected_count).await;
}

pub async fn verify_npc_data_matches_fixture(
    npc_id: i32,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(&NPC_CONFIG, npc_id, fixture_path)
        .await;
}

// Balance
pub async fn verify_balance_count(expected_count: i64) {
    verify_entity_count(&BALANCE_CONFIG, expected_count).await;
}

pub async fn verify_balance_data_matches_fixture(
    section: &str,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(&BALANCE_CONFIG, section, fixture_path)
        .await;
}

// Maps
pub async fn verify_map_count(expected_count: i64) {
    verify_entity_count(&MAP_CONFIG, expected_count).await;
}

pub async fn verify_map_data_matches_fixture(
    map_id: i32,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(&MAP_CONFIG, map_id, fixture_path)
        .await;
}

// Carpenter
pub async fn verify_carpenter_object_count(expected_count: i64) {
    verify_entity_count(&CARPENTER_CONFIG, expected_count).await;
}

pub async fn verify_carpenter_object_data_matches_fixture(
    object_id: i32,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(
        &CARPENTER_CONFIG,
        object_id,
        fixture_path,
    )
    .await;
}

// Blacksmith Weapons
pub async fn verify_blacksmith_weapon_count(expected_count: i64) {
    verify_entity_count(&BLACKSMITH_WEAPON_CONFIG, expected_count).await;
}

pub async fn verify_blacksmith_weapon_data_matches_fixture(
    weapon_id: i32,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(
        &BLACKSMITH_WEAPON_CONFIG,
        weapon_id,
        fixture_path,
    )
    .await;
}

// Blacksmith Armors
pub async fn verify_blacksmith_armor_count(expected_count: i64) {
    verify_entity_count(&BLACKSMITH_ARMOR_CONFIG, expected_count).await;
}

pub async fn verify_blacksmith_armor_data_matches_fixture(
    armor_id: i32,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(
        &BLACKSMITH_ARMOR_CONFIG,
        armor_id,
        fixture_path,
    )
    .await;
}

// Faction Armors
pub async fn verify_faction_armor_count(expected_count: i64) {
    verify_entity_count(&FACTION_ARMOR_CONFIG, expected_count).await;
}

pub async fn verify_faction_armor_data_matches_fixture(
    armor_id: i32,
    fixture_path: PathBuf,
) {
    verify_entity_data_matches_fixture(
        &FACTION_ARMOR_CONFIG,
        armor_id,
        fixture_path,
    )
    .await;
}
