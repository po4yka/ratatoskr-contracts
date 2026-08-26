//! `SocialSourceSnapshot` round trips — the drift guard for its hand-written `Deserialize`.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

mod common;

use ratatoskr_identifiers::{canonical_json, dropped_field_pointers};
use ratatoskr_social_contracts::SocialSourceSnapshot;

use common::snapshot_carrying_every_field;

/// The drift guard: a field present on `SocialSourceSnapshot` but missing from its private
/// wire mirror is dropped on parse, and this test fails the moment that happens.
#[test]
fn roundtrips_a_snapshot_carrying_every_field() {
    let snapshot = snapshot_carrying_every_field();

    let rendered = canonical_json(&snapshot).expect("canonical rendering");
    assert!(rendered.ends_with("}\n"));

    let decoded: SocialSourceSnapshot = serde_json::from_str(&rendered).expect("round trip");
    assert_eq!(
        decoded, snapshot,
        "a field present on SocialSourceSnapshot but missing from its private wire mirror is \
         dropped on parse; add it to both"
    );
    assert_eq!(
        canonical_json(&decoded).expect("canonical rendering"),
        rendered,
        "the round trip is byte-stable"
    );

    // Nothing at any nesting level was silently discarded.
    let input: serde_json::Value = serde_json::from_str(&rendered).expect("the rendering is JSON");
    let reemitted: serde_json::Value = serde_json::to_value(&decoded).expect("re-serialization");
    assert_eq!(
        dropped_field_pointers(&input, &reemitted),
        Vec::<String>::new()
    );

    // Every documented member of the wire form is actually on the wire.
    for member in [
        "social_source_id",
        "platform",
        "external_post_id",
        "permalink",
        "owner",
        "author",
        "published_at",
        "captured_at",
        "text",
        "media",
        "relations",
        "folders",
        "content_digest",
        "raw_blob",
        "acquisition",
        "saved_authority",
        "completeness",
        "upstream_availability",
        "checkpoint",
        "warnings",
        "provider_edit_history_count",
    ] {
        assert!(
            rendered.contains(member),
            "{member} is missing from {rendered}"
        );
    }
}

/// Absent optional members serialize as absent, never as null.
#[test]
fn absent_optionals_serialize_as_absent() {
    let mut snapshot = snapshot_carrying_every_field();
    snapshot.permalink = None;
    snapshot.published_at = None;
    snapshot.text = None;
    snapshot.raw_blob = None;
    snapshot.checkpoint = None;

    let rendered = serde_json::to_string(&snapshot).expect("serializes");
    for absent in [
        "permalink",
        "published_at",
        "text",
        "raw_blob",
        "checkpoint",
    ] {
        let member = format!("\"{absent}\":");
        assert!(
            !rendered.contains(&member),
            "{absent} must be absent, not null"
        );
    }

    let decoded: SocialSourceSnapshot = serde_json::from_str(&rendered).expect("round trip");
    assert_eq!(decoded, snapshot);
}

/// An authorless snapshot parses to `None` and re-emits with the member absent, never
/// substituted: absence means authorship was unobservable, and a placeholder would be a claim.
#[test]
fn snapshot_without_author_parses_and_reemits_absent() {
    let rendered = serde_json::to_string(&snapshot_carrying_every_field()).expect("serializes");
    let mut value: serde_json::Value = serde_json::from_str(&rendered).expect("the form is JSON");
    value
        .as_object_mut()
        .expect("a snapshot renders as an object")
        .remove("author");
    let stripped = value.to_string();

    let decoded: SocialSourceSnapshot =
        serde_json::from_str(&stripped).expect("an authorless snapshot parses");
    assert!(decoded.author.is_none());
    assert!(
        !serde_json::to_string(&decoded)
            .expect("re-emits")
            .contains("\"author\":"),
        "re-emission must keep the author absent, not null"
    );
}
