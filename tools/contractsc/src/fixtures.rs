//! Fixture discovery and expectation checking.
//!
//! # Which layer `check` exercises
//!
//! `ratatoskr-contractsc` depends on `jsonschema` as a **dev-dependency only** (specification
//! §3), so the library half of the gate exercises the `serde` layer: the canonical Rust type is
//! the authority, and it is strictly stronger than the published schema. The `json_schema`
//! column of `fixtures/invalid-expectations.toml` is asserted by test `F-6` in
//! `tools/contractsc/tests/fixtures.rs`, where `jsonschema` is available.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::check::Finding;
use crate::metadata::Metadata;
use crate::registry;

/// The declared reason an `invalid/` fixture is invalid.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expectation {
    /// Every layer that must reject the fixture. Every layer **not** listed must accept it.
    pub rejected_by: Vec<Layer>,
    /// A substring the rejecting layer's error text must contain.
    pub error_contains: String,
    /// Why the fixture is invalid, for the reader of the diff.
    pub reason: String,
}

/// A layer that can reject a fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// The generated draft 2020-12 schema.
    JsonSchema,
    /// The canonical Rust type's `Deserialize`.
    Serde,
}

/// The repository-relative path of the invalid-expectations registry.
pub const EXPECTATIONS_PATH: &str = "fixtures/invalid-expectations.toml";

/// Loads `fixtures/invalid-expectations.toml`. An absent or unparseable file yields no entries;
/// rule R13 then reports every unregistered `invalid/` fixture.
#[must_use]
pub fn load_expectations(root: &Path) -> BTreeMap<String, Expectation> {
    std::fs::read_to_string(root.join(EXPECTATIONS_PATH))
        .ok()
        .and_then(|text| toml::from_str(&text).ok())
        .unwrap_or_default()
}

/// The registry key of a fixture: its path relative to `fixtures/`.
#[must_use]
pub fn expectation_key(fixtures_dir: &str, tail: &str) -> String {
    let family = fixtures_dir
        .strip_prefix("fixtures/")
        .unwrap_or(fixtures_dir);
    format!("{family}/{tail}")
}

/// Every `*.json` file under `directory`, recursively, sorted by path.
#[must_use]
pub fn walk_json(directory: &Path) -> Vec<PathBuf> {
    let mut found = walk_all(directory);
    found.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    found
}

/// Every file under `directory`, whatever its extension, recursively, sorted by path.
///
/// The secret/PII scanner walks this rather than [`walk_json`]: specification §7.4 scans *every
/// byte* under `fixtures/**`, and `fixtures/invalid-expectations.toml` — a real committed file in
/// that tree — quotes fixture values verbatim inside `error_contains`.
#[must_use]
pub fn walk_all(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(directory, &mut found);
    found.sort();
    found
}

/// Recursive half of [`walk_all`].
fn collect(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, found);
        } else {
            found.push(path);
        }
    }
}

/// Every entity kind used by any string in the fixture at `path`.
///
/// A string counts as a reference only when [`ratatoskr_identifiers::EntityRef`] parses it, so a
/// message or an instant that happens to contain a colon is not mistaken for one.
#[must_use]
pub fn entity_kinds_in(path: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(document) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    let mut kinds = Vec::new();
    collect_entity_kinds(&document, &mut kinds);
    kinds.sort();
    kinds.dedup();
    kinds
}

/// Recursive half of [`entity_kinds_in`].
fn collect_entity_kinds(value: &serde_json::Value, kinds: &mut Vec<String>) {
    match value {
        serde_json::Value::String(text) => {
            if let Ok(reference) = ratatoskr_identifiers::EntityRef::parse(text) {
                kinds.push(reference.kind().as_str().to_owned());
            }
        }
        serde_json::Value::Object(members) => {
            for member in members.values() {
                collect_entity_kinds(member, kinds);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_entity_kinds(item, kinds);
            }
        }
        _ => {}
    }
}

/// Validates every fixture against the canonical Rust type and its declared expectation.
#[must_use]
pub fn validate(metadata: &Metadata, root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    let expectations = load_expectations(root);
    let registry = registry::root_types();

    for contract in &metadata.contracts {
        let Some(declared) = contract.root_types.first() else {
            continue;
        };
        let Some(root_type) = registry
            .iter()
            .find(|candidate| candidate.rust_path == declared.rust_path)
        else {
            continue;
        };
        let fixtures_root = root.join(&contract.fixtures_dir);
        for absolute in walk_json(&fixtures_root) {
            let Ok(tail) = absolute.strip_prefix(&fixtures_root) else {
                continue;
            };
            let tail = tail.to_string_lossy().replace('\\', "/");
            let display = format!("{}/{tail}", contract.fixtures_dir);
            let Ok(text) = std::fs::read_to_string(&absolute) else {
                findings.push(Finding::Fixture {
                    path: display,
                    detail: "cannot be read".to_owned(),
                });
                continue;
            };
            let parsed = match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(parsed) => parsed,
                Err(error) => {
                    findings.push(Finding::Fixture {
                        path: display,
                        detail: format!("is not well-formed JSON: {error}"),
                    });
                    continue;
                }
            };
            let outcome = (root_type.roundtrip)(&parsed);
            if tail.starts_with("invalid/") {
                let key = crate::fixtures::expectation_key(&contract.fixtures_dir, &tail);
                let Some(expectation) = expectations.get(&key) else {
                    continue; // R13 already reported the missing registration.
                };
                check_invalid(&display, expectation, &outcome, &mut findings);
            } else {
                check_accepted(&display, &parsed, &outcome, &mut findings);
            }
        }
    }
    findings
}

/// A `valid/` or `compat/` fixture must parse and must lose nothing.
fn check_accepted(
    display: &str,
    parsed: &serde_json::Value,
    outcome: &Result<serde_json::Value, String>,
    findings: &mut Vec<Finding>,
) {
    match outcome {
        Ok(reemitted) => {
            let dropped = ratatoskr_identifiers::dropped_field_pointers(parsed, reemitted);
            if !dropped.is_empty() {
                findings.push(Finding::Fixture {
                    path: display.to_owned(),
                    detail: format!("loses {} on round-trip", dropped.join(", ")),
                });
            }
        }
        Err(error) => findings.push(Finding::Fixture {
            path: display.to_owned(),
            detail: format!(
                "must be accepted by the canonical Rust type but was rejected: {error}"
            ),
        }),
    }
}

/// An `invalid/` fixture must be rejected by exactly the layers it declares.
fn check_invalid(
    display: &str,
    expectation: &Expectation,
    outcome: &Result<serde_json::Value, String>,
    findings: &mut Vec<Finding>,
) {
    let declares_serde = expectation.rejected_by.contains(&Layer::Serde);
    match (declares_serde, outcome) {
        (true, Ok(_)) => findings.push(Finding::Fixture {
            path: display.to_owned(),
            detail: "declares rejected_by = [\"serde\"] but the canonical Rust type accepted it"
                .to_owned(),
        }),
        (true, Err(error)) => {
            if !error.contains(&expectation.error_contains) {
                findings.push(Finding::Fixture {
                    path: display.to_owned(),
                    detail: format!(
                        "was rejected with {error:?}, which does not contain {:?}",
                        expectation.error_contains
                    ),
                });
            }
        }
        (false, Err(error)) => findings.push(Finding::Fixture {
            path: display.to_owned(),
            detail: format!(
                "does not declare the serde layer but the canonical Rust type rejected it: {error}"
            ),
        }),
        (false, Ok(_)) => {}
    }
}
