use crate::common::{
    DAKARA_CPP_FIXTURES, FIXTURES_DIR, crate_dir, test_incremental_addition,
    test_incremental_modification,
};
use crate::e2e::entity_configs::{
    OBJECT_CONFIG, OBJECT_FIXTURES as OBJ_FIXTURES, verify_object_count,
    verify_object_data_matches_fixture,
};

/// Verifies the number of objects in the database matches expected count.
#[tokio::test]
async fn object_count_matches_expected() {
    verify_object_count(1122).await;
}

/// Verifies OBJ1 (Manzana Roja - food item) data matches expected fixture.
#[tokio::test]
async fn object_1_manzana_roja_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(OBJ_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("OBJ1.json");
    verify_object_data_matches_fixture(1, fixture).await;
}

/// Verifies OBJ129 (Hacha de Guerra Dos Filos - weapon with class prohibitions) data matches expected fixture.
#[tokio::test]
async fn object_129_hacha_guerra_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(OBJ_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("OBJ129.json");
    verify_object_data_matches_fixture(129, fixture).await;
}

/// Verifies OBJ142 (Puerta de madera cerrada - door with key/lock properties) data matches expected fixture.
#[tokio::test]
async fn object_142_puerta_madera_data_matches_expected_fixture() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(OBJ_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("OBJ142.json");
    verify_object_data_matches_fixture(142, fixture).await;
}

/// Verifies incremental update: parser detects modified Obj.dat and updates existing object.
#[tokio::test]
async fn incremental_update_detects_modified_object() {
    test_incremental_modification(
        &OBJECT_CONFIG,
        "dat",
        "Obj.dat",
        1122,
        "Name=Manzana Roja",
        "Name=Manzana MODIFICADA",
        |pool| {
            Box::pin(async move {
                let (updated_name,): (String,) =
                    sqlx::query_as("SELECT name FROM objects WHERE id = 1")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch OBJ1");

                assert_eq!(
                    updated_name, "Manzana MODIFICADA",
                    "OBJ1 name should be updated after modification"
                );
            })
        },
    )
    .await;
}

/// Verifies incremental update: parser detects new object added to Obj.dat.
#[tokio::test]
async fn incremental_update_detects_new_object() {
    let fixture = crate_dir()
        .join(FIXTURES_DIR)
        .join(OBJ_FIXTURES)
        .join(DAKARA_CPP_FIXTURES)
        .join("test_obj_1126.txt");

    test_incremental_addition(
        &OBJECT_CONFIG,
        "dat",
        "Obj.dat",
        1122,
        1126,
        fixture,
        |pool| {
            Box::pin(async move {
                let (obj_name,): (String,) =
                    sqlx::query_as("SELECT name FROM objects WHERE id = 1126")
                        .fetch_one(pool)
                        .await
                        .expect("failed to fetch OBJ1126");

                assert_eq!(
                    obj_name, "Teleport a Dungeon Newbie",
                    "OBJ1126 should have correct name"
                );
            })
        },
    )
    .await;
}
