//! End-to-end tests for Balance.dat parsing and database import.
//!
//! Tests verify balance section count, data integrity, and incremental update behavior.

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, test_incremental_addition,
    test_incremental_modification,
};
use crate::e2e::entity_configs::{
    BALANCE_CONFIG, BALANCE_FIXTURES, verify_balance_count,
    verify_balance_data_matches_fixture,
};

/// Verifies the number of balance sections matches expected count.
#[tokio::test]
async fn balance_count_matches_expected() {
    verify_balance_count(14).await;
}

/// Verifies MODEVASION section data matches expected fixture.
#[tokio::test]
async fn balance_modevasion_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("MODEVASION.json");
    verify_balance_data_matches_fixture("MODEVASION", fixture).await;
}

/// Verifies DISTRIBUCION section data matches expected fixture.
#[tokio::test]
async fn balance_distribucion_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("DISTRIBUCION.json");
    verify_balance_data_matches_fixture("DISTRIBUCION", fixture).await;
}

/// Verifies PARTY section data matches expected fixture.
#[tokio::test]
async fn balance_party_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("PARTY.json");
    verify_balance_data_matches_fixture("PARTY", fixture).await;
}

/// Verifies RECOMPENSAFACCION section data matches expected fixture.
#[tokio::test]
async fn balance_recompensafaccion_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("RECOMPENSAFACCION.json");
    verify_balance_data_matches_fixture("RECOMPENSAFACCION", fixture).await;
}

/// Verifies incremental update: parser detects modified Balance.dat and updates.
#[tokio::test]
async fn incremental_update_detects_modified_balance_section() {
    test_incremental_modification(
        &BALANCE_CONFIG,
        "dat",
        "Balance.dat",
        14,
        "[MODEVASION]",
        "[MODEVASION] ' Balance Modificado",
        |pool| {
            Box::pin(async move {
                let (updated_data,): (serde_json::Value,) = sqlx::query_as(
                    "SELECT data FROM balance WHERE section = 'MODEVASION'",
                )
                .fetch_one(pool)
                .await
                .expect("failed to fetch MODEVASION section");

                let comment = updated_data
                    .get("_COMMENT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                assert_eq!(
                    comment, "Balance Modificado",
                    "MODEVASION comment should be updated after modification"
                );
            })
        },
    )
    .await;
}

/// Verifies incremental update: parser detects new balance section added.
#[tokio::test]
async fn incremental_update_detects_new_balance_section() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(BALANCE_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_balance_modregeneracion.txt");

    test_incremental_addition(
        &BALANCE_CONFIG,
        "dat",
        "Balance.dat",
        14,
        "MODREGENERACION",
        fixture,
        |pool| {
            Box::pin(async move {
                let (section_data,): (serde_json::Value,) = sqlx::query_as(
                "SELECT data FROM balance WHERE section = 'MODREGENERACION'",
            )
            .fetch_one(pool)
             .await
            .expect("failed to fetch MODREGENERACION");

                let guerrero = section_data
                    .get("GUERRERO")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                assert_eq!(
                    guerrero, "1.05",
                    "MODREGENERACION should have correct GUERRERO value"
                );

                let comment = section_data
                    .get("_COMMENT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                assert_eq!(
                    comment, "Regeneracion De Prueba",
                    "MODREGENERACION should have correct _COMMENT"
                );
            })
        },
    )
    .await;
}
