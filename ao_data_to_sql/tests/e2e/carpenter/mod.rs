//! End-to-end tests for ObjCarpintero.dat parsing and database import.
//!
//! Tests verify carpenter object count, data integrity, and incremental update behavior.

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, test_incremental_addition,
    test_incremental_modification,
};
use crate::e2e::entity_configs::{
    CARPENTER_CONFIG, CARPENTER_FIXTURES, verify_carpenter_object_count,
    verify_carpenter_object_data_matches_fixture,
};

/// Verifies the number of carpenter objects matches expected count.
#[tokio::test]
async fn carpenter_object_count_matches_expected() {
    verify_carpenter_object_count(34).await;
}

/// Verifies carpenter object 1 (Flecha - Index 480) data matches expected fixture.
#[tokio::test]
async fn carpenter_object_1_flecha_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(CARPENTER_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("CARPENTEROBJ1.json");
    verify_carpenter_object_data_matches_fixture(1, fixture).await;
}

/// Verifies carpenter object 10 (Arco de Cazador - Index 665) data matches expected fixture.
#[tokio::test]
async fn carpenter_object_10_arco_cazador_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(CARPENTER_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("CARPENTEROBJ10.json");
    verify_carpenter_object_data_matches_fixture(10, fixture).await;
}

/// Verifies incremental update: parser detects modified ObjCarpintero.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_carpenter_object() {
    test_incremental_modification(
        &CARPENTER_CONFIG,
        "dat",
        "ObjCarpintero.dat",
        34,
        "[OBJ1] 'Flecha",
        "[OBJ1] 'Flecha MODIFICADA",
        |pool| {
            Box::pin(async move {
                let (updated_data,): (serde_json::Value,) =
                    sqlx::query_as("SELECT data FROM carpenter_objects WHERE id = 1")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch carpenter object 1");

                let comment = updated_data
                    .get("_COMMENT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                assert_eq!(
                    comment, "Flecha MODIFICADA",
                    "Carpenter object 1 comment should be updated after modification"
                );
            })
        },
    )
    .await;
}

/// Verifies incremental update: parser detects new carpenter object added.
#[tokio::test]
async fn incremental_update_detects_new_carpenter_object() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(CARPENTER_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_carpenter_obj_35.txt");

    test_incremental_addition(
        &CARPENTER_CONFIG,
        "dat",
        "ObjCarpintero.dat",
        34,
        35,
        fixture,
        |pool| {
            Box::pin(async move {
                let (obj_data,): (serde_json::Value,) = sqlx::query_as(
                    "SELECT data FROM carpenter_objects WHERE id = 35",
                )
                .fetch_one(pool)
                .await
                .expect("failed to fetch OBJ35");

                let index = obj_data
                    .get("OBJ_INDEX")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                assert_eq!(
                    index, "999",
                    "OBJ35 should have correct OBJ_INDEX"
                );
            })
        },
    )
    .await;
}
