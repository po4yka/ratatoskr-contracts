//! Compatibility fixtures — test C-3. `ARCHITECTURE.md` S12 requires both directions per family.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use std::path::{Path, PathBuf};

use ratatoskr_error_contracts::ErrorEnvelope;
use ratatoskr_identifiers::dropped_field_pointers;

/// The two compatibility buckets of this family, as `contracts.toml` declares the fixtures dir.
fn bucket(direction: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/core/error-envelope/compat")
        .join(direction)
}

/// Every `.json` file in a bucket, in sorted order so a failure names the same file every run.
fn fixtures(direction: &str) -> Vec<PathBuf> {
    let dir = bucket(direction);
    let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", dir.display()))
        .map(|entry| entry.expect("a readable directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    found.sort();
    assert!(
        !found.is_empty(),
        "{} carries no compatibility fixture, so the direction is untested",
        dir.display()
    );
    found
}

/// Parses a fixture and asserts that re-emitting it drops nothing.
fn accepts_losslessly(path: &Path) {
    let bytes = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    let decoded: ErrorEnvelope = serde_json::from_str(&bytes)
        .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));

    let input: serde_json::Value = serde_json::from_str(&bytes).expect("the fixture is JSON");
    let reemitted: serde_json::Value =
        serde_json::to_value(&decoded).expect("the decoded value re-serializes");
    assert_eq!(
        dropped_field_pointers(&input, &reemitted),
        Vec::<String>::new(),
        "{} lost a member on the way through this build",
        path.display()
    );
}

/// C-3, forward half: today's type accepts and re-emits what a newer producer sent.
#[test]
fn old_consumer_accepts_new_producer() {
    for path in fixtures("old-consumer-new-producer") {
        accepts_losslessly(&path);
    }
}

/// C-3, backward half: a payload frozen at the first shape still parses today.
#[test]
fn new_consumer_accepts_old_producer() {
    for path in fixtures("new-consumer-old-producer") {
        accepts_losslessly(&path);
    }
}
