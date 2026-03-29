//! Tests for manifest hash storage and client manifest building.
//!
//! Validates the upsert logic, first-run behavior, and that the
//! client-facing JSON structure matches the documented contract.

use ao_sql_to_static_files::manifest::{
    build_client_manifest, get_stored_hash, sha256_hex, upsert_hash,
};
use ao_sql_to_static_files::serialization::serialize_data_for_hash;

use crate::common::insert_test_character;
use ao_shared::testing::setup_test_db;

/// Fetches both rankings, hashes them, upserts only the entries that changed,
/// and returns `(level_hash, pvp_hash)` for the caller to compare across runs.
async fn run_export(pool: &sqlx::PgPool) -> (String, String) {
    use ao_sql_to_static_files::queries::{
        fetch_top_level, fetch_top_pvp_kills,
    };

    let level_data = fetch_top_level(pool, 50).await.expect("query failed");
    let pvp_data = fetch_top_pvp_kills(pool, 50).await.expect("query failed");

    let level_hash =
        sha256_hex(&serialize_data_for_hash(&level_data).expect("ser failed"));
    let pvp_hash =
        sha256_hex(&serialize_data_for_hash(&pvp_data).expect("ser failed"));

    let stored_level = get_stored_hash(pool, "characters/top-by-level")
        .await
        .expect("query failed");
    let stored_pvp = get_stored_hash(pool, "characters/top-by-kills")
        .await
        .expect("query failed");

    if stored_level.as_deref() != Some(&level_hash) {
        upsert_hash(pool, "characters/top-by-level", &level_hash)
            .await
            .expect("upsert failed");
    }
    if stored_pvp.as_deref() != Some(&pvp_hash) {
        upsert_hash(pool, "characters/top-by-kills", &pvp_hash)
            .await
            .expect("upsert failed");
    }

    (level_hash, pvp_hash)
}

// Real 64-char SHA-256 hex strings used as test fixtures.
// The hash column is CHAR(64), which right-pads shorter values with spaces,
// so all test hashes must be exactly 64 hex characters to avoid spurious mismatches.
const HASH_A: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_LEVEL: &str =
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const HASH_KILLS: &str =
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

// Verifies first-run behavior: before any export has run, get_stored_hash
// must return None rather than an error or a default value.
// This is the signal the exporter uses to decide whether to skip a file.
#[tokio::test]
async fn get_stored_hash_returns_none_for_unknown_key() {
    let (_container, pool) = setup_test_db().await;

    let result = get_stored_hash(&pool, "nonexistent/key")
        .await
        .expect("query failed");

    assert!(result.is_none(), "unknown key should return None");
}

// Verifies the basic insert + read round-trip: a hash written via upsert_hash
// must be retrievable with get_stored_hash using the same file key.
// This is the core invariant the deduplication logic depends on.
#[tokio::test]
async fn upsert_hash_insert_then_get_returns_same_hash() {
    let (_container, pool) = setup_test_db().await;

    upsert_hash(&pool, "characters/top-by-level", HASH_A)
        .await
        .expect("upsert failed");

    let stored = get_stored_hash(&pool, "characters/top-by-level")
        .await
        .expect("query failed");

    assert_eq!(
        stored,
        Some(HASH_A.to_owned()),
        "stored hash should match inserted value"
    );
}

// Verifies the upsert semantics: a second call with a different hash for the
// same key must overwrite the first, not insert a duplicate row.
// The exporter calls upsert after every successful upload.
#[tokio::test]
async fn upsert_hash_updates_existing_entry() {
    let (_container, pool) = setup_test_db().await;

    upsert_hash(&pool, "key", HASH_A)
        .await
        .expect("first upsert failed");
    upsert_hash(&pool, "key", HASH_B)
        .await
        .expect("second upsert failed");

    let stored = get_stored_hash(&pool, "key").await.expect("query failed");

    assert_eq!(
        stored,
        Some(HASH_B.to_owned()),
        "second upsert should overwrite the first"
    );
}

// Verifies the empty-state output: a freshly deployed server with no exports
// must still produce valid JSON with a generated_at timestamp and an empty
// files map rather than an error or a null value.
#[tokio::test]
async fn build_client_manifest_empty_table_produces_valid_json() {
    let (_container, pool) = setup_test_db().await;

    let json = build_client_manifest(&pool)
        .await
        .expect("manifest build failed");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("invalid JSON");

    assert!(
        parsed
            .get("generated_at")
            .expect("missing generated_at")
            .is_string(),
        "manifest should have a generated_at string"
    );
    let files = parsed
        .get("files")
        .expect("missing files")
        .as_object()
        .expect("files should be object");
    assert!(
        files.is_empty(),
        "empty table should produce empty files map"
    );
}

// Verifies the client-facing JSON contract: all manifest entries must appear
// under the files key, sorted alphabetically, each with a hash field.
// Frontend clients rely on this structure to detect file changes.
#[tokio::test]
async fn build_client_manifest_includes_all_entries_sorted_by_key() {
    let (_container, pool) = setup_test_db().await;

    upsert_hash(&pool, "characters/top-by-level", HASH_LEVEL)
        .await
        .expect("upsert failed");
    upsert_hash(&pool, "characters/top-by-kills", HASH_KILLS)
        .await
        .expect("upsert failed");

    let json = build_client_manifest(&pool)
        .await
        .expect("manifest build failed");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("invalid JSON");

    let files = parsed
        .get("files")
        .expect("missing files")
        .as_object()
        .expect("files should be object");
    assert_eq!(files.len(), 2, "manifest should contain both entries");

    // BTreeMap guarantees alphabetical order
    let keys: Vec<&String> = files.keys().collect();
    assert_eq!(
        keys.first().expect("missing first key"),
        &"characters/top-by-kills",
        "entries should be sorted alphabetically"
    );
    assert_eq!(
        keys.get(1).expect("missing second key"),
        &"characters/top-by-level",
        "entries should be sorted alphabetically"
    );

    assert_eq!(
        files
            .get("characters/top-by-level")
            .expect("missing level entry")
            .get("hash")
            .expect("missing hash"),
        HASH_LEVEL,
        "level hash should match"
    );
    assert_eq!(
        files
            .get("characters/top-by-kills")
            .expect("missing kills entry")
            .get("hash")
            .expect("missing hash"),
        HASH_KILLS,
        "kills hash should match"
    );
}

// Simulates two consecutive export runs where only the PvP data changes.
// Verifies that updated_at advances for the changed entry and is preserved
// for the unchanged one, mirroring the upload_ranking_if_changed logic.
#[tokio::test]
async fn manifest_only_updates_changed_entries() {
    let (_container, pool) = setup_test_db().await;

    // --- Run 1: initial export ---
    insert_test_character(&pool, "Hero", 45, 999_999, 50, 1, 1, 50_000, false)
        .await;
    insert_test_character(
        &pool, "Warrior", 40, 500_000, 200, 3, 2, 30_000, false,
    )
    .await;

    let (level_hash, pvp_hash) = run_export(&pool).await;

    // Capture the initial updated_at timestamps from the manifest
    let manifest_run1 = build_client_manifest(&pool)
        .await
        .expect("manifest build failed");
    let parsed_run1: serde_json::Value =
        serde_json::from_str(&manifest_run1).expect("invalid JSON");
    let files_run1 = parsed_run1.get("files").expect("missing files");
    let original_level_updated_at = files_run1
        .get("characters/top-by-level")
        .expect("missing level entry")
        .get("updated_at")
        .expect("missing updated_at")
        .as_str()
        .expect("updated_at not a string")
        .to_owned();
    let original_pvp_updated_at = files_run1
        .get("characters/top-by-kills")
        .expect("missing kills entry")
        .get("updated_at")
        .expect("missing updated_at")
        .as_str()
        .expect("updated_at not a string")
        .to_owned();

    // Small delay so timestamps differ if upserted again
    tokio::time::sleep(core::time::Duration::from_millis(50)).await;

    // --- Run 2: only PvP data changes ---
    // Modify Warrior's kills; run_export will only upsert the entries that changed
    insert_test_character(
        &pool, "Warrior", 40, 500_000, 999, 3, 2, 30_000, false,
    )
    .await;

    let (level_hash_2, pvp_hash_2) = run_export(&pool).await;

    assert_ne!(
        pvp_hash, pvp_hash_2,
        "PvP hash should differ after kill change"
    );

    // Build final manifest and verify timestamps
    let manifest_run2 = build_client_manifest(&pool)
        .await
        .expect("manifest build failed");
    let parsed_run2: serde_json::Value =
        serde_json::from_str(&manifest_run2).expect("invalid JSON");
    let files_run2 = parsed_run2.get("files").expect("missing files");

    let final_pvp_updated_at = files_run2
        .get("characters/top-by-kills")
        .expect("missing kills entry")
        .get("updated_at")
        .expect("missing updated_at")
        .as_str()
        .expect("updated_at not a string");

    assert_ne!(
        original_pvp_updated_at, final_pvp_updated_at,
        "PvP updated_at should change when data changes"
    );

    // For entries whose hash did NOT change, updated_at must be preserved
    let final_level_updated_at = files_run2
        .get("characters/top-by-level")
        .expect("missing level entry")
        .get("updated_at")
        .expect("missing updated_at")
        .as_str()
        .expect("updated_at not a string");

    if level_hash == level_hash_2 {
        assert_eq!(
            original_level_updated_at, final_level_updated_at,
            "Level updated_at should be preserved when data is unchanged"
        );
    }
}
