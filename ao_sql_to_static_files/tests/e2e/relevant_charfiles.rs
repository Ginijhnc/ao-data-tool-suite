//! Tests for the relevant charfile export pipeline.
//!
//! Validates that charfile deduplication correctly skips unchanged profiles
//! and re-exports only characters whose data changed between runs.

use ao_sql_to_static_files::manifest::{
    get_stored_hash, sha256_hex, upsert_hash,
};
use ao_sql_to_static_files::queries::build_all_charfile_exports;

use crate::common::insert_test_character;
use ao_shared::testing::setup_test_db;

// Simulates two consecutive export runs. On the first run all charfiles are
// new so all hashes are stored. Between runs, one character gains kills and
// is re-imported with new data. On the second run only that character's hash
// differs, so only their profile would be re-uploaded to R2.
#[tokio::test]
async fn only_changed_charfile_gets_new_hash_after_data_update() {
    let (_container, pool) = setup_test_db().await;

    insert_test_character(&pool, "Hero", 45, 999_999, 50, 1, 1, 50_000, false)
        .await;
    insert_test_character(
        &pool, "Warrior", 40, 500_000, 200, 3, 2, 30_000, false,
    )
    .await;
    insert_test_character(&pool, "Mage", 38, 400_000, 10, 1, 1, 20_000, false)
        .await;

    // --- Run 1: first export, store all hashes ---
    let names_run1 =
        vec!["Hero".to_owned(), "Warrior".to_owned(), "Mage".to_owned()];
    let entries_run1 = build_all_charfile_exports(&pool, names_run1)
        .await
        .expect("run 1 failed");
    assert_eq!(
        entries_run1.len(),
        3,
        "should produce one entry per character"
    );

    for entry in &entries_run1 {
        let hash = sha256_hex(&entry.hash_content);
        upsert_hash(&pool, &entry.manifest_key, &hash)
            .await
            .expect("upsert failed");
    }

    // Capture run 1 hashes
    let hero_hash_run1 = get_stored_hash(&pool, "characters/profiles/hero")
        .await
        .expect("query failed")
        .expect("hash should exist after run 1");
    let warrior_hash_run1 =
        get_stored_hash(&pool, "characters/profiles/warrior")
            .await
            .expect("query failed")
            .expect("hash should exist after run 1");
    let mage_hash_run1 = get_stored_hash(&pool, "characters/profiles/mage")
        .await
        .expect("query failed")
        .expect("hash should exist after run 1");

    // Between runs: Warrior gains kills (re-imported by ao_data_to_sql)
    insert_test_character(
        &pool, "Warrior", 40, 500_000, 999, 3, 2, 30_000, false,
    )
    .await;

    // --- Run 2: re-export the same names ---
    let names_run2 =
        vec!["Hero".to_owned(), "Warrior".to_owned(), "Mage".to_owned()];
    let entries_run2 = build_all_charfile_exports(&pool, names_run2)
        .await
        .expect("run 2 failed");

    // Compute which entries would actually be uploaded (hash differs from stored)
    let mut uploads: Vec<&str> = Vec::new();
    for entry in &entries_run2 {
        let new_hash = sha256_hex(&entry.hash_content);
        let stored = get_stored_hash(&pool, &entry.manifest_key)
            .await
            .expect("query failed");
        if stored.as_deref() != Some(new_hash.as_str()) {
            uploads.push(&entry.manifest_key);
        }
    }

    assert_eq!(
        uploads,
        vec!["characters/profiles/warrior"],
        "only Warrior changed — only their profile should be re-uploaded"
    );

    // Verify hashes: Hero and Mage are identical across runs
    let hero_entry = entries_run2
        .iter()
        .find(|e| e.manifest_key == "characters/profiles/hero")
        .expect("hero entry missing");
    let mage_entry = entries_run2
        .iter()
        .find(|e| e.manifest_key == "characters/profiles/mage")
        .expect("mage entry missing");
    let warrior_entry = entries_run2
        .iter()
        .find(|e| e.manifest_key == "characters/profiles/warrior")
        .expect("warrior entry missing");

    assert_eq!(
        sha256_hex(&hero_entry.hash_content),
        hero_hash_run1,
        "Hero data unchanged — hash must be identical"
    );
    assert_eq!(
        sha256_hex(&mage_entry.hash_content),
        mage_hash_run1,
        "Mage data unchanged — hash must be identical"
    );
    assert_ne!(
        sha256_hex(&warrior_entry.hash_content),
        warrior_hash_run1,
        "Warrior gained kills — hash must differ"
    );
}
