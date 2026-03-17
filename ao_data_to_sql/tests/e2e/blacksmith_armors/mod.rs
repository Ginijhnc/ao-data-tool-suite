//! End-to-end tests for ArmadurasHerrero.dat parsing and database import.
//!
//! Tests verify blacksmith armor count, data integrity, and incremental update behavior.

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, test_incremental_addition,
    test_incremental_modification,
};
use crate::e2e::entity_configs::{
    BLACKSMITH_ARMOR_CONFIG, BLACKSMITH_ARMORS_FIXTURES,
    verify_blacksmith_armor_count,
    verify_blacksmith_armor_data_matches_fixture,
};

/// Verifies the number of blacksmith armors matches expected count.
#[tokio::test]
async fn blacksmith_armor_count_matches_expected() {
    verify_blacksmith_armor_count(61).await;
}

/// Verifies blacksmith armor 1 (Casco de hierro - Index 132) data matches expected fixture.
#[tokio::test]
async fn blacksmith_armor_1_casco_hierro_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("BLACKSMITHARMOR1.json");
    verify_blacksmith_armor_data_matches_fixture(1, fixture).await;
}

/// Verifies blacksmith armor 41 (Anillo Magico - Index 697) data matches expected fixture.
#[tokio::test]
async fn blacksmith_armor_41_anillo_magico_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("BLACKSMITHARMOR41.json");
    verify_blacksmith_armor_data_matches_fixture(41, fixture).await;
}

/// Verifies incremental update: parser detects modified ArmadurasHerrero.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_blacksmith_armor() {
    test_incremental_modification(
        &BLACKSMITH_ARMOR_CONFIG,
        "dat",
        "ArmadurasHerrero.dat",
        61,
        "[Armadura1] ' Casco de hierro",
        "[Armadura1] ' Casco de hierro MODIFICADO",
        |pool| {
            Box::pin(async move {
                let (updated_data,): (serde_json::Value,) =
                    sqlx::query_as("SELECT data FROM blacksmith_armors WHERE id = 1")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch blacksmith armor 1");

                let comment = updated_data
                    .get("_COMMENT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                assert_eq!(
                    comment, "Casco de hierro MODIFICADO",
                    "Blacksmith armor 1 comment should be updated after modification"
                );
            })
        },
    )
    .await;
}

/// Verifies incremental update: parser detects new blacksmith armor added.
#[tokio::test]
async fn incremental_update_detects_new_blacksmith_armor() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BLACKSMITH_ARMORS_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_blacksmith_armor_62.txt");

    test_incremental_addition(
        &BLACKSMITH_ARMOR_CONFIG,
        "dat",
        "ArmadurasHerrero.dat",
        61,
        62,
        fixture,
        |pool| {
            Box::pin(async move {
                let (armor_data,): (serde_json::Value,) =
                    sqlx::query_as("SELECT data FROM blacksmith_armors WHERE id = 62")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch Armadura62");

                let index = armor_data
                    .get("OBJ_INDEX")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0);
                assert_eq!(index, 999, "Armadura62 should have correct OBJ_INDEX");

                assert!(
                    armor_data.get("_COMMENT").is_none(),
                    "Armadura62 should not have _COMMENT field (no comment in source)"
                );
            })
        },
    )
    .await;
}
