//! Fixtures of the sites contracts (changeset SITE-025, workspace change `add-site-publications`,
//! spec `site-manifest`).
//!
//! The tests find each contract by its generated schema path and its Rust path in the registry, so
//! this file compiles before the types exist and fails on an assertion instead.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{GENERATOR_VERSION, Metadata, generate, registry};

const MANIFEST_SCHEMA: &str = "schemas/json-schema/sites/site-manifest.v1.schema.json";
const MANIFEST_TYPE: &str = "ratatoskr_sites_contracts::SiteManifest";
const MANIFEST_FIXTURES: &str = "fixtures/sites/site-manifest";

/// Each invalid manifest fixture, the key its name states, and the value the refusal must name.
///
/// The key is the JSON Pointer of the refused member. An unknown key is refused at the root, so its
/// refusal must name the key itself.
const INVALID_MANIFESTS: [(&str, &str, Option<&str>); 4] = [
    ("invalid-site-id.toml", "/site_id", None),
    ("invalid-two-hostnames.toml", "/hostname", None),
    (
        "invalid-unknown-capability.toml",
        "/capabilities",
        Some("notes.delete"),
    ),
    ("invalid-unknown-key.toml", "", Some("theme")),
];

const VALID_MANIFESTS: [&str; 2] = ["valid-all-capabilities.toml", "valid-minimal.toml"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

/// The schema that `cargo contracts generate` writes at `output`, or `None` when no row and no Rust
/// type produce it.
fn generated_schema(output: &str) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(repo_root().join(Metadata::FILE_NAME))
        .expect("contracts.toml is committed");
    let metadata = Metadata::parse(&text).expect("contracts.toml parses");
    let generated =
        generate(&metadata, GENERATOR_VERSION).expect("the committed contracts generate");
    generated
        .get(Path::new(output))
        .map(|body| serde_json::from_str(body).expect("a generated schema is JSON"))
}

/// Deserializes `value` into the Rust type at `rust_path` and serializes it again, or `None` when
/// the registry has no such type.
fn roundtrip(
    rust_path: &str,
    value: &serde_json::Value,
) -> Option<Result<serde_json::Value, String>> {
    registry::root_types()
        .iter()
        .find(|root| root.rust_path == rust_path)
        .map(|root| (root.roundtrip)(value))
}

fn validator(schema: &serde_json::Value) -> jsonschema::Validator {
    jsonschema::options()
        .should_validate_formats(true)
        .build(schema)
        .expect("a generated schema compiles as draft 2020-12")
}

/// The TOML fixture `name` as the JSON value the schema validates.
fn manifest_fixture(name: &str) -> serde_json::Value {
    let path = repo_root().join(MANIFEST_FIXTURES).join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let parsed: toml::Value =
        toml::from_str(&text).unwrap_or_else(|error| panic!("{name} is not TOML: {error}"));
    serde_json::to_value(parsed).expect("a TOML document is a JSON value")
}

/// Every `*.toml` file directly under the manifest fixture folder, sorted by name.
fn manifest_fixture_names() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(repo_root().join(MANIFEST_FIXTURES))
        .expect("the manifest fixture folder exists")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .filter_map(|path| Some(path.file_name()?.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names
}

/// Workspace spec `site-manifest`, "A site manifest has one closed shape": each `valid-*` fixture
/// passes the published schema and the Rust type, and each `invalid-*` fixture fails both, and the
/// schema refuses it only on the key its name states.
#[test]
fn site_manifest_fixtures_match_their_names() {
    let schema = generated_schema(MANIFEST_SCHEMA);
    assert!(
        schema.is_some(),
        "sites/site-manifest.v1 does not exist: nothing generates {MANIFEST_SCHEMA}"
    );
    let validator = validator(&schema.unwrap_or_default());

    let mut expected: Vec<String> = VALID_MANIFESTS
        .iter()
        .copied()
        .chain(INVALID_MANIFESTS.iter().map(|(name, ..)| *name))
        .map(str::to_owned)
        .collect();
    expected.sort();
    assert_eq!(
        manifest_fixture_names(),
        expected,
        "each manifest fixture has a stated outcome"
    );

    for name in VALID_MANIFESTS {
        let manifest = manifest_fixture(name);
        let refusals: Vec<String> = validator
            .iter_errors(&manifest)
            .map(|error| format!("{} at {:?}", error, error.instance_path().as_str()))
            .collect();
        assert!(refusals.is_empty(), "{name} fails the schema: {refusals:?}");
        let parsed = roundtrip(MANIFEST_TYPE, &manifest)
            .unwrap_or_else(|| panic!("{MANIFEST_TYPE} is not in the registry"));
        assert!(parsed.is_ok(), "{name} fails the Rust type: {parsed:?}");
    }

    for (name, key, named_value) in INVALID_MANIFESTS {
        let manifest = manifest_fixture(name);
        let refusals: Vec<(String, String)> = validator
            .iter_errors(&manifest)
            .map(|error| (error.instance_path().as_str().to_owned(), error.to_string()))
            .collect();
        assert!(!refusals.is_empty(), "{name} passes the schema");
        for (pointer, message) in &refusals {
            assert!(
                pointer == key || pointer.starts_with(&format!("{key}/")),
                "{name} must fail on {key:?} only, but the schema refuses {pointer:?}: {message}"
            );
            if let Some(value) = named_value {
                assert!(
                    message.contains(value),
                    "{name}: the refusal does not name {value:?}: {message}"
                );
            }
        }
        let parsed = roundtrip(MANIFEST_TYPE, &manifest)
            .unwrap_or_else(|| panic!("{MANIFEST_TYPE} is not in the registry"));
        let error = parsed.expect_err(&format!("{name} passes the Rust type"));
        if let Some(value) = named_value {
            assert!(
                error.contains(value),
                "{name}: the Rust type refuses it without naming {value:?}: {error}"
            );
        }
    }
}

/// The release and page contracts: (contract name, generated schema, Rust type, fixture folder).
const DOCUMENTS: [(&str, &str, &str, &str); 2] = [
    (
        "sites/release.v1",
        "schemas/json-schema/sites/release.v1.schema.json",
        "ratatoskr_sites_contracts::Release",
        "fixtures/sites/release/valid",
    ),
    (
        "sites/page.v1",
        "schemas/json-schema/sites/page.v1.schema.json",
        "ratatoskr_sites_contracts::Page",
        "fixtures/sites/page/valid",
    ),
];

/// Every `*.json` file directly under `folder`, sorted by name, with its parsed value.
fn json_fixtures(folder: &str) -> Vec<(String, serde_json::Value)> {
    let mut found: Vec<(String, serde_json::Value)> = std::fs::read_dir(repo_root().join(folder))
        .unwrap_or_else(|error| panic!("{folder}: {error}"))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let parsed = serde_json::from_str(&text)
                .unwrap_or_else(|error| panic!("{} is not JSON: {error}", path.display()));
            (path.display().to_string(), parsed)
        })
        .collect();
    found.sort_by(|left, right| left.0.cmp(&right.0));
    found
}

/// Workspace spec `site-content-delivery`: a site reads `sites/release.v1` and `sites/page.v1`.
/// Each fixture deserialises into its Rust type and serialises back to the same canonical bytes,
/// so the type loses, adds and reorders nothing that a fixture carries.
#[test]
fn release_and_page_fixtures_round_trip() {
    let missing: Vec<&str> = DOCUMENTS
        .iter()
        .filter(|(_, schema, ..)| generated_schema(schema).is_none())
        .map(|(name, ..)| *name)
        .collect();
    assert!(
        missing.is_empty(),
        "{missing:?} do not exist: nothing generates their schemas"
    );

    for (name, _, rust_path, folder) in DOCUMENTS {
        let fixtures = json_fixtures(folder);
        assert!(!fixtures.is_empty(), "{name} has no fixture under {folder}");
        for (path, parsed) in fixtures {
            let reemitted = roundtrip(rust_path, &parsed)
                .unwrap_or_else(|| panic!("{rust_path} is not in the registry"))
                .unwrap_or_else(|error| panic!("{path} is refused by {rust_path}: {error}"));
            let before = ratatoskr_identifiers::canonical_json(&parsed).expect("JSON renders");
            let after = ratatoskr_identifiers::canonical_json(&reemitted).expect("JSON renders");
            assert_eq!(
                after, before,
                "{path} does not round-trip to the same bytes"
            );
        }
    }
}

/// The minimal release names each page fixture by the SHA-256 of that fixture's bytes, which are
/// the bytes a stub delivery route serves for its slug, and it names every page fixture. An edit to
/// a page fixture without a matching edit to the release fixture fails here.
#[test]
fn release_fixture_names_each_page_fixture_by_its_sha256() {
    use sha2::Digest as _;

    let release_path = repo_root().join("fixtures/sites/release/valid/minimal.json");
    let release: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&release_path).expect("the minimal release fixture exists"),
    )
    .expect("the minimal release fixture is JSON");
    let mut listed: Vec<(String, String)> = release
        .get("pages")
        .unwrap_or_default()
        .as_array()
        .expect("the release lists its pages")
        .iter()
        .map(|page| {
            (
                page.get("slug")
                    .unwrap_or_default()
                    .as_str()
                    .expect("a page has a slug")
                    .to_owned(),
                page.get("page_sha256")
                    .unwrap_or_default()
                    .as_str()
                    .expect("a page has a page_sha256")
                    .to_owned(),
            )
        })
        .collect();
    listed.sort();

    let folder = repo_root().join("fixtures/sites/page/valid");
    let mut served: Vec<(String, String)> = std::fs::read_dir(&folder)
        .expect("the page fixture folder exists")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| {
            let bytes = std::fs::read(&path).expect("a page fixture is readable");
            let page: serde_json::Value =
                serde_json::from_slice(&bytes).expect("a page fixture is JSON");
            (
                page.get("slug")
                    .unwrap_or_default()
                    .as_str()
                    .expect("a page has a slug")
                    .to_owned(),
                hex::encode(sha2::Sha256::digest(&bytes)),
            )
        })
        .collect();
    served.sort();

    assert_eq!(
        listed, served,
        "the release fixture must name each page fixture by the SHA-256 of its bytes"
    );
}

/// Workspace spec `site-content-delivery`: the sites service serves the release and its pages,
/// the site SDK reads them for every site, and the web client shows the release history. The rows
/// name exactly these three readers.
#[test]
fn release_and_page_name_their_readers() {
    let text = std::fs::read_to_string(repo_root().join(Metadata::FILE_NAME))
        .expect("contracts.toml is committed");
    let metadata = Metadata::parse(&text).expect("contracts.toml parses");
    for id in ["sites.release", "sites.page"] {
        let contract = metadata
            .contracts
            .iter()
            .find(|contract| contract.id == id)
            .unwrap_or_else(|| panic!("{id} is not registered"));
        let mut consumers = contract.consumers.clone();
        consumers.sort();
        assert_eq!(
            consumers,
            ["ratatoskr-site-sdk", "ratatoskr-sites", "ratatoskr-web"],
            "{id}: the readers are the sites service, the site SDK and the web client"
        );
    }
}
