//! The conflict events `notes.conflict.raised.v1` and `notes.conflict.resolved.v1` (contract
//! `notes/conflict.v1`, workspace spec `notes-write-command`). They carry hashes, never file
//! content.

#![allow(clippy::panic, reason = "assertions in a test binary")]

mod common;

use common::ContractUnderTest;

fn raised() -> &'static ContractUnderTest {
    common::contract("notes/conflict.v1", "notes.conflict.raised.v1")
}

/// Notes proposes a three-way merge only for Markdown and plain text. A conflict on a binary
/// file that carries a merge proposal is refused by the Rust type and by the schema, so a
/// schema-only consumer sees the rule too.
#[test]
fn attachment_conflict_with_proposal_is_refused() {
    let contract = raised();
    contract.assert_accepted_by_both("attachment.json");
    contract.assert_refused_by_type(
        "attachment-with-proposal.json",
        "merge_proposal is allowed only for text/markdown and text/plain",
    );
    contract.assert_refused_by_both("attachment-with-proposal.json");
}

/// A three-way merge reads the base, the current revision and the writer's content, so a
/// proposal without one of the three hashes is refused by both layers.
#[test]
fn proposal_requires_three_hashes() {
    let contract = raised();
    contract.assert_accepted_by_both("create-collision.json");
    contract.assert_refused_by_type(
        "proposal-without-base.json",
        "merge_proposal requires base_hash, current_hash and writer_hash",
    );
    contract.assert_refused_by_both("proposal-without-base.json");
}

/// A text conflict with its merge proposal goes through the Rust type and the schema whole.
#[test]
fn text_conflict_round_trips() {
    let contract = raised();
    contract.assert_accepted_by_both("text-with-proposal.json");
    contract.assert_accepted_by_both("text-with-overlap.json");
}

/// A group conflict can name all stale files without changing the old event form.
#[test]
fn raised_may_name_every_stale_path() {
    let contract = raised();
    contract.assert_accepted_by_both("text-with-overlap.json");
    let payload = contract.fixture("valid/group-with-stale-paths.json");
    let parsed = (contract.roundtrip)(&payload);
    assert_eq!(
        (parsed.is_ok(), contract.validator().is_valid(&payload)),
        (true, true),
        "(serde, schema) acceptance of the group path list; serde: {parsed:?}"
    );
    contract.assert_accepted_by_both("group-with-stale-paths.json");
    contract.assert_refused_by_both("group-with-invalid-stale-path.json");
}
