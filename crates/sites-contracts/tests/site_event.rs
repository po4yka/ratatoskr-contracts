//! The site event `sites/site-event.v1` (changeset SITE-027, workspace change
//! `add-site-owner-editing-and-live`, spec `site-live-events`): each fixture against the published
//! schema.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::path::{Path, PathBuf};

const SCHEMA: &str = "schemas/json-schema/sites/site-event.v1.schema.json";
const FIXTURES: &str = "fixtures/sites/site-event";

/// Every event type of the stream (spec `site-live-events`).
const EVENT_TYPES: [&str; 5] = [
    "release.published",
    "draft.changed",
    "edit.completed",
    "edit.conflicted",
    "resync",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/sites-contracts sits two levels below the repository root")
        .to_path_buf()
}

/// The published schema. Fails, naming the file, while the schema does not exist.
fn schema() -> serde_json::Value {
    let path = repo_root().join(SCHEMA);
    assert!(path.is_file(), "{SCHEMA} does not exist");
    let text = std::fs::read_to_string(&path).expect("the schema is readable");
    serde_json::from_str(&text).expect("the schema is JSON")
}

/// Every fixture under `bucket` of the site event folder, sorted by name, with its parsed value.
fn fixtures(bucket: &str) -> Vec<(String, serde_json::Value)> {
    let folder = repo_root().join(FIXTURES).join(bucket);
    let mut found: Vec<(String, serde_json::Value)> = std::fs::read_dir(&folder)
        .unwrap_or_else(|error| panic!("{}: {error}", folder.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| {
            let text = std::fs::read_to_string(&path).expect("a fixture is readable");
            let value = serde_json::from_str(&text)
                .unwrap_or_else(|error| panic!("{} is not JSON: {error}", path.display()));
            (path.display().to_string(), value)
        })
        .collect();
    found.sort_by(|left, right| left.0.cmp(&right.0));
    found
}

/// `Ok` when `instance` satisfies `schema`, with format checking switched on.
fn validate(schema: &serde_json::Value, instance: &serde_json::Value) -> Result<(), String> {
    jsonschema::options()
        .should_validate_formats(true)
        .build(schema)
        .expect("the schema compiles as draft 2020-12")
        .validate(instance)
        .map_err(|error| error.to_string())
}

fn invalid(name: &str) -> serde_json::Value {
    let path = repo_root().join(FIXTURES).join("invalid").join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{name} is not JSON: {error}"))
}

/// Spec `site-live-events`, "A page list is refused by the contract".
#[test]
fn release_published_refuses_page_list() {
    let schema = schema();
    assert!(
        validate(&schema, &invalid("release-published-with-slugs.json")).is_err(),
        "a release.published event with a list of slugs passes the schema"
    );
}

/// Spec `site-live-events`, "A public event carries no page list and no private content": no
/// source reference.
#[test]
fn release_published_refuses_source_ref() {
    let schema = schema();
    assert!(
        validate(&schema, &invalid("release-published-with-source-ref.json")).is_err(),
        "a release.published event with a source_ref passes the schema"
    );
}

/// Each event type of the stream has exactly one valid fixture, and it passes the schema.
#[test]
fn every_event_type_has_a_valid_fixture() {
    let schema = schema();
    let valid = fixtures("valid");
    for event_type in EVENT_TYPES {
        let matching: Vec<&(String, serde_json::Value)> = valid
            .iter()
            .filter(|(_, value)| {
                value.get("type").and_then(serde_json::Value::as_str) == Some(event_type)
            })
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "{event_type} must have exactly one valid fixture"
        );
        for (path, value) in matching {
            assert_eq!(validate(&schema, value), Ok(()), "{path} fails the schema");
        }
    }
}
