//! End-to-end tests for Hechizos.dat parsing and database import.
//!
//! Tests verify spell count, data integrity, and incremental update behavior.

use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, test_incremental_addition,
    test_incremental_modification,
};
use crate::e2e::entity_configs::{
    SPELL_CONFIG, SPELL_FIXTURES, verify_spell_count,
    verify_spell_data_matches_fixture,
};

/// Verifies the number of spells in the database matches expected count.
#[tokio::test]
async fn spell_count_matches_expected() {
    verify_spell_count(50).await;
}

/// Verifies HECHIZO9 (Paralizar - status effect spell) data matches expected fixture.
#[tokio::test]
async fn spell_9_paralizar_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(SPELL_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("HECHIZO9.json");
    verify_spell_data_matches_fixture(9, fixture).await;
}

/// Verifies HECHIZO36 (Tormenta Pretoriana - damage spell) data matches expected fixture.
#[tokio::test]
async fn spell_36_tormenta_pretoriana_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(SPELL_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("HECHIZO36.json");
    verify_spell_data_matches_fixture(36, fixture).await;
}

/// Verifies incremental update: parser detects modified Hechizos.dat and updates existing spell.
#[tokio::test]
async fn incremental_update_detects_modified_spell() {
    test_incremental_modification(
        &SPELL_CONFIG,
        "dat",
        "Hechizos.dat",
        50,
        "Nombre=Antídoto Mágico",
        "Nombre=Antidoto MODIFICADO",
        |pool| {
            Box::pin(async move {
                let (updated_name,): (String,) =
                    sqlx::query_as("SELECT name FROM spells WHERE id = 1")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch HECHIZO1");

                assert_eq!(
                    updated_name, "Antidoto MODIFICADO",
                    "HECHIZO1 name should be updated after modification"
                );
            })
        },
    )
    .await;
}

/// Verifies incremental update: parser detects new spell added to Hechizos.dat.
#[tokio::test]
async fn incremental_update_detects_new_spell() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(SPELL_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_spell_51.txt");

    test_incremental_addition(
        &SPELL_CONFIG,
        "dat",
        "Hechizos.dat",
        50,
        51,
        fixture,
        |pool| {
            Box::pin(async move {
                let (spell_name,): (String,) =
                    sqlx::query_as("SELECT name FROM spells WHERE id = 51")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch HECHIZO51");

                assert_eq!(
                    spell_name, "Hechizo de prueba incremental",
                    "HECHIZO51 should have correct name"
                );
            })
        },
    )
    .await;
}
