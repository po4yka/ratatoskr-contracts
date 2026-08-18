//! Secret and PII fixture scanning — tests S-1 to S-4 (`docs/TESTING.md` T7).
//!
//! `ARCHITECTURE.md` S12: "Fixtures must use synthetic data and must not contain provider tokens,
//! private exports, personal messages, or real user identifiers." `THREAT_MODEL.md` names the
//! sensitive-fixture leak.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{Finding, secrets};

/// The repository root, derived from the compiled-in manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

/// Every `(path, pattern)` the scanner reported over the committed fixture tree.
fn tree_hits() -> Vec<(String, &'static str)> {
    secrets::scan_tree(&repo_root())
        .into_iter()
        .filter_map(|finding| match finding {
            Finding::Secret { path, pattern } => Some((path, pattern)),
            _ => None,
        })
        .collect()
}

/// The hits whose pattern name is in `names`.
fn hits_for(names: &[&str]) -> Vec<(String, &'static str)> {
    tree_hits()
        .into_iter()
        .filter(|(_, pattern)| names.contains(pattern))
        .collect()
}

/// Asserts the scanner fires on a planted sample, so a clean tree means "nothing there" rather
/// than "the rule is broken".
fn assert_fires(sample: &str, expected: &str) {
    let hits: Vec<&str> = secrets::scan_text("planted.json", sample)
        .into_iter()
        .filter_map(|finding| match finding {
            Finding::Secret { pattern, .. } => Some(pattern),
            _ => None,
        })
        .collect();
    assert!(
        hits.contains(&expected),
        "{expected} did not fire on {sample:?}; got {hits:?}"
    );
}

/// S-1. No fixture carries a credential or a credential-shaped JSON key.
#[test]
fn fixtures_contain_no_secret_or_credential_patterns() {
    let hits = hits_for(&[
        "private-key",
        "openai-key",
        "github-token",
        "slack-token",
        "aws-access-key",
        "bearer-header",
        "jwt",
        "credential-shaped-key",
    ]);
    assert!(hits.is_empty(), "{hits:?}");

    assert_fires("-----BEGIN RSA PRIVATE KEY-----", "private-key");
    assert_fires("\"authorization\": \"x\"", "credential-shaped-key");
    assert_fires("AKIAIOSFODNN7EXAMPLE", "aws-access-key");
    assert_fires("eyJhbGciOi.eyJzdWIi.sig", "jwt");
}

/// S-2. Every UUID in every fixture is from the reserved synthetic block, which turns "no real
/// user identifiers" from a review convention into a mechanical check.
#[test]
fn fixture_uuids_are_from_the_reserved_synthetic_range() {
    let hits = hits_for(&["uuid-outside-the-reserved-synthetic-block"]);
    assert!(hits.is_empty(), "{hits:?}");
    assert_fires(
        "\"event_id\": \"3f2504e0-4f89-11d3-9a0c-0305e82c3301\"",
        "uuid-outside-the-reserved-synthetic-block",
    );
}

/// S-3. No fixture carries personal data or a storage location (`ARCHITECTURE.md` S14).
#[test]
fn fixtures_contain_no_email_addresses_phone_numbers_urls_or_paths() {
    let hits = hits_for(&[
        "email-address",
        "e164-phone-number",
        "at-handle",
        "url",
        "object-store-url",
        "absolute-path",
        "windows-path",
    ]);
    assert!(hits.is_empty(), "{hits:?}");

    assert_fires("\"owner\": \"someone@example.com\"", "email-address");
    assert_fires("\"phone\": \"+14155550123\"", "e164-phone-number");
    assert_fires("\"href\": \"https://example.com/x\"", "url");
    assert_fires("\"blob\": \"s3://bucket/key\"", "object-store-url");
    assert_fires("\"blob\": \"/var/lib/blob/x\"", "absolute-path");
    assert_fires("\"blob\": \"C:\\\\data\"", "windows-path");
}

/// S-4. No acceptable fixture carries a floating-point number, which extends the byte-stability
/// rule from the generated schemas to the fixtures.
///
/// `invalid/` fixtures are exempt by design: one of them exists precisely to prove that a
/// fractional `progress_percent` is refused, and a fixture that is never rendered cannot make a
/// rendering unstable.
#[test]
fn fixtures_contain_no_floating_point_numbers() {
    let hits = hits_for(&["floating-point-number"]);
    assert!(hits.is_empty(), "{hits:?}");
    assert_fires("{\"progress_percent\": 15.5}", "floating-point-number");
}

/// The scanner sees a non-empty tree, and it sees **every** file in it, not only `*.json`.
///
/// A scan that silently found no files would make every assertion above vacuous, and a scan that
/// skipped `fixtures/invalid-expectations.toml` would make them vacuous over the one file in the
/// tree that quotes fixture values verbatim (§7.4: "every byte under `fixtures/**`").
#[test]
fn the_scanner_reads_every_committed_fixture_file() {
    let root = repo_root();
    let scanned = ratatoskr_contractsc::fixtures::walk_all(&root.join("fixtures"));
    assert!(
        scanned.len() >= 40,
        "only {} fixture files were discovered",
        scanned.len()
    );
    assert!(
        scanned.contains(&root.join("fixtures/invalid-expectations.toml")),
        "the non-JSON registry inside fixtures/ is not scanned"
    );
}
