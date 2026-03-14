use crate::common::{
    ALKON_VB6_FIXTURES, DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir,
};
use crate::e2e::entity_configs::{
    CHARACTER_FIXTURES, verify_character_count,
    verify_character_data_matches_fixture,
};

/// Verifies that the number of characters in the database matches the expected amount.
#[tokio::test]
async fn character_count_matches_expected() {
    verify_character_count(4).await;
}

/// Verifies that the CLERIGO character data (Dakara C++) matches the expected fixture
#[tokio::test]
async fn clerigo_dakara_cpp_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(CHARACTER_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("CLERIGO.json");
    verify_character_data_matches_fixture("CLERIGO", fixture).await;
}

/// Verifies that the JORGE character data (Alkon VB6) matches the expected fixture
#[tokio::test]
async fn jorge_alkon_vb6_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(CHARACTER_FIXTURES)
        .join(ALKON_VB6_FIXTURES)
        .join("JORGE.json");
    verify_character_data_matches_fixture("JORGE", fixture).await;
}
