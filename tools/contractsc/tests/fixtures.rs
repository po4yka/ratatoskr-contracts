//! Schema self-validation and fixtures — tests F-1 to F-6 (`docs/TESTING.md` T1).
//!
//! `jsonschema` is a dev-dependency, so this is the only place the `json_schema` layer of
//! `fixtures/invalid-expectations.toml` can be exercised. The `serde` layer is exercised here and
//! by the library half of `cargo contracts check`.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ratatoskr_contractsc::fixtures::{Layer, expectation_key, load_expectations, walk_json};
use ratatoskr_contractsc::{GENERATOR_VERSION, Metadata, generate, registry};

/// One contract's fixtures, paired with everything needed to judge them.
struct Subject {
    /// Repository-relative fixture root, e.g. `fixtures/core/event-envelope`.
    fixtures_dir: String,
    /// The generated schema document.
    schema: serde_json::Value,
    /// Fully qualified Rust path of the canonical type.
    rust_path: String,
}

/// The repository root, derived from the compiled-in manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

/// The committed metadata.
fn committed() -> Metadata {
    let path = repo_root().join(Metadata::FILE_NAME);
    let text = std::fs::read_to_string(&path).expect("contracts.toml is committed");
    Metadata::parse(&text).expect("contracts.toml parses")
}

/// Every generated artifact, in memory.
fn generated() -> BTreeMap<PathBuf, String> {
    generate(&committed(), GENERATOR_VERSION).expect("the committed contracts generate")
}

/// Every contract with fixtures, in metadata order.
///
/// A declared contract that produces no generated output is a failure, not a subject to drop: a
/// silently shrinking input set would let F-2 to F-6 pass over fewer and fewer families while
/// still reporting ok.
fn subjects() -> Vec<Subject> {
    let generated = generated();
    committed()
        .contracts
        .iter()
        .map(|contract| {
            let declared = contract.root_types.first().unwrap_or_else(|| {
                panic!("contract {} declares no root type", contract.id);
            });
            let body = generated
                .get(Path::new(&declared.output))
                .unwrap_or_else(|| panic!("{} produces no generated output", declared.rust_path));
            Subject {
                fixtures_dir: contract.fixtures_dir.clone(),
                schema: serde_json::from_str(body).expect("a generated schema is JSON"),
                rust_path: declared.rust_path.clone(),
            }
        })
        .collect()
}

/// Every acceptable fixture of one contract — `valid/` and `compat/` together.
///
/// Non-empty by assertion: a family whose fixtures were deleted must fail the test that covers
/// it, not quietly drop out of the loop.
fn acceptable_fixtures(fixtures_dir: &str) -> Vec<(String, String, serde_json::Value)> {
    let found: Vec<(String, String, serde_json::Value)> = ["valid/", "compat/"]
        .into_iter()
        .flat_map(|bucket| fixtures_in(fixtures_dir, bucket))
        .collect();
    assert!(
        !found.is_empty(),
        "{fixtures_dir} has no acceptable fixture, so this family is untested"
    );
    found
}

/// Every fixture under one contract whose repository-relative tail starts with `bucket`.
fn fixtures_in(fixtures_dir: &str, bucket: &str) -> Vec<(String, String, serde_json::Value)> {
    let root = repo_root().join(fixtures_dir);
    walk_json(&root)
        .into_iter()
        .filter_map(|absolute| {
            let tail = absolute
                .strip_prefix(&root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");
            if !tail.starts_with(bucket) {
                return None;
            }
            let text = std::fs::read_to_string(&absolute).ok()?;
            let parsed = serde_json::from_str(&text)
                .unwrap_or_else(|error| panic!("{fixtures_dir}/{tail} is not JSON: {error}"));
            Some((tail, text, parsed))
        })
        .collect()
}

/// Deserializes into the canonical Rust type, then re-serializes to a `serde_json::Value`.
fn roundtrip(rust_path: &str, value: &serde_json::Value) -> Result<serde_json::Value, String> {
    let root = registry::root_types();
    let root = root
        .iter()
        .find(|candidate| candidate.rust_path == rust_path)
        .unwrap_or_else(|| panic!("{rust_path} is registered"));
    (root.roundtrip)(value)
}

/// Deserializes into the canonical Rust type, then renders it with the repository's canonical
/// JSON rule — which serializes the **typed value**, so struct fields keep declaration order.
fn canonical(rust_path: &str, value: &serde_json::Value) -> Result<String, String> {
    macro_rules! render {
        ($ty:ty) => {
            serde_json::from_value::<$ty>(value.clone())
                .map_err(|error| error.to_string())
                .and_then(|typed| {
                    ratatoskr_identifiers::canonical_json(&typed).map_err(|e| e.to_string())
                })
        };
    }
    match rust_path {
        "ratatoskr_ai_archive_contracts::AiArchiveImport" => {
            render!(ratatoskr_ai_archive_contracts::AiArchiveImport)
        }
        "ratatoskr_ai_archive_contracts::AiArchiveSnapshot" => {
            render!(ratatoskr_ai_archive_contracts::AiArchiveSnapshot)
        }
        "ratatoskr_ai_archive_contracts::AiConversation" => {
            render!(ratatoskr_ai_archive_contracts::AiConversation)
        }
        "ratatoskr_ai_archive_contracts::AiConversationAdded" => {
            render!(ratatoskr_ai_archive_contracts::AiConversationAdded)
        }
        "ratatoskr_ai_archive_contracts::AiConversationUpdated" => {
            render!(ratatoskr_ai_archive_contracts::AiConversationUpdated)
        }
        "ratatoskr_ai_archive_contracts::AiMessage" => {
            render!(ratatoskr_ai_archive_contracts::AiMessage)
        }
        "ratatoskr_ai_archive_contracts::AiProject" => {
            render!(ratatoskr_ai_archive_contracts::AiProject)
        }
        "ratatoskr_document_contracts::Document" => {
            render!(ratatoskr_document_contracts::Document)
        }
        "ratatoskr_error_contracts::ErrorEnvelope" => {
            render!(ratatoskr_error_contracts::ErrorEnvelope)
        }
        "ratatoskr_event_envelope::EventEnvelope" => {
            render!(ratatoskr_event_envelope::EventEnvelope)
        }
        "ratatoskr_identifiers::BlobRef" => render!(ratatoskr_identifiers::BlobRef),
        "ratatoskr_operation_contracts::OperationProgressed" => {
            render!(ratatoskr_operation_contracts::OperationProgressed)
        }
        "ratatoskr_operation_contracts::OperationReported" => {
            render!(ratatoskr_operation_contracts::OperationReported)
        }
        "ratatoskr_operation_contracts::OperationSnapshot" => {
            render!(ratatoskr_operation_contracts::OperationSnapshot)
        }
        "ratatoskr_social_contracts::SocialSourceCaptured" => {
            render!(ratatoskr_social_contracts::SocialSourceCaptured)
        }
        "ratatoskr_social_contracts::SocialSourceSnapshot" => {
            render!(ratatoskr_social_contracts::SocialSourceSnapshot)
        }
        "ratatoskr_social_contracts::SocialSourceUpdated" => {
            render!(ratatoskr_social_contracts::SocialSourceUpdated)
        }
        other => panic!("{other} is registered but has no canonical renderer in this test"),
    }
}

/// A format-checking validator for one generated schema.
///
/// `should_validate_formats(true)` is not the default in `jsonschema` 0.49: without it
/// `{"format": "uuid"}` accepts `"nope"`. Every identifier schema also carries an explicit
/// `pattern`, so a misconfigured validator elsewhere still cannot silently pass garbage.
fn validator(schema: &serde_json::Value) -> jsonschema::Validator {
    jsonschema::options()
        .should_validate_formats(true)
        .build(schema)
        .expect("a generated schema compiles as draft 2020-12")
}

/// F-1. Every generated file is a valid draft 2020-12 schema document.
#[test]
fn generated_schemas_are_valid_draft_2020_12() {
    for (path, body) in generated() {
        let document: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
        jsonschema::meta::validate(&document)
            .unwrap_or_else(|error| panic!("{} is not a valid schema: {error}", path.display()));
    }
}

/// F-2. Every `valid/` and `compat/` fixture validates against its published schema, with format
/// checking switched on.
#[test]
fn valid_fixtures_pass_json_schema() {
    for subject in subjects() {
        let validator = validator(&subject.schema);
        for (tail, _, parsed) in acceptable_fixtures(&subject.fixtures_dir) {
            validator.validate(&parsed).unwrap_or_else(|error| {
                panic!(
                    "{}/{tail} fails its own schema: {error}",
                    subject.fixtures_dir
                )
            });
        }
    }
}

/// F-3. Every `valid/` and `compat/` fixture deserializes into its canonical Rust type.
#[test]
fn valid_fixtures_pass_serde() {
    for subject in subjects() {
        for (tail, _, parsed) in acceptable_fixtures(&subject.fixtures_dir) {
            roundtrip(&subject.rust_path, &parsed).unwrap_or_else(|error| {
                panic!(
                    "{}/{tail} is rejected by serde: {error}",
                    subject.fixtures_dir
                )
            });
        }
    }
}

/// F-4. Every `valid/` fixture is byte-canonical: two-space pretty, one trailing newline, LF, and
/// fields in struct-declaration order. Catches unsorted keys and non-canonical instants.
#[test]
fn valid_fixtures_are_byte_canonical() {
    for subject in subjects() {
        let found = fixtures_in(&subject.fixtures_dir, "valid/");
        assert!(
            !found.is_empty(),
            "{} has no valid/ fixture, so byte-canonicality is untested for it",
            subject.fixtures_dir
        );
        for (tail, text, parsed) in found {
            let rendered = canonical(&subject.rust_path, &parsed)
                .unwrap_or_else(|error| panic!("{}/{tail}: {error}", subject.fixtures_dir));
            assert_eq!(
                rendered, text,
                "{}/{tail} is not byte-canonical",
                subject.fixtures_dir
            );
        }
    }
}

/// F-5. Nothing a fixture carries is silently discarded on the way through the Rust type — the
/// tolerant-reader guarantee, asserted rather than assumed.
#[test]
fn valid_fixtures_roundtrip_losslessly() {
    for subject in subjects() {
        for (tail, _, parsed) in acceptable_fixtures(&subject.fixtures_dir) {
            let reemitted = roundtrip(&subject.rust_path, &parsed)
                .unwrap_or_else(|error| panic!("{}/{tail}: {error}", subject.fixtures_dir));
            let dropped = ratatoskr_identifiers::dropped_field_pointers(&parsed, &reemitted);
            assert!(
                dropped.is_empty(),
                "{}/{tail} loses {dropped:?}",
                subject.fixtures_dir
            );
        }
    }
}

/// F-6. Every `invalid/` fixture is rejected by **every** layer it declares, **accepted** by every
/// layer it does not, and the rejecting error text contains the declared substring.
///
/// The both-directions assertion is the point: a fixture that is invalid by accident — a typo
/// rather than the contract rule it claims to exercise — fails here.
#[test]
fn invalid_fixtures_are_rejected_for_the_declared_reason() {
    let expectations = load_expectations(&repo_root());
    let mut checked = 0usize;

    for subject in subjects() {
        let validator = validator(&subject.schema);
        for (tail, _, parsed) in fixtures_in(&subject.fixtures_dir, "invalid/") {
            let key = expectation_key(&subject.fixtures_dir, &tail);
            let expectation = expectations.get(&key).unwrap_or_else(|| {
                panic!("{key} has no entry in fixtures/invalid-expectations.toml")
            });

            let schema_error = validator
                .validate(&parsed)
                .err()
                .map(|error| error.to_string());
            let serde_error = roundtrip(&subject.rust_path, &parsed).err();

            let declares_schema = expectation.rejected_by.contains(&Layer::JsonSchema);
            assert_eq!(
                schema_error.is_some(),
                declares_schema,
                "{key}: rejected_by declares json_schema = {declares_schema} but the published \
                 schema said otherwise ({schema_error:?})"
            );
            let declares_serde = expectation.rejected_by.contains(&Layer::Serde);
            assert_eq!(
                serde_error.is_some(),
                declares_serde,
                "{key}: rejected_by declares serde = {declares_serde} but the canonical Rust type \
                 said otherwise ({serde_error:?})"
            );
            assert!(
                !expectation.rejected_by.is_empty(),
                "{key} declares no rejecting layer, so it is not a test of anything"
            );

            // The declared substring is matched against the strongest layer that rejected it: the
            // canonical Rust type when it is listed, otherwise the published schema.
            let text = serde_error.or(schema_error).unwrap_or_default();
            assert!(
                text.contains(&expectation.error_contains),
                "{key} was rejected with {text:?}, which does not contain {:?}",
                expectation.error_contains
            );
            assert!(
                !expectation.reason.trim().is_empty(),
                "{key} declares no reason, so a reviewer cannot tell what it proves"
            );
            checked += 1;
        }
    }
    assert_eq!(
        checked,
        expectations.len(),
        "every registration has exactly one fixture"
    );
}
