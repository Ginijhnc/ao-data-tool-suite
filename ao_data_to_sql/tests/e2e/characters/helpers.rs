use std::path::PathBuf;
use std::process::Command;

use pretty_assertions::assert_eq;

use crate::common::{crate_dir, setup_test_db, with_test_db_env};

/// Subdirectory of fixtures for characters
pub const CHARACTER_FIXTURES: &str = "characters";

/// Helper that verifies the number of characters in the database
pub async fn verify_character_count(expected_character_count: i64) {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let charfile_dir = crate_dir().join("Charfile");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("CHARFILE_DIR", charfile_dir.to_str().unwrap());

    let status = cmd.status().expect("fallo al ejecutar ao_data_to_sql");
    assert!(status.success(), "ao_data_to_sql termino con error");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM characters")
        .fetch_one(&pool)
        .await
        .expect("fallo la query COUNT(*)");

    assert_eq!(
        count.0, expected_character_count,
        "Esperados {} personajes en la tabla characters, encontrados {}",
        expected_character_count, count.0
    );
}

/// Helper that verifies a character's data matches a JSON fixture
pub async fn verify_character_data_matches_fixture(
    character_name: &str,
    fixture_path: PathBuf,
) {
    let (container, pool) = setup_test_db().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let charfile_dir = crate_dir().join("Charfile");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ao_data_to_sql"));
    with_test_db_env(&mut cmd, host_port)
        .env("CHARFILE_DIR", charfile_dir.to_str().unwrap());

    let status = cmd.status().expect("fallo al ejecutar ao_data_to_sql");
    assert!(status.success(), "ao_data_to_sql termino con error");

    let (actual_data,): (serde_json::Value,) =
        sqlx::query_as("SELECT data FROM characters WHERE name = $1")
            .bind(character_name)
            .fetch_one(&pool)
            .await
            .expect("fallo al obtener datos del personaje");

    let fixture_content = std::fs::read_to_string(&fixture_path)
        .expect("fallo al leer archivo fixture");
    let expected_data: serde_json::Value =
        serde_json::from_str(&fixture_content)
            .expect("fallo al parsear fixture JSON");

    assert_eq!(
        actual_data, expected_data,
        "Los datos del personaje {character_name} no coinciden con el fixture"
    );
}
