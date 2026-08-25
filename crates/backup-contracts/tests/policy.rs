//! Policy-document invariants, succession grammar and catalog-drift semantics.
//!
//! Fixture-level rejection is asserted by `cargo contracts check` against
//! `fixtures/invalid-expectations.toml`; this binary asserts the Rust-layer rules those
//! fixtures pin, plus the pure drift functions no fixture can reach.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_backup_contracts::{
    BackupContractError, BackupPriorityHint, MirrorCadence, RepositoryBackupEntry,
    apply_exclusions, entries_absent_from_catalog, uncovered_catalog_repositories,
    validate_policy_succession,
};
use ratatoskr_identifiers::EntityRef;

/// A repository reference with a distinctive tail, so assertions name what they mean.
fn repo_ref(tail: &str) -> EntityRef {
    let raw = format!("repository:018f0000-0000-7000-8000-{tail}");
    EntityRef::parse(&raw).expect("a repository reference")
}

/// A minimal entry naming one repository with default hints.
fn entry(repository_ref: EntityRef) -> RepositoryBackupEntry {
    RepositoryBackupEntry {
        repository_ref,
        mirror_cadence: MirrorCadence::Daily,
        priority_hint: BackupPriorityHint::Standard,
        size_hint_bytes: None,
        exclusions: Vec::new(),
    }
}

/// Version zero cannot open a sequence, whatever came before.
#[test]
fn succession_validation_rejects_zero() {
    let error = validate_policy_succession(0, 5).expect_err("zero starts no sequence");
    assert!(
        matches!(error, BackupContractError::ZeroPolicyVersion),
        "expected ZeroPolicyVersion, got {error:?}"
    );

    let error = validate_policy_succession(0, 0).expect_err("zero starts no sequence");
    assert!(
        matches!(error, BackupContractError::ZeroPolicyVersion),
        "expected ZeroPolicyVersion, got {error:?}"
    );
}

/// A candidate that does not strictly exceed the applied version replays or rewinds state.
#[test]
fn succession_validation_rejects_non_increase() {
    let equal = validate_policy_succession(3, 3).expect_err("equal versions replay state");
    let behind = validate_policy_succession(2, 7).expect_err("a lower candidate rewinds state");

    assert!(
        matches!(equal, BackupContractError::SuccessionNotIncreasing { .. }),
        "expected SuccessionNotIncreasing, got {equal:?}"
    );
    assert!(
        matches!(behind, BackupContractError::SuccessionNotIncreasing { .. }),
        "expected SuccessionNotIncreasing, got {behind:?}"
    );
}

/// A catalog repository the policy does not name is uncovered: default-deny coverage.
#[test]
fn uncovered_catalog_repositories_lists_unnamed_catalog_entries() {
    let covered = repo_ref("000000000504");
    let unnamed = repo_ref("000000000505");
    let catalog = vec![covered.clone(), unnamed.clone()];
    let entries = vec![entry(covered)];

    assert_eq!(
        uncovered_catalog_repositories(&catalog, &entries),
        vec![unnamed],
        "the catalog repository no entry names must be reported as uncovered"
    );
}

/// An entry naming a repository outside the catalog is reportable drift, never a silent skip.
#[test]
fn entries_absent_from_catalog_reports_unknown_entry_references() {
    let known = repo_ref("000000000504");
    let unknown = repo_ref("000000000506");
    let catalog = vec![known.clone()];
    let entries = vec![entry(known), entry(unknown.clone())];

    assert_eq!(
        entries_absent_from_catalog(&catalog, &entries),
        vec![unknown],
        "the entry reference missing from the catalog must be reported"
    );
}

/// Exclusions only ever narrow: the kept set is an order-preserving subset of the candidates,
/// whatever predicate the caller supplies for matching.
#[test]
fn apply_exclusions_yields_a_subset_preserving_order() {
    use ratatoskr_backup_contracts::{BackupExclusion, BackupExclusionScope, ExclusionExpression};

    let values = [
        "refs/heads/main",
        "refs/heads/scratch/x",
        "refs/tags/v1",
        "refs/heads/scratch/y",
    ];
    let exclusions = vec![BackupExclusion {
        scope: BackupExclusionScope::RefsMatching,
        expression: ExclusionExpression::parse("scratch").expect("a legal expression"),
    }];

    let without_exclusions = apply_exclusions(&values, &[], |_value, _| false);
    assert_eq!(
        without_exclusions,
        vec![
            &"refs/heads/main",
            &"refs/heads/scratch/x",
            &"refs/tags/v1",
            &"refs/heads/scratch/y",
        ],
        "no exclusions keep every candidate in order"
    );

    let kept = apply_exclusions(&values, &exclusions, |value, exclusion| {
        value.contains(exclusion.expression.as_str())
    });
    assert_eq!(
        kept,
        vec![&"refs/heads/main", &"refs/tags/v1"],
        "matched candidates drop; everything else keeps its order"
    );
}
