//! Shared snapshot builders for the social-contracts integration tests.

#![allow(
    dead_code,
    reason = "each test binary consumes a different subset of this module"
)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_error_contracts::{ErrorCode, FieldPath, WarningEnvelope};
use ratatoskr_identifiers::MediaType;
use ratatoskr_identifiers::{
    BlobOwner, BlobRef, ContentDigest, DigestAlgorithm, DigestHex, EntityLocalId, Extensions,
    SafeMessage, SocialSourceId, TenantRef, WireTimestamp,
};
use ratatoskr_social_contracts::{
    AcquisitionMethod, AuthorHandle, CaptureCompleteness, DisplayName, Platform, PostPermalink,
    PostText, SavedAuthority, SocialAuthor, SocialFolderMembership, SocialMediaItem,
    SocialMediaKind, SocialRelation, SocialRelationKind, SocialSourceSnapshot,
    SyncCheckpointCursor, UpstreamAvailability,
};

pub(crate) const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// A legal envelope fragment carrying an empty payload, used as the carrier in composition
/// tests. `set_payload` replaces `event_type` and body.
pub(crate) const MINIMAL_ENVELOPE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000301",
  "event_type": "social.source.captured.v1",
  "occurred_at": "2026-08-17T10:00:02Z",
  "producer": "ratatoskr-x",
  "aggregate_id": "social_source:018f0000-0000-7000-8000-000000000201",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000302",
  "schema_version": 1,
  "payload": {}
}
"#;

pub(crate) fn instant(raw: &str) -> WireTimestamp {
    WireTimestamp::parse(raw).expect("a canonical instant")
}

pub(crate) fn digest() -> ContentDigest {
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        hex: DigestHex::parse(DIGEST).expect("a SHA-256 digest"),
    }
}

pub(crate) fn blob_ref(media_type: &str) -> BlobRef {
    BlobRef {
        owner_service: BlobOwner::parse("ratatoskr-x").expect("a legal blob owner"),
        digest: digest(),
        media_type: MediaType::parse(media_type).expect("a legal media type"),
        length_bytes: 2048,
    }
}

/// A snapshot in which **every** member is present and non-default, including preserved
/// extensions at the top level.
///
/// This is the drift guard partner of the private wire mirror: a field added to
/// [`SocialSourceSnapshot`] and not to its mirror is dropped on parse, and the round-trip test
/// fails the moment that happens. Do not add a field to one of these alone.
#[allow(
    clippy::too_many_lines,
    reason = "a fixture builder naming every field"
)]
pub(crate) fn snapshot_carrying_every_field() -> SocialSourceSnapshot {
    let mut extensions = Extensions::new();
    extensions.insert("provider_edit_history_count", serde_json::json!(2));

    SocialSourceSnapshot {
        social_source_id: SocialSourceId::parse("018f0000-0000-7000-8000-000000000201")
            .expect("a canonical UUID"),
        platform: Platform::parse("x").expect("a platform"),
        external_post_id: EntityLocalId::parse("1234567890123456789").expect("a provider id"),
        permalink: Some(
            PostPermalink::parse("https://x.com/example_user/status/1234567890123456789")
                .expect("an HTTPS permalink"),
        ),
        owner: TenantRef::parse("user:018f0000-0000-7000-8000-000000000005")
            .expect("a legal owner"),
        author: SocialAuthor {
            platform: Platform::parse("x").expect("a platform"),
            external_author_id: EntityLocalId::parse("987654321").expect("a provider id"),
            handle: Some(AuthorHandle::parse("example_user").expect("a handle")),
            display_name: Some(DisplayName::parse("Example User").expect("a display name")),
        },
        published_at: Some(instant("2026-08-16T09:30:00Z")),
        captured_at: instant("2026-08-17T10:00:00Z"),
        text: Some(PostText::parse("First line.\nSecond line.").expect("post text")),
        media: vec![
            SocialMediaItem {
                media_kind: SocialMediaKind::parse("image").expect("a kind"),
                blob: blob_ref("image/jpeg"),
                alt_text: Some(PostText::parse("A chart of test results.").expect("alt text")),
            },
            SocialMediaItem {
                media_kind: SocialMediaKind::parse("video").expect("a kind"),
                blob: blob_ref("video/mp4"),
                alt_text: None,
            },
        ],
        relations: vec![SocialRelation {
            relation_kind: SocialRelationKind::parse("quote").expect("a relation kind"),
            target_post_id: EntityLocalId::parse("9988776655").expect("a provider post id"),
        }],
        folders: vec![
            SocialFolderMembership {
                folder_id: EntityLocalId::parse("folder-abc").expect("a provider folder id"),
                folder_name: Some(DisplayName::parse("Read later").expect("a folder name")),
            },
            SocialFolderMembership {
                folder_id: EntityLocalId::parse("folder-def").expect("a provider folder id"),
                folder_name: None,
            },
        ],
        content_digest: digest(),
        raw_blob: Some(blob_ref("application/json")),
        acquisition: AcquisitionMethod::OfficialApi,
        saved_authority: SavedAuthority::AuthoritativePlatformState,
        completeness: CaptureCompleteness::Complete,
        upstream_availability: UpstreamAvailability::Available,
        checkpoint: Some(
            SyncCheckpointCursor::parse("eyJib29rbWFya19yZXNwb25zZV9pZCI6IjEifQ==")
                .expect("a cursor token"),
        ),
        warnings: vec![WarningEnvelope {
            code: ErrorCode::parse("social.source.folder_name_truncated").expect("a legal code"),
            message: SafeMessage::parse("One folder name was longer than the provider allows.")
                .expect("a safe message"),
            field_path: Some(FieldPath::parse("/folders/0/folder_name").expect("a path")),
            extensions: Extensions::new(),
        }],
        extensions,
    }
}
