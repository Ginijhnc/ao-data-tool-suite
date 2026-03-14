//! Unit tests for Server.ini GM name parsing.

use std::path::PathBuf;

use ao_data_to_sql::parsers::server_ini::parse_gm_names;

use crate::common::{ALKON_VB6_FIXTURES, FIXTURES_DIR};
use crate::e2e::entity_configs::SERVER_INI_FIXTURES;

/// Verifies that the ini/Server.ini file is parsed correctly for GM names.
#[test]
fn parse_ini_server_ini_extracts_gm_names() {
    let server_ini_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("ini")
        .join("Server.ini");

    let result = parse_gm_names(&server_ini_path).unwrap();

    let expected_gms = [
        "PEPITO",
        "GS",
        "RAMZA",
        "RESISTENCIA",
        "ZAVETH",
        "MISS LUCHITAZ",
    ];

    for gm in expected_gms {
        assert!(
            result.contains(gm),
            "Expected GM '{gm}' not found in result"
        );
    }

    assert!(
        !result.contains("JORGE"),
        "Result should not contain regular player names"
    );
}

/// Verifies that missing sections are handled gracefully.
#[test]
fn parse_gm_names_handles_missing_sections() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES_DIR)
        .join(SERVER_INI_FIXTURES)
        .join(ALKON_VB6_FIXTURES)
        .join("missing_sections.ini");

    let result = parse_gm_names(&fixture_path).unwrap();

    assert!(result.contains("ADMIN_USER"));
    assert!(result.contains("ANOTHER_ADMIN"));
    assert!(result.contains("GOD_USER"));
    assert_eq!(result.len(), 3);
}

/// Verifies that empty and whitespace-only values are excluded.
#[test]
fn parse_gm_names_handles_empty_values() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES_DIR)
        .join(SERVER_INI_FIXTURES)
        .join(ALKON_VB6_FIXTURES)
        .join("empty_values.ini");

    let result = parse_gm_names(&fixture_path).unwrap();

    assert!(result.contains("VALID_ADMIN"));
    assert!(result.contains("VALID_GOD"));
    assert!(result.contains("VALID_SEMI"));
    assert!(result.contains("VALID_RM"));
    assert_eq!(result.len(), 4);
}

/// Verifies that all names are uppercased.
#[test]
fn parse_gm_names_uppercases_names() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES_DIR)
        .join(SERVER_INI_FIXTURES)
        .join(ALKON_VB6_FIXTURES)
        .join("mixed_case.ini");

    let result = parse_gm_names(&fixture_path).unwrap();

    assert!(result.contains("PEPITO"));
    assert!(result.contains("JUAN"));
    assert!(result.contains("GS"));
    assert!(result.contains("MARIA"));
    assert!(result.contains("PEDRO"));
    assert!(result.contains("LUIS"));
    assert!(result.contains("ANA"));
    assert_eq!(result.len(), 7);
}

/// Verifies that a non-existent file returns an error.
#[test]
fn parse_gm_names_file_not_found() {
    let non_existent_path = PathBuf::from("/nonexistent/path/Server.ini");

    let result = parse_gm_names(&non_existent_path);

    assert!(result.is_err());
}
