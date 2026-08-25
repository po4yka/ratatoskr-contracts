//! Rust public-API compatibility — tests A-1 to A-4.
//!
//! `docs/TESTING.md`, "Compatibility checks": the frozen-baseline checker classifies a removed or
//! changed public item as breaking and an added item as additive, passes when nothing differs,
//! and accepts a regenerated baseline as the bless path for an approved change.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::fs;
use std::path::{Path, PathBuf};

use ratatoskr_contractsc::api;

/// A unique scratch directory for one test; the caller reads and writes baseline documents there.
fn temp_dir(tag: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "contractsc-api-compat-{tag}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("scratch directory is created");
    directory
}

/// Writes `document` to `directory/baseline.txt` and returns the path.
fn write_baseline(directory: &Path, document: &str) -> PathBuf {
    let path = directory.join("baseline.txt");
    fs::write(&path, document).expect("baseline file is written");
    path
}

/// The two items every scenario starts from.
const STRUCT_ITEM: &str = "pub struct ratatoskr_identifiers::BlobRef";
const FUNCTION_ITEM: &str = "pub fn ratatoskr_identifiers::BlobRef::as_str(&self) -> &str";

/// A-1. Identical snapshots classify clean.
#[test]
fn check_passes_when_baseline_matches() {
    let items = vec![FUNCTION_ITEM.to_owned(), STRUCT_ITEM.to_owned()];
    let document = api::render_baseline("ratatoskr-identifiers", "cargo-public-api 0.52.0", &items);
    let diff = api::classify(&document, &document);
    assert!(
        diff.is_clean(),
        "identical documents must classify clean: {diff:?}"
    );
}

/// A-2. An item present in the baseline and absent now is breaking, by name.
#[test]
fn removed_item_is_breaking() {
    let baseline = api::render_baseline(
        "ratatoskr-identifiers",
        "cargo-public-api 0.52.0",
        &[FUNCTION_ITEM.to_owned(), STRUCT_ITEM.to_owned()],
    );
    let current = api::render_baseline(
        "ratatoskr-identifiers",
        "cargo-public-api 0.52.0",
        &[STRUCT_ITEM.to_owned()],
    );
    let diff = api::classify(&baseline, &current);
    assert!(!diff.is_clean(), "a removal must fail the check");
    assert_eq!(
        diff.breaking,
        vec![FUNCTION_ITEM.to_owned()],
        "the removed item must be named"
    );
    assert!(diff.additive.is_empty(), "a removal adds nothing");
}

/// A-3. An item absent from the baseline and present now is additive, by name.
#[test]
fn added_item_is_additive() {
    const ENUM_ITEM: &str = "pub enum ratatoskr_identifiers::EntityKind";
    let baseline = api::render_baseline(
        "ratatoskr-identifiers",
        "cargo-public-api 0.52.0",
        &[STRUCT_ITEM.to_owned()],
    );
    let current = api::render_baseline(
        "ratatoskr-identifiers",
        "cargo-public-api 0.52.0",
        &[ENUM_ITEM.to_owned(), STRUCT_ITEM.to_owned()],
    );
    let diff = api::classify(&baseline, &current);
    assert!(!diff.is_clean(), "an addition must fail the check too");
    assert_eq!(diff.additive, vec![ENUM_ITEM.to_owned()]);
    assert!(diff.breaking.is_empty(), "an addition removes nothing");
}

/// A-4. Regenerating the baseline blesses an approved change without code changes elsewhere.
#[test]
fn write_then_check_round_trip() {
    let directory = temp_dir("round-trip");
    let original = api::render_baseline(
        "ratatoskr-identifiers",
        "cargo-public-api 0.52.0",
        &[STRUCT_ITEM.to_owned()],
    );
    let baseline_path = write_baseline(&directory, &original);

    // The sources grew a new export; the committed baseline does not know yet.
    let grown = format!("{STRUCT_ITEM}\npub fn ratatoskr_identifiers::BlobRef::new() -> Self\n");
    let committed = fs::read_to_string(&baseline_path).expect("baseline reads back");
    let diff = api::classify(&committed, &grown);
    assert!(!diff.is_clean(), "the unblessed addition must fail");

    // The bless path: regenerate, review, commit. The same tree then checks clean.
    let blessed = api::render_baseline("ratatoskr-identifiers", "cargo-public-api 0.52.0", &{
        api::snapshot_items(&grown)
    });
    fs::write(&baseline_path, &blessed).expect("blessed baseline is written");
    let reread = fs::read_to_string(&baseline_path).expect("blessed baseline reads back");
    let after = api::classify(&reread, &grown);
    assert!(
        after.is_clean(),
        "the blessed baseline must pass: {after:?}"
    );

    fs::remove_dir_all(&directory).ok();
}
