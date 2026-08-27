//! Shared builders for the ai-archive-contracts integration tests.

#![allow(
    dead_code,
    reason = "each test binary consumes a different subset of this module"
)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_ai_archive_contracts::{
    AiAssetKind, AiAuthorRole, AiContentPart, AiConversation, AiFileName, AiMessage, AiText,
    AiTitle, ParserName, ParserVersion,
};
use ratatoskr_identifiers::{
    AiConversationId, AiProjectId, BlobOwner, BlobRef, ContentDigest, DigestAlgorithm, DigestHex,
    EntityLocalId, EntityRef, Extensions, MediaType, TenantRef, WireTimestamp,
};

pub(crate) const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
pub(crate) const OWNER: &str = "user:018f0000-0000-7000-8000-000000000005";
pub(crate) const ARCHIVE_UUID: &str = "018f0000-0000-7000-8000-000000000401";
pub(crate) const PROJECT_UUID: &str = "018f0000-0000-7000-8000-000000000402";
pub(crate) const CONVERSATION_UUID: &str = "018f0000-0000-7000-8000-000000000403";
pub(crate) const PROJECT_REF: &str = "ai_project:018f0000-0000-7000-8000-000000000402";

pub(crate) fn text(raw: &str) -> AiText {
    AiText::parse(raw).expect("legal text")
}

pub(crate) fn title(raw: &str) -> AiTitle {
    AiTitle::parse(raw).expect("a legal title")
}

pub(crate) fn instant(raw: &str) -> WireTimestamp {
    WireTimestamp::parse(raw).expect("a canonical instant")
}

pub(crate) fn provider(raw: &str) -> ratatoskr_ai_archive_contracts::AiProvider {
    ratatoskr_ai_archive_contracts::AiProvider::parse(raw).expect("a legal provider")
}

pub(crate) fn archive_id() -> ratatoskr_identifiers::AiArchiveId {
    ratatoskr_identifiers::AiArchiveId::parse(ARCHIVE_UUID).expect("a canonical UUID")
}

pub(crate) fn project_id() -> AiProjectId {
    AiProjectId::parse(PROJECT_UUID).expect("a canonical UUID")
}

pub(crate) fn conversation_id() -> AiConversationId {
    AiConversationId::parse(CONVERSATION_UUID).expect("a canonical UUID")
}

/// The conversation identity with its last three hex digits replaced by `index`.
pub(crate) fn conversation_id_at(index: usize) -> AiConversationId {
    let mut raw = String::from(CONVERSATION_UUID);
    let keep = raw.len() - 3;
    raw.replace_range(keep.., &format!("{index:03x}"));
    AiConversationId::parse(&raw).expect("a canonical UUID")
}

pub(crate) fn project_ref() -> EntityRef {
    EntityRef::parse(PROJECT_REF).expect("a legal project reference")
}

pub(crate) fn digest() -> ContentDigest {
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        hex: DigestHex::parse(DIGEST).expect("a SHA-256 digest"),
    }
}

pub(crate) fn blob_ref(media_type: &str) -> BlobRef {
    blob_ref_for("ratatoskr-chatgpt", media_type, 2048)
}

pub(crate) fn blob_ref_for(owner: &str, media_type: &str, length_bytes: u64) -> BlobRef {
    BlobRef {
        owner_service: BlobOwner::parse(owner).expect("a legal blob owner"),
        digest: digest(),
        media_type: MediaType::parse(media_type).expect("a legal media type"),
        length_bytes,
    }
}

pub(crate) fn local_id(raw: &str) -> EntityLocalId {
    EntityLocalId::parse(raw).expect("a legal local id")
}

pub(crate) fn asset_kind(raw: &str) -> AiAssetKind {
    AiAssetKind::parse(raw).expect("a legal asset kind")
}

pub(crate) fn tool_name(raw: &str) -> ratatoskr_ai_archive_contracts::AiToolName {
    ratatoskr_ai_archive_contracts::AiToolName::parse(raw).expect("a legal tool name")
}

pub(crate) fn file_name(raw: &str) -> AiFileName {
    AiFileName::parse(raw).expect("a legal file name")
}

pub(crate) fn parser_name(raw: &str) -> ParserName {
    ParserName::parse(raw).expect("a legal parser name")
}

pub(crate) fn parser_version(raw: &str) -> ParserVersion {
    ParserVersion::parse(raw).expect("a legal parser version")
}

/// A one-message conversation with every optional member omitted.
pub(crate) fn minimal_conversation() -> AiConversation {
    AiConversation {
        ai_conversation_id: conversation_id(),
        provider: provider("chatgpt"),
        external_conversation_id: None,
        owner: TenantRef::parse(OWNER).expect("a legal owner"),
        project_ref: None,
        title: None,
        provider_created_at: None,
        provider_updated_at: None,
        messages: vec![minimal_message()],
        content_digest: digest(),
        parser_name: parser_name("chatgpt_export"),
        parser_version: parser_version("2026.08.1"),
        warnings: Vec::new(),
        extensions: Extensions::new(),
    }
}

/// A message with every optional member omitted.
pub(crate) fn minimal_message() -> AiMessage {
    AiMessage {
        external_message_id: EntityLocalId::parse("msg-0001").expect("a legal local id"),
        author_role: AiAuthorRole::User,
        parent_message_id: None,
        parts: Vec::new(),
        model: None,
        provider_created_at: None,
        parser_name: parser_name("chatgpt_export"),
        parser_version: parser_version("2026.08.1"),
        extensions: Extensions::new(),
    }
}

/// A completeness report whose counts are consistent with `snapshot_with_conversations(n)`
/// when `conversations = n` and the gaps vector carries exactly `gaps` entries.
pub(crate) fn report(
    state: ratatoskr_ai_archive_contracts::AiArchiveCompleteness,
    conversations: u32,
    gaps: usize,
) -> ratatoskr_ai_archive_contracts::AiCompletenessReport {
    use ratatoskr_ai_archive_contracts::{AiCompletenessReport, AiGap, AiGapKind};
    use ratatoskr_identifiers::SafeMessage;

    let gaps_vec: Vec<AiGap> = (0..gaps)
        .map(|index| AiGap {
            gap_kind: AiGapKind::parse("missing_file").expect("a legal kind"),
            detail: SafeMessage::parse("One attachment file was absent from the export.")
                .expect("a safe message"),
            external_ref: Some(EntityLocalId::parse(&format!("file-{index}")).expect("id")),
            affected_count: Some(1),
            extensions: Extensions::new(),
        })
        .collect();

    AiCompletenessReport {
        completeness: state,
        conversation_count: conversations,
        message_count: conversations * 2,
        asset_count: 0,
        gap_count: u32::try_from(gaps_vec.len()).unwrap_or(u32::MAX),
        gaps: gaps_vec,
    }
}

/// An import head carrying exactly this report.
pub(crate) fn import_with_report(
    report: ratatoskr_ai_archive_contracts::AiCompletenessReport,
) -> ratatoskr_ai_archive_contracts::AiArchiveImport {
    ratatoskr_ai_archive_contracts::AiArchiveImport {
        ai_archive_id: archive_id(),
        provider: provider("chatgpt"),
        owner: TenantRef::parse(OWNER).expect("a legal owner"),
        source_export: blob_ref_for("ratatoskr-chatgpt", "application/zip", 1_048_576),
        imported_at: instant("2026-08-17T10:00:00Z"),
        parser_name: parser_name("chatgpt_export"),
        parser_version: parser_version("2026.08.1"),
        completeness_report: report,
        warnings: Vec::new(),
        extensions: Extensions::new(),
    }
}

/// A snapshot with `n` minimal conversations and a matching consistent report.
pub(crate) fn snapshot_with_conversations(
    n: usize,
) -> ratatoskr_ai_archive_contracts::AiArchiveSnapshot {
    use ratatoskr_ai_archive_contracts::AiArchiveSnapshot;
    use ratatoskr_identifiers::ContentDigest;

    let mut import = import_with_report(ratatoskr_ai_archive_contracts::AiCompletenessReport {
        completeness: ratatoskr_ai_archive_contracts::AiArchiveCompleteness::Complete,
        conversation_count: u32::try_from(n).unwrap_or(u32::MAX),
        message_count: u32::try_from(n).unwrap_or(u32::MAX),
        asset_count: 0,
        gap_count: 0,
        gaps: Vec::new(),
    });
    import.completeness_report.message_count = u32::try_from(n * 2).unwrap_or(u32::MAX);

    AiArchiveSnapshot {
        import,
        projects: Vec::new(),
        conversations: (0..n)
            .map(|index| AiConversation {
                ai_conversation_id: conversation_id_at(index),
                provider: provider("chatgpt"),
                external_conversation_id: None,
                owner: TenantRef::parse(OWNER).expect("a legal owner"),
                project_ref: None,
                title: None,
                provider_created_at: None,
                provider_updated_at: None,
                messages: vec![minimal_message(), minimal_message()],
                content_digest: ContentDigest {
                    algorithm: ratatoskr_identifiers::DigestAlgorithm::Sha256,
                    hex: ratatoskr_identifiers::DigestHex::parse(DIGEST).expect("digest"),
                },
                parser_name: parser_name("chatgpt_export"),
                parser_version: parser_version("2026.08.1"),
                warnings: Vec::new(),
                extensions: Extensions::new(),
            })
            .collect(),
        extensions: Extensions::new(),
    }
}

/// A non-blocking warning envelope.
pub(crate) fn warning(code: &str) -> ratatoskr_error_contracts::WarningEnvelope {
    ratatoskr_error_contracts::WarningEnvelope {
        code: ratatoskr_error_contracts::ErrorCode::parse(code).expect("a legal code"),
        message: ratatoskr_identifiers::SafeMessage::parse("A non-blocking problem was recorded.")
            .expect("a safe message"),
        field_path: None,
        extensions: Extensions::new(),
    }
}

/// A legal envelope fragment carrying an empty payload, used as the carrier in composition
/// tests. `set_payload` replaces `event_type` and body.
pub(crate) const MINIMAL_ENVELOPE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000301",
  "event_type": "ai_archive.archive.imported.v1",
  "occurred_at": "2026-08-17T10:00:02Z",
  "producer": "ratatoskr-chatgpt",
  "aggregate_id": "ai_archive:018f0000-0000-7000-8000-000000000401",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000302",
  "schema_version": 1,
  "payload": {}
}
"#;

/// Parses `wire` as the typed part and re-serializes it.
pub(crate) fn roundtrip(wire: &serde_json::Value, part: &AiContentPart) -> serde_json::Value {
    let parsed: AiContentPart = serde_json::from_value(wire.clone())
        .unwrap_or_else(|error| panic!("the wire form must parse: {error}"));
    assert_eq!(&parsed, part, "the wire form must equal the typed value");
    serde_json::to_value(&parsed).expect("the typed value re-serializes")
}

/// Serializes a typed value only.
pub(crate) fn roundtrip_value<T: serde::Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).expect("the typed value serializes")
}
