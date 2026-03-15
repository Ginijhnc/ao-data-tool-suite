//! End-to-end tests for ArmadurasFaccionarias.dat parsing and database import.
//!
//! Tests verify faction armor count, data integrity, and incremental update behavior.

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, test_incremental_addition,
    test_incremental_modification,
};
use crate::e2e::entity_configs::{
    FACTION_ARMOR_CONFIG, FACTION_ARMORS_FIXTURES, verify_faction_armor_count,
    verify_faction_armor_data_matches_fixture,
};

/// Verifies the number of faction armor classes matches expected count.
#[tokio::test]
async fn faction_armor_count_matches_expected() {
    verify_faction_armor_count(12).await;
}

/// Verifies faction armor 1 (Mago) data matches expected fixture.
#[tokio::test]
async fn faction_armor_1_mago_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(FACTION_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("CLASE1.json");
    verify_faction_armor_data_matches_fixture(1, fixture).await;
}

/// Verifies faction armor 3 (Guerrero) data matches expected fixture.
#[tokio::test]
async fn faction_armor_3_guerrero_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(FACTION_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("CLASE3.json");
    verify_faction_armor_data_matches_fixture(3, fixture).await;
}

/// Verifies incremental update: parser detects modified ArmadurasFaccionarias.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_faction_armor() {
    test_incremental_modification(
        &FACTION_ARMOR_CONFIG,
        "dat",
        "ArmadurasFaccionarias.dat",
        12,
        "[CLASE3] ' Guerrero",
        "[CLASE3] ' Guerrero MODIFICADO",
        |pool| {
            Box::pin(async move {
                let (updated_data,): (serde_json::Value,) =
                    sqlx::query_as("SELECT data FROM faction_armors WHERE id = 3")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch faction armor 3");

                let comment = updated_data
                    .get("_COMMENT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                assert_eq!(
                    comment, "Guerrero MODIFICADO",
                    "Faction armor 3 comment should be updated after modification"
                );
            })
        },
    )
    .await;
}

/// Verifies incremental update: parser detects new faction armor class added.
#[tokio::test]
async fn incremental_update_detects_new_faction_armor() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(FACTION_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_faction_armor_13.txt");

    test_incremental_addition(
        &FACTION_ARMOR_CONFIG,
        "dat",
        "ArmadurasFaccionarias.dat",
        12,
        13,
        fixture,
        |pool| {
            Box::pin(async move {
                let (armor_data,): (serde_json::Value,) =
                    sqlx::query_as("SELECT data FROM faction_armors WHERE id = 13")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch CLASE13");

                let def_min = armor_data
                    .get("DEFMINARMYALTO")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                assert_eq!(def_min, "675", "CLASE13 should have correct DEFMINARMYALTO");

                assert!(
                    armor_data.get("_COMMENT").is_none(),
                    "CLASE13 should not have _COMMENT field (no comment in source)"
                );
            })
        },
    )
    .await;
}
