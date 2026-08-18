//! The provenance header and the provenance-free digest — tests P-1 to P-5.
//!
//! `ARCHITECTURE.md` S11: "Generated outputs include a provenance header containing generator
//! version, source digest, and contract version."

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{
    GENERATOR_VERSION, Metadata, generate, normalize, provenance, registry, render,
};

/// Every member the provenance block must carry, and nothing else.
const REQUIRED_MEMBERS: &[&str] = &[
    "canonical_source",
    "contract_id",
    "contract_major_version",
    "generated",
    "generator",
    "generator_version",
    "schemars_version",
    "source_digest",
    "validation_note",
];

/// The repository root, derived from the compiled-in manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

/// Every generated artifact, in memory.
fn generated() -> BTreeMap<PathBuf, String> {
    let path = repo_root().join(Metadata::FILE_NAME);
    let text = std::fs::read_to_string(&path).expect("contracts.toml is committed");
    let metadata = Metadata::parse(&text).expect("contracts.toml parses");
    generate(&metadata, GENERATOR_VERSION).expect("the committed contracts generate")
}

/// P-1. All nine members are present, the marker string is exact, and the digest is a lowercase
/// `sha256:` hex string.
#[test]
fn provenance_header_is_complete() {
    for (path, body) in generated() {
        let document: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
        let block = document
            .get(provenance::PROVENANCE_KEY)
            .and_then(serde_json::Value::as_object)
            .unwrap_or_else(|| panic!("{} carries no provenance block", path.display()));

        let members: Vec<&str> = block.keys().map(String::as_str).collect();
        assert_eq!(members, REQUIRED_MEMBERS, "{}", path.display());

        assert_eq!(
            block.get("generated").and_then(serde_json::Value::as_str),
            Some(provenance::GENERATED_MARKER),
            "{} carries the wrong generated-file marker",
            path.display()
        );
        assert_eq!(
            block.get("generator").and_then(serde_json::Value::as_str),
            Some(provenance::GENERATOR_NAME)
        );
        assert_eq!(
            block
                .get("generator_version")
                .and_then(serde_json::Value::as_str),
            Some(GENERATOR_VERSION)
        );
        assert_eq!(
            block
                .get("validation_note")
                .and_then(serde_json::Value::as_str),
            Some(provenance::VALIDATION_NOTE)
        );
        assert!(
            block
                .get("contract_major_version")
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|major| major >= 1),
            "{} declares no contract major version",
            path.display()
        );

        let digest = block
            .get("source_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{} carries no source_digest", path.display()));
        let hex = digest
            .strip_prefix("sha256:")
            .unwrap_or_else(|| panic!("{digest} is not a sha256 digest"));
        assert_eq!(hex.len(), 64, "{digest}");
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "{digest} is not lowercase hex"
        );
    }
}

/// P-2. Delete the provenance member, re-render with the canonical serializer, hash: the result is
/// the embedded digest. The digest definition is well-formed and third-party auditable.
#[test]
fn source_digest_matches_recomputed_body() {
    for (path, body) in generated() {
        let embedded = provenance::embedded_digest(&body)
            .unwrap_or_else(|| panic!("{} carries no source_digest", path.display()));
        let recomputed = provenance::recompute_digest(&body)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(embedded, recomputed, "{}", path.display());
    }
}

/// P-3. Nothing environment-dependent leaked into an artifact: no absolute path, no home
/// directory, no user name, and no `generated_at`-style clock reading.
#[test]
fn no_generated_file_contains_a_timestamp_or_absolute_path() {
    let root = repo_root();
    let root_text = root.display().to_string();
    let mut needles = vec![root_text];
    for variable in ["HOME", "USER", "LOGNAME", "CARGO_HOME", "PWD"] {
        if let Ok(value) = std::env::var(variable)
            && value.len() > 3
        {
            needles.push(value);
        }
    }

    for (path, body) in generated() {
        for needle in &needles {
            assert!(
                !body.contains(needle.as_str()),
                "{} leaks the environment value {needle:?}",
                path.display()
            );
        }
        let document: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
        let block = document
            .get(provenance::PROVENANCE_KEY)
            .and_then(serde_json::Value::as_object)
            .expect("a provenance block");
        for clock_reading in ["generated_at", "built_at", "timestamp", "date"] {
            assert!(
                !block.contains_key(clock_reading),
                "{} records {clock_reading}, which no generator can reproduce",
                path.display()
            );
        }
    }
}

/// P-4. The recorded generator identity is true: `SCHEMARS_VERSION` is the version the lockfile
/// pins, because schemars shapes the bytes.
#[test]
fn schemars_version_const_matches_lockfile() {
    let text =
        std::fs::read_to_string(repo_root().join("Cargo.lock")).expect("Cargo.lock is committed");
    let lockfile: toml::Value = toml::from_str(&text).expect("Cargo.lock is TOML");
    let packages = lockfile
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("Cargo.lock lists packages");
    let locked = packages
        .iter()
        .find(|package| package.get("name").and_then(toml::Value::as_str) == Some("schemars"))
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .expect("schemars is locked");
    assert_eq!(locked, provenance::SCHEMARS_VERSION);
}

/// P-5. The serialize and deserialize contracts currently produce identical bytes, so publishing
/// one schema is lossless. If this ever fails, both must be published.
#[test]
fn serialize_and_deserialize_contracts_are_identical() {
    for root in registry::root_types() {
        let mut deserialize = schemars::generate::SchemaSettings::draft2020_12()
            .for_deserialize()
            .into_generator();
        let mut serialize = schemars::generate::SchemaSettings::draft2020_12()
            .for_serialize()
            .into_generator();
        let mut reading = (root.schema)(&mut deserialize);
        let mut writing = (root.schema)(&mut serialize);
        normalize::sort_required(&mut reading);
        normalize::sort_required(&mut writing);
        assert_eq!(
            render::render(&reading),
            render::render(&writing),
            "{} reads and writes different shapes",
            root.rust_path
        );
    }
}
