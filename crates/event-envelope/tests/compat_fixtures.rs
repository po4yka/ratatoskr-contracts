//! Backward / forward compatibility by family — tests C-1 and C-2 for the event family.
//!
//! `fixtures/core/event-envelope/compat/old-consumer-new-producer/` holds records a *newer*
//! producer emits; this build is the old consumer and must accept, preserve and re-emit them.
//! `.../new-consumer-old-producer/` holds records an *older* producer emitted; this build is the
//! new consumer and must still parse them.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use std::path::{Path, PathBuf};

use ratatoskr_event_envelope::EventEnvelope;
use ratatoskr_identifiers::dropped_field_pointers;

/// The repository root, two levels above this crate.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the crate directory has a repository root above it")
}

/// Every `*.json` file directly inside one compatibility direction, sorted by path.
///
/// A missing or empty directory is a failure, not a skip. Rule R13 of `contractsc` only walks
/// files that already exist and only requires an `invalid/` file to be registered; nothing there
/// requires a `compat/` fixture to exist at all, so deleting this directory would otherwise leave
/// C-1 and C-2 iterating zero times and reporting ok. The sibling families assert the same thing.
fn compat_fixtures(direction: &str) -> Vec<(PathBuf, String)> {
    let directory = repo_root()
        .join("fixtures/core/event-envelope/compat")
        .join(direction);
    let mut found: Vec<(PathBuf, String)> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| {
            let body = std::fs::read_to_string(&path).expect("a fixture must be readable UTF-8");
            (path, body)
        })
        .collect();
    found.sort();
    assert!(
        !found.is_empty(),
        "{} carries no compatibility fixture, so the direction is untested",
        directory.display()
    );
    found
}

/// C-1. This build reads what a newer producer writes, and gives back every byte it was sent.
#[test]
fn old_consumer_accepts_new_producer_fixtures() {
    for (path, body) in compat_fixtures("old-consumer-new-producer") {
        let name = path.display();
        let envelope = EventEnvelope::from_json(body.as_bytes())
            .unwrap_or_else(|error| panic!("{name} must parse: {error}"));

        let reemitted = envelope
            .to_canonical_json()
            .unwrap_or_else(|error| panic!("{name} must re-emit: {error}"));

        let input: serde_json::Value = serde_json::from_str(&body).unwrap();
        let roundtripped: serde_json::Value = serde_json::from_str(&reemitted).unwrap();
        assert_eq!(
            dropped_field_pointers(&input, &roundtripped),
            Vec::<String>::new(),
            "{name}: an old consumer must not discard a new producer's members"
        );
    }
}

/// C-2. This build still reads what an older producer wrote.
#[test]
fn new_consumer_accepts_old_producer_fixtures() {
    for (path, body) in compat_fixtures("new-consumer-old-producer") {
        let name = path.display();
        EventEnvelope::from_json(body.as_bytes())
            .unwrap_or_else(|error| panic!("{name} must still parse today: {error}"));
    }
}
