//! The bundle manifest `sites/bundle-manifest.v1` (changeset SITE-026, workspace change
//! `add-site-gateway-hosting`, spec `site-bundles`): each fixture against the published schema.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::path::{Path, PathBuf};

const SCHEMA: &str = "schemas/json-schema/sites/bundle-manifest.v1.schema.json";
const FIXTURES: &str = "fixtures/sites/bundle-manifest";

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

/// The fixture `name` under the bundle manifest fixture folder.
fn fixture(name: &str) -> serde_json::Value {
    let path = repo_root().join(FIXTURES).join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{name} is not JSON: {error}"))
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

/// Spec `site-bundles`, "A bundle carries a manifest that lists every file": every file names its
/// SHA-256.
#[test]
fn a_manifest_without_a_file_digest_is_refused() {
    let schema = schema();
    let fixture = fixture("invalid/file-without-sha256.json");
    assert!(
        validate(&schema, &fixture).is_err(),
        "a file without sha256 passes the schema"
    );
}

/// Spec `site-bundles`, "Bundle paths are confined and static": no `..` segment.
#[test]
fn a_path_with_a_parent_segment_is_refused() {
    let schema = schema();
    let fixture = fixture("invalid/parent-segment.json");
    assert!(
        validate(&schema, &fixture).is_err(),
        "a path with a .. segment passes the schema"
    );
}

/// Spec `site-bundles`, "A bundle shadows the forwarding prefix": no path under `_r/`.
#[test]
fn a_path_under_the_forwarding_prefix_is_refused() {
    let schema = schema();
    let fixture = fixture("invalid/forwarding-prefix.json");
    assert!(
        validate(&schema, &fixture).is_err(),
        "a path under _r/ passes the schema"
    );
}
