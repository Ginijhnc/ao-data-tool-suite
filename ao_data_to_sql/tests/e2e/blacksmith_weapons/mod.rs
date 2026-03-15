//! End-to-end tests for ArmasHerrero.dat parsing and database import.
//!
//! Tests verify blacksmith weapon count, data integrity, and incremental update behavior.

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, test_incremental_addition,
    test_incremental_modification,
};
use crate::e2e::entity_configs::{
    BLACKSMITH_WEAPON_CONFIG, BLACKSMITH_WEAPONS_FIXTURES,
    verify_blacksmith_weapon_count,
    verify_blacksmith_weapon_data_matches_fixture,
};

/// Verifies the number of blacksmith weapons matches expected count.
#[tokio::test]
async fn blacksmith_weapon_count_matches_expected() {
    verify_blacksmith_weapon_count(26).await;
}

/// Verifies blacksmith weapon 1 (Daga - Index 15) data matches expected fixture.
#[tokio::test]
async fn blacksmith_weapon_1_daga_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_WEAPONS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("BLACKSMITHWEAPON1.json");
    verify_blacksmith_weapon_data_matches_fixture(1, fixture).await;
}

/// Verifies blacksmith weapon 16 (Hacha de Barbaro - Index 159) data matches expected fixture.
#[tokio::test]
async fn blacksmith_weapon_16_hacha_barbaro_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_WEAPONS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("BLACKSMITHWEAPON16.json");
    verify_blacksmith_weapon_data_matches_fixture(16, fixture).await;
}

/// Verifies incremental update: parser detects modified ArmasHerrero.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_blacksmith_weapon() {
    test_incremental_modification(
        &BLACKSMITH_WEAPON_CONFIG,
        "dat",
        "ArmasHerrero.dat",
        26,
        "[Arma1] ' Daga",
        "[Arma1] ' Daga MODIFICADO",
        |pool| {
            Box::pin(async move {
                let (updated_data,): (serde_json::Value,) =
                    sqlx::query_as("SELECT data FROM blacksmith_weapons WHERE id = 1")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch blacksmith weapon 1");

                let comment = updated_data
                    .get("_COMMENT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                assert_eq!(
                    comment, "Daga MODIFICADO",
                    "Blacksmith weapon 1 comment should be updated after modification"
                );
            })
        },
    )
    .await;
}

/// Verifies incremental update: parser detects new blacksmith weapon added.
#[tokio::test]
async fn incremental_update_detects_new_blacksmith_weapon() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_WEAPONS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_blacksmith_weapon_27.txt");

    test_incremental_addition(
        &BLACKSMITH_WEAPON_CONFIG,
        "dat",
        "ArmasHerrero.dat",
        26,
        27,
        fixture,
        |pool| {
            Box::pin(async move {
                let (weapon_data,): (serde_json::Value,) =
                    sqlx::query_as("SELECT data FROM blacksmith_weapons WHERE id = 27")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch Arma27");

                let index = weapon_data
                    .get("OBJ_INDEX")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                assert_eq!(index, "999", "Arma27 should have correct OBJ_INDEX");

                assert!(
                    weapon_data.get("_COMMENT").is_none(),
                    "Arma27 should not have _COMMENT field (no comment in source)"
                );
            })
        },
    )
    .await;
}
