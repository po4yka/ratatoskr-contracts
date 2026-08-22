//! The O-2-style drift guard: a snapshot carrying every field of every node must survive a
//! canonical round trip without losing anything.

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes run on values the test itself built, so an               out-of-bounds or missing member panics the test, which is the reporting mechanism"
)]

mod common;

use ratatoskr_ai_archive_contracts::{
    AiArchiveCompleteness, AiArchiveImport, AiArchiveSnapshot, AiAsset, AiAssetKind, AiAuthorRole,
    AiCitation, AiContentPart, AiConversation, AiFileName, AiGap, AiMessage, AiProject,
    AiSourceUrl, AiText, AiToolCall, AiToolName, AiToolOutcome, AiToolResult,
};
use ratatoskr_error_contracts::{ErrorCode, FieldPath};
use ratatoskr_identifiers::{Extensions, SafeMessage, TenantRef, dropped_field_pointers};

/// Every documented member name of the whole tree appears in the wire rendering.
const EVERY_MEMBER: &[&str] = &[
    // snapshot
    "import",
    "projects",
    "conversations",
    // import head
    "ai_archive_id",
    "provider",
    "owner",
    "source_export",
    "imported_at",
    "parser_name",
    "parser_version",
    "completeness_report",
    "warnings",
    // report
    "completeness",
    "conversation_count",
    "message_count",
    "asset_count",
    "gap_count",
    "gaps",
    // gap
    "gap_kind",
    "detail",
    "external_ref",
    "affected_count",
    // project
    "ai_project_id",
    "external_project_id",
    "title",
    "description",
    "instructions",
    "provider_created_at",
    "provider_updated_at",
    // conversation
    "ai_conversation_id",
    "external_conversation_id",
    "project_ref",
    "messages",
    "content_digest",
    // message
    "author_role",
    "parent_message_id",
    "parts",
    "model",
    // part kinds on the wire
    "part_kind",
    "text",
    "markdown",
    "image",
    "asset",
    "citation",
    "tool_call",
    "tool_result",
    // asset / citation / tools
    "asset_kind",
    "blob",
    "file_name",
    "url",
    "source_blob",
    "tool_call_id",
    "tool_name",
    "outcome",
    "output_text",
];

/// Builds a snapshot in which every node carries every member, including an unknown content
/// part and preserved extensions at three levels.
#[allow(
    clippy::too_many_lines,
    reason = "a fixture builder naming every field"
)]
fn snapshot_carrying_every_field() -> AiArchiveSnapshot {
    let mut head_extensions = Extensions::new();
    head_extensions.insert("provider_export_id", serde_json::json!("export-2026-08"));

    let citation = AiCitation {
        title: Some(common::title("A cited page")),
        url: Some(AiSourceUrl::parse("https://example.com/cited-source").expect("an HTTPS URL")),
        source_blob: Some(common::blob_ref_for("ratatoskr-claude", "text/html", 512)),
    };
    let unknown_part = serde_json::json!({
        "part_kind": "hologram_projection",
        "holo_data": { "depth_map_ref": "blob-42" }
    });

    let message = AiMessage {
        external_message_id: common::local_id("msg-0001"),
        author_role: AiAuthorRole::Assistant,
        parent_message_id: Some(common::local_id("msg-0000")),
        parts: vec![
            AiContentPart::Text {
                text: common::text("Plain preamble."),
            },
            AiContentPart::Markdown {
                markdown: common::text("# Answer\nWith **emphasis**."),
            },
            AiContentPart::Image {
                image: common::blob_ref_for("ratatoskr-chatgpt", "image/png", 2048),
            },
            AiContentPart::Asset {
                asset: AiAsset {
                    asset_kind: AiAssetKind::parse("artifact").expect("a kind"),
                    blob: common::blob_ref_for("ratatoskr-claude", "text/html", 4096),
                    file_name: Some(AiFileName::parse("analysis.html").expect("a name")),
                },
            },
            AiContentPart::Citation { citation },
            AiContentPart::ToolCall {
                tool_call: AiToolCall {
                    tool_call_id: Some(common::local_id("call_abc")),
                    tool_name: AiToolName::parse("web_search").expect("a tool"),
                },
            },
            AiContentPart::ToolResult {
                tool_result: AiToolResult {
                    tool_call_id: Some(common::local_id("call_abc")),
                    outcome: AiToolOutcome::Succeeded,
                    output_text: Some(common::text("3 results")),
                },
            },
            AiContentPart::Unknown(unknown_part),
        ],
        model: Some(
            ratatoskr_ai_archive_contracts::AiModelName::parse("claude-opus-4-6").expect("a model"),
        ),
        provider_created_at: Some(common::instant("2026-08-02T10:00:00Z")),
        parser_name: common::parser_name("claude_export"),
        parser_version: common::parser_version("1.4.2"),
        extensions: Extensions::new(),
    };

    let conversation = AiConversation {
        ai_conversation_id: common::conversation_id(),
        provider: common::provider("claude"),
        external_conversation_id: Some(common::local_id("conv-77")),
        owner: TenantRef::parse(common::OWNER).expect("a legal owner"),
        project_ref: Some(common::project_ref()),
        title: Some(common::title("Borrow checker deep dive")),
        provider_created_at: Some(common::instant("2026-08-02T09:59:00Z")),
        provider_updated_at: Some(common::instant("2026-08-02T10:05:00Z")),
        messages: vec![message],
        content_digest: common::digest(),
        parser_name: common::parser_name("claude_export"),
        parser_version: common::parser_version("1.4.2"),
        warnings: vec![common::warning("ai_archive.message_timestamp_absent")],
        extensions: Extensions::new(),
    };

    let project = AiProject {
        ai_project_id: common::project_id(),
        provider: common::provider("claude"),
        external_project_id: Some(common::local_id("prd_01j9")),
        title: common::title("Contract work"),
        description: Some(AiText::parse("Everything about the wire contracts.").expect("text")),
        instructions: Some(AiText::parse("Answer tersely.\nCite sources.").expect("text")),
        provider_created_at: Some(common::instant("2026-07-01T08:00:00Z")),
        provider_updated_at: Some(common::instant("2026-08-01T09:30:00Z")),
        parser_name: common::parser_name("claude_export"),
        parser_version: common::parser_version("1.4.2"),
        extensions: Extensions::new(),
    };

    let mut import = common::import_with_report(common::report(
        AiArchiveCompleteness::StructurallyPartial,
        1,
        1,
    ));
    import.parser_name = common::parser_name("claude_export");
    import.extensions = head_extensions;

    AiArchiveSnapshot {
        import,
        projects: vec![project],
        conversations: vec![conversation],
        extensions: Extensions::new(),
    }
}

/// The full tree survives serialization, parsing and re-serialization byte-stably.
#[test]
fn roundtrips_a_snapshot_carrying_every_field() {
    let snapshot = snapshot_carrying_every_field();

    let first = ratatoskr_identifiers::canonical_json(&snapshot).expect("serializes");
    let reparsed: AiArchiveSnapshot =
        serde_json::from_str(&first).expect("the canonical form parses");
    assert_eq!(reparsed, snapshot);

    let second = ratatoskr_identifiers::canonical_json(&reparsed).expect("re-serializes");
    assert_eq!(first, second, "the canonical form is byte-stable");

    let sent: serde_json::Value = serde_json::from_str(&first).expect("wire is JSON");
    let received: serde_json::Value = serde_json::from_str(&second).expect("wire is JSON");
    assert_eq!(
        dropped_field_pointers(&sent, &received),
        Vec::<String>::new(),
        "no member may be dropped"
    );

    for member in EVERY_MEMBER {
        assert!(
            first.contains(&format!("\"{member}\"")),
            "member {member} must appear in the rendering"
        );
    }

    // The unknown part came through verbatim, including its exotic members.
    let message = reparsed
        .conversations
        .first()
        .expect("one conversation")
        .messages
        .first()
        .expect("one message");
    let unknown = message
        .parts
        .get(7)
        .expect("the eighth part is the unknown one");
    let unknown = unknown
        .as_unknown()
        .expect("the eighth part is the unknown one");
    assert_eq!(unknown["holo_data"]["depth_map_ref"], "blob-42");

    // The citation's URL survived even though fixtures cannot carry URLs.
    match message
        .parts
        .get(4)
        .expect("the fifth part is the citation")
    {
        AiContentPart::Citation { citation } => {
            assert_eq!(
                citation
                    .url
                    .as_ref()
                    .map(ratatoskr_ai_archive_contracts::AiSourceUrl::as_str),
                Some("https://example.com/cited-source")
            );
        }
        other => panic!("expected the citation part, got {other:?}"),
    }
}

/// The import head alone — the imported-event payload — round-trips through both the derived
/// path and the hand-written mirror.
#[test]
fn roundtrips_the_import_head_alone() {
    let head = snapshot_carrying_every_field().import;
    let wire = ratatoskr_identifiers::canonical_json(&head).expect("serializes");
    let reparsed: AiArchiveImport = serde_json::from_str(&wire).expect("parses");
    assert_eq!(reparsed, head);
    assert_eq!(
        ratatoskr_identifiers::canonical_json(&reparsed).unwrap(),
        wire
    );

    // Preserved extensions survive the mirror.
    assert_eq!(
        reparsed.extensions.keys().collect::<Vec<_>>(),
        ["provider_export_id"]
    );
}

/// A helper gap builder proving the report's gap entries ride along too.
#[test]
fn gaps_and_warnings_survive_the_tree() {
    let snapshot = snapshot_carrying_every_field();
    let report = &snapshot.import.completeness_report;
    assert_eq!(report.gaps.len(), 1);
    let gap: &AiGap = report.gaps.first().expect("one gap");
    assert_eq!(gap.gap_kind.as_str(), "missing_file");
    assert_eq!(
        gap.detail.as_str(),
        "One attachment file was absent from the export."
    );
    assert!(SafeMessage::parse(gap.detail.as_str()).is_ok());
    let conversation = snapshot.conversations.first().expect("one conversation");
    assert_eq!(
        conversation.warnings.len(),
        1,
        "conversation-level warnings travel beside the record"
    );
    assert_eq!(
        conversation
            .warnings
            .first()
            .expect("one warning")
            .code
            .to_string(),
        ErrorCode::parse("ai_archive.message_timestamp_absent")
            .expect("a legal code")
            .to_string()
    );
    assert!(FieldPath::parse("/conversations/0/warnings/0").is_ok());
}

/// Keeps `AiGap` referenced at the type level for the drift-guard reader.
#[test]
fn gap_type_is_reachable() {
    fn takes_gap(_gap: AiGap) {}
    takes_gap(
        snapshot_carrying_every_field()
            .import
            .completeness_report
            .gaps
            .first()
            .expect("one gap")
            .clone(),
    );
}
