//! The completeness vocabulary and the cross-field invariants A1-A3.

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes run on values the test itself built, so an               out-of-bounds or missing member panics the test, which is the reporting mechanism"
)]

mod common;

use ratatoskr_ai_archive_contracts::{
    AiArchiveCompleteness, AiArchiveImport, AiArchiveSnapshot, AiGap,
};
use ratatoskr_identifiers::{Extensions, SafeMessage};

/// All six states parse by their wire tokens; an unknown state stops processing.
#[test]
fn completeness_states_parse_by_their_tokens() {
    for token in [
        "complete",
        "conversations_complete",
        "structurally_partial",
        "assets_partial",
        "unknown",
        "failed_validation",
    ] {
        let parsed: AiArchiveCompleteness =
            serde_json::from_str(&format!("\"{token}\"")).expect(token);
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            format!("\"{token}\"")
        );
    }

    let error = serde_json::from_str::<AiArchiveCompleteness>("\"mostly_fine\"")
        .expect_err("an unknown state stops processing");
    assert!(error.to_string().contains("unknown variant"), "{error}");
}

/// Invariant A1: every non-complete state requires at least one gap.
#[test]
fn incomplete_without_gap_is_rejected() {
    for state in [
        AiArchiveCompleteness::ConversationsComplete,
        AiArchiveCompleteness::StructurallyPartial,
        AiArchiveCompleteness::AssetsPartial,
        AiArchiveCompleteness::Unknown,
        AiArchiveCompleteness::FailedValidation,
    ] {
        let mut import = common::import_with_report(common::report(state, 1, 0));
        import.completeness_report.gaps.clear();

        let error =
            serde_json::from_value::<AiArchiveImport>(serde_json::to_value(&import).unwrap())
                .expect_err("a non-complete state without a gap is rejected");
        assert!(
            error.to_string().contains("requires at least one gap"),
            "unexpected error: {error}"
        );
    }

    // Complete with zero gaps parses.
    let complete =
        common::import_with_report(common::report(AiArchiveCompleteness::Complete, 2, 0));
    serde_json::from_value::<AiArchiveImport>(serde_json::to_value(&complete).unwrap())
        .expect("a complete import may carry no gap");
}

/// Complete imports may still warn; warnings do not reduce completeness.
#[test]
fn complete_import_may_carry_warnings() {
    let mut complete =
        common::import_with_report(common::report(AiArchiveCompleteness::Complete, 0, 0));
    complete.warnings = vec![common::warning("ai_archive.export_unlisted_file_skipped")];

    let decoded: AiArchiveImport =
        serde_json::from_value(serde_json::to_value(&complete).unwrap()).expect("parses");
    assert_eq!(decoded.warnings.len(), 1);
}

/// Invariants A2/A3: counts must match what the snapshot actually carries.
#[test]
fn snapshot_counts_must_match_the_tree() {
    let mut snapshot = common::snapshot_with_conversations(2);

    snapshot.import.completeness_report.conversation_count = 5;
    let error =
        serde_json::from_value::<AiArchiveSnapshot>(serde_json::to_value(&snapshot).unwrap())
            .expect_err("conversation_count must equal the carried conversations");
    assert!(
        error
            .to_string()
            .contains("conversation_count 5 does not match the 2"),
        "unexpected error: {error}"
    );

    let mut snapshot = common::snapshot_with_conversations(2);
    snapshot.import.completeness_report.gap_count = 7;
    let error =
        serde_json::from_value::<AiArchiveSnapshot>(serde_json::to_value(&snapshot).unwrap())
            .expect_err("gap_count must equal the gaps length");
    assert!(
        error
            .to_string()
            .contains("gap_count 7 does not match the 0"),
        "{error}"
    );

    // Consistent counts pass.
    let consistent = common::snapshot_with_conversations(2);
    serde_json::from_value::<AiArchiveSnapshot>(serde_json::to_value(&consistent).unwrap())
        .expect("consistent counts parse");
}

/// The head alone enforces A1 but cannot check tree counts.
#[test]
fn head_alone_enforces_only_its_own_invariant() {
    let import = common::import_with_report(common::report(
        AiArchiveCompleteness::StructurallyPartial,
        99,
        1,
    ));

    let decoded: AiArchiveImport = serde_json::from_value(serde_json::to_value(&import).unwrap())
        .expect("the head has no tree to disagree with");
    assert_eq!(decoded.completeness_report.conversation_count, 99);
    assert_eq!(decoded.completeness_report.gaps.len(), 1);
}

/// Gaps carry open kinds and survive round trips with all members set.
#[test]
fn gaps_round_trip_with_every_member() {
    let mut extensions = Extensions::new();
    extensions.insert("provider_error_slug", serde_json::json!("attachment_quota"));

    let gap = AiGap {
        gap_kind: ratatoskr_ai_archive_contracts::AiGapKind::parse("missing_file")
            .expect("a legal kind"),
        detail: SafeMessage::parse("One attachment file was absent from the export.")
            .expect("a safe message"),
        external_ref: Some(common::local_id("file-9")),
        affected_count: Some(3),
        extensions,
    };
    let wire = serde_json::to_value(&gap).unwrap();
    assert_eq!(wire["affected_count"], 3);
    let decoded: AiGap = serde_json::from_value(wire).expect("and deserializes");
    assert_eq!(decoded, gap);
}
