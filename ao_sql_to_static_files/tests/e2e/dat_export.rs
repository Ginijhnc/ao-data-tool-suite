//! Tests for the DAT table export pipeline.
//!
//! Exercises the full flow from database insertion through flattening
//! and serialization for all dat table types: named, id-only, balance, and maps.

use ao_sql_to_static_files::queries::build_all_dat_exports;
use ao_sql_to_static_files::serialization::write_export_file;

use ao_shared::testing::setup_test_db;

use crate::common::{
    insert_balance_row, insert_id_only_dat_row, insert_map_row,
    insert_named_dat_row,
};

// Verifies that named tables (npcs, objects, spells) produce flat JSON with
// ID injected from the DB column and all JSONB keys at the same level.
// Also verifies NOMBRE -> NAME normalization for spells.
#[tokio::test]
async fn dat_export_named_table_produces_flat_json() {
    let (_container, pool) = setup_test_db().await;
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    insert_named_dat_row(
        &pool,
        "npcs",
        1,
        "Bandido",
        serde_json::json!({
            "NAME": "Bandido",
            "NPCTYPE": "1",
            "BODY": "26",
            "HEAD": "7"
        }),
    )
    .await;
    insert_named_dat_row(
        &pool,
        "npcs",
        2,
        "Dragon",
        serde_json::json!({
            "NAME": "Dragon",
            "NPCTYPE": "2",
            "BODY": "100"
        }),
    )
    .await;

    let entries = build_all_dat_exports(&pool).await.expect("build failed");
    let npcs_entry = entries
        .iter()
        .find(|e| e.file_key == "dat/npcs.json")
        .expect("npcs entry not found");

    write_export_file(
        output_dir.path(),
        &npcs_entry.file_key,
        &npcs_entry.json_content,
    )
    .expect("write failed");

    let content =
        std::fs::read_to_string(output_dir.path().join("dat/npcs.json"))
            .expect("read failed");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("invalid JSON");

    assert!(
        parsed
            .get("generated_at")
            .expect("missing generated_at")
            .is_string(),
        "should have generated_at timestamp"
    );

    let data = parsed
        .get("data")
        .expect("missing data")
        .as_array()
        .expect("data should be array");
    assert_eq!(data.len(), 2, "should contain both NPCs");

    let first = data.first().expect("expected first entry");
    assert_eq!(
        first.get("ID").expect("missing ID"),
        1,
        "ID should be integer from DB"
    );
    assert_eq!(
        first.get("NAME").expect("missing NAME"),
        "Bandido",
        "NAME from JSONB"
    );
    assert_eq!(
        first.get("NPCTYPE").expect("missing NPCTYPE"),
        "1",
        "JSONB key flattened"
    );
    assert_eq!(
        first.get("BODY").expect("missing BODY"),
        "26",
        "JSONB key flattened"
    );
    assert!(first.get("id").is_none(), "lowercase id should not exist");
    assert!(
        first.get("name").is_none(),
        "lowercase name should not exist"
    );

    let second = data.get(1).expect("expected second entry");
    assert_eq!(second.get("ID").expect("missing ID"), 2, "ordered by ID");
    assert_eq!(second.get("NAME").expect("missing NAME"), "Dragon");
}

// Verifies that id-only tables produce flat JSON with ID injected
// and no NAME field present.
#[tokio::test]
async fn dat_export_id_only_table_produces_flat_json() {
    let (_container, pool) = setup_test_db().await;

    insert_id_only_dat_row(
        &pool,
        "carpenter_objects",
        1,
        serde_json::json!({
            "OBJ_INDEX": "132",
            "_COMMENT": "Casco de hierro"
        }),
    )
    .await;
    insert_id_only_dat_row(
        &pool,
        "carpenter_objects",
        2,
        serde_json::json!({
            "OBJ_INDEX": "245",
            "_COMMENT": "Escudo de madera"
        }),
    )
    .await;

    let entries = build_all_dat_exports(&pool).await.expect("build failed");
    let entry = entries
        .iter()
        .find(|e| e.file_key == "dat/carpenter-objects.json")
        .expect("carpenter-objects entry not found");

    let parsed: serde_json::Value =
        serde_json::from_str(&entry.json_content).expect("invalid JSON");
    let data = parsed
        .get("data")
        .expect("missing data")
        .as_array()
        .expect("data should be array");
    assert_eq!(data.len(), 2, "should contain both carpenter objects");

    let first = data.first().expect("expected first entry");
    assert_eq!(first.get("ID").expect("missing ID"), 1);
    assert_eq!(first.get("OBJ_INDEX").expect("missing OBJ_INDEX"), "132");
    assert!(
        first.get("NAME").is_none(),
        "id-only tables should not have NAME"
    );
    assert!(first.get("id").is_none(), "lowercase id should not exist");
}

// Verifies that the balance table produces flat JSON with SECTION injected
// (uppercase) and entries ordered alphabetically by section.
#[tokio::test]
async fn dat_export_balance_table_produces_flat_json() {
    let (_container, pool) = setup_test_db().await;

    insert_balance_row(
        &pool,
        "MODEVASION",
        serde_json::json!({
            "GUERRERO": "1",
            "MAGO": "0.4",
            "LADRON": "1.1"
        }),
    )
    .await;
    insert_balance_row(
        &pool,
        "DISTRIBUCION",
        serde_json::json!({
            "E1": "0",
            "E2": "30",
            "E3": "40"
        }),
    )
    .await;

    let entries = build_all_dat_exports(&pool).await.expect("build failed");
    let entry = entries
        .iter()
        .find(|e| e.file_key == "dat/balance.json")
        .expect("balance entry not found");

    let parsed: serde_json::Value =
        serde_json::from_str(&entry.json_content).expect("invalid JSON");
    let data = parsed
        .get("data")
        .expect("missing data")
        .as_array()
        .expect("data should be array");
    assert_eq!(data.len(), 2, "should contain both balance sections");

    // Ordered alphabetically: DISTRIBUCION before MODEVASION
    let first = data.first().expect("expected first entry");
    assert_eq!(
        first.get("SECTION").expect("missing SECTION"),
        "DISTRIBUCION",
        "SECTION should be uppercase"
    );
    assert_eq!(first.get("E1").expect("missing E1"), "0");
    assert!(
        first.get("section").is_none(),
        "lowercase section should not exist"
    );
    assert!(first.get("id").is_none(), "balance should not have id");

    let second = data.get(1).expect("expected second entry");
    assert_eq!(
        second.get("SECTION").expect("missing SECTION"),
        "MODEVASION"
    );
    assert_eq!(second.get("GUERRERO").expect("missing GUERRERO"), "1");
}

// Verifies that maps flatten the MAPA{n} section to top level while keeping
// SONIDOS/SONIDO{n} sections as nested objects.
#[tokio::test]
async fn dat_export_map_flattens_main_section_and_keeps_sounds_nested() {
    let (_container, pool) = setup_test_db().await;

    insert_map_row(
        &pool,
        1,
        "Ciudad de Ullathorpe",
        serde_json::json!({
            "MAPA1": {
                "NAME": "Ciudad de Ullathorpe",
                "PK": "1",
                "ZONA": "CIUDAD",
                "TERRENO": "BOSQUE",
                "MUSICNUM": "4"
            },
            "SONIDOS": { "CANTIDAD": "2" },
            "SONIDO1": { "SONIDO": "21", "PROBABILIDAD": "20" },
            "SONIDO2": { "SONIDO": "22", "PROBABILIDAD": "20" }
        }),
    )
    .await;

    let entries = build_all_dat_exports(&pool).await.expect("build failed");
    let entry = entries
        .iter()
        .find(|e| e.file_key == "dat/maps.json")
        .expect("maps entry not found");

    let parsed: serde_json::Value =
        serde_json::from_str(&entry.json_content).expect("invalid JSON");
    let data = parsed
        .get("data")
        .expect("missing data")
        .as_array()
        .expect("data should be array");
    assert_eq!(data.len(), 1, "should contain one map");

    let map_entry = data.first().expect("expected map entry");

    // ID injected from DB column
    assert_eq!(map_entry.get("ID").expect("missing ID"), 1);

    // MAPA1 section flattened to top level
    assert_eq!(
        map_entry.get("NAME").expect("missing NAME"),
        "Ciudad de Ullathorpe"
    );
    assert_eq!(map_entry.get("PK").expect("missing PK"), "1");
    assert_eq!(map_entry.get("ZONA").expect("missing ZONA"), "CIUDAD");
    assert_eq!(map_entry.get("TERRENO").expect("missing TERRENO"), "BOSQUE");
    assert_eq!(map_entry.get("MUSICNUM").expect("missing MUSICNUM"), "4");

    // MAPA1 wrapper key should not exist in output
    assert!(
        map_entry.get("MAPA1").is_none(),
        "MAPA1 wrapper should be removed"
    );

    // Sound sections remain as nested objects
    let sonidos = map_entry.get("SONIDOS").expect("missing SONIDOS");
    assert_eq!(sonidos.get("CANTIDAD").expect("missing CANTIDAD"), "2");

    let sound1 = map_entry.get("SONIDO1").expect("missing SONIDO1");
    assert_eq!(sound1.get("SONIDO").expect("missing SONIDO"), "21");
    assert_eq!(
        sound1.get("PROBABILIDAD").expect("missing PROBABILIDAD"),
        "20"
    );

    let sound2 = map_entry.get("SONIDO2").expect("missing SONIDO2");
    assert_eq!(sound2.get("SONIDO").expect("missing SONIDO"), "22");
}

// Verifies that build_all_dat_exports produces exactly 9 entries with
// the expected file keys.
#[tokio::test]
async fn dat_export_produces_all_nine_tables() {
    let (_container, pool) = setup_test_db().await;

    // Insert one row per table to avoid empty-table edge cases
    insert_named_dat_row(
        &pool,
        "npcs",
        1,
        "NPC1",
        serde_json::json!({"NAME": "NPC1"}),
    )
    .await;
    insert_named_dat_row(
        &pool,
        "objects",
        1,
        "Obj1",
        serde_json::json!({"NAME": "Obj1"}),
    )
    .await;
    insert_named_dat_row(
        &pool,
        "spells",
        1,
        "Spell1",
        serde_json::json!({"NOMBRE": "Spell1"}),
    )
    .await;
    insert_map_row(
        &pool,
        1,
        "Map1",
        serde_json::json!({"MAPA1": {"NAME": "Map1"}}),
    )
    .await;
    insert_id_only_dat_row(
        &pool,
        "carpenter_objects",
        1,
        serde_json::json!({"OBJ_INDEX": "1"}),
    )
    .await;
    insert_id_only_dat_row(
        &pool,
        "blacksmith_armors",
        1,
        serde_json::json!({"OBJ_INDEX": "1"}),
    )
    .await;
    insert_id_only_dat_row(
        &pool,
        "blacksmith_weapons",
        1,
        serde_json::json!({"OBJ_INDEX": "1"}),
    )
    .await;
    insert_id_only_dat_row(
        &pool,
        "faction_armors",
        1,
        serde_json::json!({"OBJ_INDEX": "1"}),
    )
    .await;
    insert_balance_row(&pool, "TEST", serde_json::json!({"KEY": "val"})).await;

    let entries = build_all_dat_exports(&pool).await.expect("build failed");
    assert_eq!(entries.len(), 9, "should produce exactly 9 dat entries");

    let expected_keys = [
        "dat/npcs.json",
        "dat/objects.json",
        "dat/spells.json",
        "dat/maps.json",
        "dat/carpenter-objects.json",
        "dat/blacksmith-armors.json",
        "dat/blacksmith-weapons.json",
        "dat/faction-armors.json",
        "dat/balance.json",
    ];
    for key in expected_keys {
        assert!(
            entries.iter().any(|e| e.file_key == key),
            "missing expected file key: {key}"
        );
    }
}

// Verifies that empty tables produce entries with empty data arrays
// rather than errors.
#[tokio::test]
async fn dat_export_empty_tables_produce_empty_data_arrays() {
    let (_container, pool) = setup_test_db().await;

    let entries = build_all_dat_exports(&pool).await.expect("build failed");
    assert_eq!(
        entries.len(),
        9,
        "should produce 9 entries even when tables are empty"
    );

    for entry in &entries {
        let parsed: serde_json::Value =
            serde_json::from_str(&entry.json_content).expect("invalid JSON");
        let data = parsed
            .get("data")
            .expect("missing data")
            .as_array()
            .expect("data should be array");
        assert!(
            data.is_empty(),
            "empty table {} should produce empty data array",
            entry.file_key
        );
    }
}

// Verifies that spells normalize NOMBRE to NAME for consistency
// with other named tables.
#[tokio::test]
async fn dat_export_spells_normalize_nombre_to_name() {
    let (_container, pool) = setup_test_db().await;

    insert_named_dat_row(
        &pool,
        "spells",
        1,
        "Paralizar",
        serde_json::json!({
            "NOMBRE": "Paralizar",
            "TIPO": "2",
            "MANAREQUERIDO": "450"
        }),
    )
    .await;

    let entries = build_all_dat_exports(&pool).await.expect("build failed");
    let entry = entries
        .iter()
        .find(|e| e.file_key == "dat/spells.json")
        .expect("spells entry not found");

    let parsed: serde_json::Value =
        serde_json::from_str(&entry.json_content).expect("invalid JSON");
    let data = parsed
        .get("data")
        .expect("missing data")
        .as_array()
        .expect("data should be array");

    let spell = data.first().expect("expected spell entry");
    assert_eq!(
        spell.get("NAME").expect("missing NAME"),
        "Paralizar",
        "NOMBRE should be normalized to NAME"
    );
    assert!(
        spell.get("NOMBRE").is_none(),
        "NOMBRE should be removed after normalization"
    );
    assert_eq!(spell.get("ID").expect("missing ID"), 1);
    assert_eq!(spell.get("TIPO").expect("missing TIPO"), "2");
}
