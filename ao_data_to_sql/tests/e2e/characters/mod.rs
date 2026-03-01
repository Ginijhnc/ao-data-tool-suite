mod helpers;

use helpers::{verify_character_count, verify_character_data_matches_fixture};

/// Verifies that the number of characters in the database matches the expected amount.
#[tokio::test]
async fn character_count_matches_expected() {
    verify_character_count(3).await;
}

/// Verifies that the CLERIGO character data (Dakara C++) matches the expected fixture
#[tokio::test]
async fn clerigo_dakara_cpp_data_matches_expected_fixture() {
    let fixture = crate::common::crate_dir()
        .join("tests/fixtures/characters/dakara_cpp/CLERIGO.json");
    verify_character_data_matches_fixture("CLERIGO", fixture).await;
}
