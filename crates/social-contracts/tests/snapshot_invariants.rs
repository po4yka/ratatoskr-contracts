//! Snapshot cross-field invariants (spec: completeness and checkpoint requirements).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

mod common;

use ratatoskr_error_contracts::{ErrorCode, FieldPath, WarningEnvelope};
use ratatoskr_identifiers::{EntityLocalId, Extensions, SafeMessage};
use ratatoskr_social_contracts::{
    CaptureCompleteness, PostPermalink, SocialSourceSnapshot, SyncCheckpointCursor,
    UpstreamAvailability,
};

use common::snapshot_carrying_every_field;

/// A partial capture that explains nothing is unactionable for the consumer.
#[test]
fn partial_without_a_warning_is_rejected() {
    let mut snapshot = snapshot_carrying_every_field();
    snapshot.completeness = CaptureCompleteness::Partial;
    snapshot.warnings = Vec::new();

    let rendered = serde_json::to_string(&snapshot).expect("serializes");
    let error = serde_json::from_str::<SocialSourceSnapshot>(&rendered)
        .expect_err("partial must explain itself");
    assert!(
        error
            .to_string()
            .contains("completeness `partial` requires at least one warning"),
        "unexpected error: {error}"
    );
}

/// The same payload with evidence parses, and the warning survives.
#[test]
fn partial_with_a_warning_parses_and_keeps_the_warning() {
    let mut snapshot = snapshot_carrying_every_field();
    snapshot.completeness = CaptureCompleteness::Partial;
    snapshot.upstream_availability = UpstreamAvailability::Unavailable;
    snapshot.warnings = vec![WarningEnvelope {
        code: ErrorCode::parse("social.source.media_download_failed").expect("a legal code"),
        message: SafeMessage::parse("One video could not be downloaded before it expired.")
            .expect("a safe message"),
        field_path: Some(FieldPath::parse("/media/1").expect("a path")),
        extensions: Extensions::new(),
    }];

    let rendered = serde_json::to_string(&snapshot).expect("serializes");
    let decoded: SocialSourceSnapshot =
        serde_json::from_str(&rendered).expect("evidence-backed partial parses");
    assert_eq!(decoded, snapshot);
    assert_eq!(decoded.warnings.len(), 1);
}

/// A complete capture may still carry warnings that did not reduce completeness; the rule is
/// asymmetric on purpose and the asymmetry is the documented contract.
#[test]
fn complete_with_warnings_is_accepted() {
    let snapshot = snapshot_carrying_every_field();
    assert_eq!(snapshot.completeness, CaptureCompleteness::Complete);
    assert!(!snapshot.warnings.is_empty());

    let rendered = serde_json::to_string(&snapshot).expect("serializes");
    let decoded: SocialSourceSnapshot =
        serde_json::from_str(&rendered).expect("complete-with-warnings parses");
    assert_eq!(decoded, snapshot);
}

/// A control character cannot hide in an opaque cursor.
#[test]
fn checkpoint_rejects_control_characters() {
    assert!(SyncCheckpointCursor::parse("line one\nline two").is_err());
    assert!(SyncCheckpointCursor::parse("tab\tseparated").is_err());
    assert!(SyncCheckpointCursor::parse("space separated").is_err());

    // Printable provider grammar this repository knows nothing about is fine.
    let cursor = SyncCheckpointCursor::parse("eyJhIjpbMSwyLDMsXX0+/=").expect("printable ASCII");
    let wire = serde_json::to_string(&cursor).unwrap();
    assert_eq!(
        serde_json::from_str::<SyncCheckpointCursor>(&wire)
            .expect("round trip")
            .as_str(),
        "eyJhIjpbMSwyLDMsXX0+/="
    );
}

/// A source observed deleted upstream keeps everything captured before it went away.
#[test]
fn deleted_upstream_keeps_a_complete_capture() {
    let mut snapshot = snapshot_carrying_every_field();
    snapshot.upstream_availability = UpstreamAvailability::DeletedUpstream;
    snapshot.checkpoint = None;
    snapshot.relations = Vec::new();
    snapshot.folders = Vec::new();
    snapshot.author.handle = None;
    snapshot.author.display_name = None;
    snapshot.permalink = Some(common_permalink());

    let rendered = serde_json::to_string(&snapshot).expect("serializes");
    let decoded: SocialSourceSnapshot = serde_json::from_str(&rendered)
        .expect("deleted upstream does not invalidate what was captured");
    assert_eq!(
        decoded.upstream_availability,
        UpstreamAvailability::DeletedUpstream
    );
    assert!(decoded.text.is_some());
    assert_eq!(decoded.media.len(), 2);
}

fn common_permalink() -> ratatoskr_social_contracts::PostPermalink {
    PostPermalink::parse("https://x.com/example_user/status/1234567890123456789")
        .expect("an HTTPS permalink")
}

/// A folder-less, media-less minimal record parses — the first-day shape every producer can
/// emit.
#[test]
fn minimal_first_day_shape_parses() {
    let mut snapshot = snapshot_carrying_every_field();
    snapshot.permalink = None;
    snapshot.published_at = None;
    snapshot.text = None;
    snapshot.media = Vec::new();
    snapshot.relations = Vec::new();
    snapshot.folders = Vec::new();
    snapshot.raw_blob = None;
    snapshot.checkpoint = None;
    snapshot.warnings = Vec::new();
    snapshot.extensions = Extensions::new();
    snapshot.author.external_author_id = EntityLocalId::parse("provider-slug.01").expect("id");

    let rendered = serde_json::to_string(&snapshot).expect("serializes");
    let decoded: SocialSourceSnapshot = serde_json::from_str(&rendered).expect("parses");
    assert_eq!(decoded, snapshot);
}
