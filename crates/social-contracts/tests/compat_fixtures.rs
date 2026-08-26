//! Compatibility fixtures for the three social families (test C-style, both directions).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use std::path::{Path, PathBuf};

use ratatoskr_identifiers::dropped_field_pointers;

/// Every `.json` file in one family's compatibility direction, sorted by path.
///
/// A missing or empty directory is a failure, not a skip: nothing else requires a `compat/`
/// fixture to exist, so deleting one would otherwise leave a direction untested while the
/// suite reports ok.
fn compat_fixtures(family: &str, direction: &str) -> Vec<(PathBuf, String)> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .join(family)
        .join("compat")
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

/// Parses `body` as `T` and asserts that re-emitting it drops nothing.
fn accepts_losslessly<T>(family: &str, name: &str, body: &str)
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let decoded: T = serde_json::from_str(body)
        .unwrap_or_else(|error| panic!("{family}/{name} must parse: {error}"));

    let input: serde_json::Value = serde_json::from_str(body).expect("the fixture is JSON");
    let reemitted: serde_json::Value =
        serde_json::to_value(&decoded).expect("the decoded value re-serializes");
    assert_eq!(
        dropped_field_pointers(&input, &reemitted),
        Vec::<String>::new(),
        "{family}/{name}: this build must not discard a member it was sent"
    );
}

macro_rules! direction_tests {
    ($mod_name:ident, $family:literal, $payload:ty) => {
        mod $mod_name {
            const FAMILY: &str = $family;

            /// Forward half: today's type accepts and re-emits what a newer producer sent.
            #[test]
            fn old_consumer_accepts_new_producer() {
                for (path, body) in super::compat_fixtures(FAMILY, "old-consumer-new-producer") {
                    let name = path.file_name().expect("a file name").to_string_lossy();
                    super::accepts_losslessly::<$payload>(FAMILY, &name, &body);
                }
            }

            /// Backward half: the minimal first-day shape still parses today.
            #[test]
            fn new_consumer_accepts_old_producer() {
                for (path, body) in super::compat_fixtures(FAMILY, "new-consumer-old-producer") {
                    let name = path.file_name().expect("a file name").to_string_lossy();
                    super::accepts_losslessly::<$payload>(FAMILY, &name, &body);
                }
            }
        }
    };
}

direction_tests!(
    snapshot_family,
    "social/social-source-snapshot",
    ratatoskr_social_contracts::SocialSourceSnapshot
);
direction_tests!(
    captured_family,
    "events/social.source.captured.v1",
    ratatoskr_social_contracts::SocialSourceCaptured
);
direction_tests!(
    updated_family,
    "events/social.source.updated.v1",
    ratatoskr_social_contracts::SocialSourceUpdated
);
direction_tests!(
    removed_family,
    "events/social.source.removed.v1",
    ratatoskr_social_contracts::SocialSourceRemoved
);
