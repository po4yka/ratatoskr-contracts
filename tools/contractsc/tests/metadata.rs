//! `contracts.toml` validated against reality — tests M-1 to M-11 (specification rules R1–R14).
//!
//! Running the same rules from `cargo test` as from `cargo contracts check` is deliberate: a bad
//! metadata edit fails the test suite too, not only the gate.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{Finding, GENERATOR_VERSION, Metadata, generate, metadata, registry};

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
fn generated(metadata: &Metadata) -> BTreeMap<PathBuf, String> {
    generate(metadata, GENERATOR_VERSION).expect("the committed contracts generate")
}

/// Every finding one metadata rule produced against the committed tree.
fn findings_for(rule: &str) -> Vec<String> {
    let metadata = committed();
    let generated = generated(&metadata);
    metadata::validate(&metadata, &repo_root(), &generated)
        .into_iter()
        .filter_map(|finding| match finding {
            Finding::Metadata {
                rule: found,
                detail,
            } if found == rule => Some(detail),
            _ => None,
        })
        .collect()
}

/// Asserts one rule is clean, printing every detail it reported when it is not.
fn assert_rule_clean(rule: &str) {
    let details = findings_for(rule);
    assert!(details.is_empty(), "{rule}: {}", details.join("; "));
}

/// M-1. R2 — a contract cannot ship undeclared, and the metadata cannot describe a contract that
/// does not exist. Both directions, so neither list can drift ahead of the other.
#[test]
fn registry_and_metadata_agree_both_directions() {
    let metadata = committed();
    let compiled: Vec<&str> = registry::root_types()
        .iter()
        .map(|root| root.rust_path)
        .collect();
    let declared: Vec<String> = metadata
        .contracts
        .iter()
        .flat_map(|contract| contract.root_types.iter())
        .map(|root| root.rust_path.clone())
        .collect();

    for path in &compiled {
        assert!(
            declared.iter().any(|d| d == path),
            "{path} is generated but undeclared"
        );
    }
    for path in &declared {
        assert!(
            compiled.contains(&path.as_str()),
            "{path} is declared but never generated"
        );
    }
    assert_rule_clean("R2");
}

/// M-2. The registry is sorted by `rust_path`, so review diffs stay stable.
#[test]
fn registry_is_sorted_by_rust_path() {
    let paths: Vec<&str> = registry::root_types()
        .iter()
        .map(|root| root.rust_path)
        .collect();
    let mut sorted = paths.clone();
    sorted.sort_unstable();
    assert_eq!(paths, sorted);
}

/// M-3. R3 and R4 — the output layout and every `$id` are mechanical, not folklore.
#[test]
fn output_paths_and_schema_ids_follow_convention() {
    assert_rule_clean("R3");
    assert_rule_clean("R4");
}

/// M-4. R5 — every declared path exists and the canonical source names its own root type, which
/// is what catches a file move.
#[test]
fn declared_paths_exist_and_name_their_type() {
    assert_rule_clean("R5");
}

/// M-5. R6 — every generated path is claimed by exactly one root type.
#[test]
fn every_generated_path_is_claimed_exactly_once() {
    assert_rule_clean("R6");
}

/// M-6. R7 — `AGENTS.md` "Review requirements": producers and consumers are identified, and drawn
/// from the closed service vocabulary.
#[test]
fn service_tokens_are_known_and_non_empty() {
    assert_rule_clean("R7");
    for contract in committed().contracts {
        assert!(
            !contract.producers.is_empty(),
            "{} declares no producer",
            contract.id
        );
        assert!(
            !contract.consumers.is_empty(),
            "{} declares no consumer",
            contract.id
        );
    }
}

/// M-7. R8 — every `crate_name` is a real workspace member.
#[test]
fn crate_names_are_workspace_members() {
    assert_rule_clean("R8");
}

/// M-8. R9 — the event registration agrees with the payload type, including the major version and
/// `EventPayload::EVENT_TYPE`.
#[test]
fn event_metadata_matches_the_payload_type() {
    assert_rule_clean("R9");

    let declared = registry::event_payload_types();
    let mut checked = 0usize;
    for contract in committed().contracts {
        let Some(event) = contract.event else {
            continue;
        };
        let parsed = ratatoskr_event_envelope::EventType::parse(&event.event_type)
            .expect("a registered event type parses");
        assert_eq!(parsed.major(), contract.major_version, "{}", contract.id);
        assert_eq!(
            declared.get(event.payload_type.as_str()).copied(),
            Some(event.event_type.as_str()),
            "{} names an event the payload type does not declare",
            contract.id
        );
        checked += 1;
    }
    assert_eq!(
        checked, 4,
        "four event contracts are registered; update this pin when the registry changes"
    );
}

/// M-9. A credential-bearing contract cannot be declared at all: `privacy = "credential"` fails at
/// **parse**, so the `SECURITY.md` baseline is enforced by the type system rather than by a rule
/// somebody could forget to run.
#[test]
fn credential_privacy_class_is_unrepresentable() {
    let text = std::fs::read_to_string(repo_root().join(Metadata::FILE_NAME))
        .expect("contracts.toml is committed");
    let poisoned = text.replacen("\"boundary_metadata\"", "\"credential\"", 1);
    assert_ne!(poisoned, text, "the privacy anchor still exists");
    let error = Metadata::parse(&poisoned).expect_err("credential is not a privacy class");
    assert!(error.contains("credential"), "{error}");
}

/// M-10. R13 — every fixture is registered and every registration has a file, so no fixture is
/// silently untested.
#[test]
fn every_fixture_is_registered_and_every_registration_exists() {
    assert_rule_clean("R13");
}

/// M-11. R12 — the wire type keeps an open entity-kind vocabulary while a typo in a fixture still
/// fails the build.
#[test]
fn fixture_entity_kinds_are_known() {
    assert_rule_clean("R12");
}
