//! Author, media, relation and folder value types (spec: identity, media-by-reference,
//! relation and folder-membership requirements).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::{
    BlobOwner, BlobRef, ContentDigest, DigestAlgorithm, DigestHex, EntityLocalId, MediaType,
};
use ratatoskr_social_contracts::{
    AuthorHandle, DisplayName, PostPermalink, PostText, SocialAuthor, SocialFolderMembership,
    SocialMediaItem, SocialMediaKind, SocialRelation, SocialRelationKind,
};

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn digest() -> ContentDigest {
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        hex: DigestHex::parse(DIGEST).expect("a SHA-256 digest"),
    }
}

fn blob_ref() -> BlobRef {
    BlobRef {
        owner_service: BlobOwner::parse("ratatoskr-x").expect("a legal blob owner"),
        digest: digest(),
        media_type: MediaType::parse("image/jpeg").expect("a legal media type"),
        length_bytes: 2048,
    }
}

#[test]
fn provider_handle_rejects_the_at_prefix_and_control_characters() {
    let handle = AuthorHandle::parse("example_user").expect("a bare screen name parses");
    assert_eq!(handle.as_str(), "example_user");
    let wire = serde_json::to_string(&handle).unwrap();
    assert_eq!(wire, "\"example_user\"");
    assert_eq!(
        serde_json::from_str::<AuthorHandle>(&wire).expect("round trip"),
        handle
    );

    // The @ prefix is presentation, not identity; the wire carries the bare name.
    assert!(AuthorHandle::parse("@example_user").is_err());
    // Handles are single-line tokens.
    assert!(AuthorHandle::parse("example\nuser").is_err());
    assert!(AuthorHandle::parse("").is_err());
}

#[test]
fn display_name_bans_control_characters_but_takes_unicode() {
    let name = DisplayName::parse("Ada Lovelace ✅").expect("a display name parses");
    let wire = serde_json::to_string(&name).unwrap();
    assert_eq!(
        serde_json::from_str::<DisplayName>(&wire).expect("round trip"),
        name
    );

    assert!(DisplayName::parse("Ada\u{7f}").is_err());
    assert!(DisplayName::parse("line one\nline two").is_err());
    assert!(DisplayName::parse("").is_err());
}

#[test]
fn post_text_preserves_line_breaks_and_bans_other_controls() {
    let text = PostText::parse("first line\nsecond line\ttabbed\r\nthird")
        .expect("line breaks are content");
    let wire = serde_json::to_string(&text).unwrap();
    let decoded: PostText = serde_json::from_str(&wire).expect("round trip");
    assert_eq!(decoded.as_str(), "first line\nsecond line\ttabbed\r\nthird");

    for rejected in [
        "bell\u{7}",
        "vertical tab\u{b}",
        "escape\u{1b}",
        "del\u{7f}",
    ] {
        assert!(
            PostText::parse(rejected).is_err(),
            "{rejected:?} carries a banned control character"
        );
    }
    assert!(PostText::parse("").is_err());
}

#[test]
fn permalink_requires_an_absolute_https_form() {
    let link =
        PostPermalink::parse("https://x.com/example_user/status/123").expect("an HTTPS permalink");
    let wire = serde_json::to_string(&link).unwrap();
    assert_eq!(
        serde_json::from_str::<PostPermalink>(&wire).expect("round trip"),
        link
    );

    // Lower bound only: no scheme, or whitespace anywhere, is refused. Full URL syntax is the
    // producer's validation job.
    assert!(PostPermalink::parse("http://x.com/example_user/status/123").is_err());
    assert!(PostPermalink::parse("x.com/example_user/status/123").is_err());
    assert!(PostPermalink::parse("https://x.com/a b").is_err());
    assert!(PostPermalink::parse("").is_err());
}

#[test]
fn author_round_trips_with_and_without_optional_members() {
    let author = SocialAuthor {
        platform: ratatoskr_social_contracts::Platform::parse("x").expect("a platform"),
        external_author_id: EntityLocalId::parse("1234567890").expect("a provider id"),
        handle: Some(AuthorHandle::parse("example_user").expect("a handle")),
        display_name: Some(DisplayName::parse("Example User").expect("a display name")),
    };
    let wire = serde_json::to_string(&author).unwrap();
    assert_eq!(
        serde_json::from_str::<SocialAuthor>(&wire).expect("round trip"),
        author
    );
    assert!(wire.contains("\"external_author_id\""));

    let minimal = SocialAuthor {
        platform: ratatoskr_social_contracts::Platform::parse("threads").expect("a platform"),
        external_author_id: EntityLocalId::parse("provider-slug.01").expect("a provider id"),
        handle: None,
        display_name: None,
    };
    let minimal_wire = serde_json::to_string(&minimal).unwrap();
    assert!(
        !minimal_wire.contains("handle"),
        "None serializes as absent"
    );
    assert!(!minimal_wire.contains("display_name"));
    assert_eq!(
        serde_json::from_str::<SocialAuthor>(&minimal_wire).expect("round trip"),
        minimal
    );
}

#[test]
fn media_item_carries_a_reference_never_bytes() {
    let item = SocialMediaItem {
        media_kind: SocialMediaKind::parse("image").expect("a kind"),
        blob: blob_ref(),
        alt_text: Some(PostText::parse("A chart of test results.").expect("alt text")),
    };
    let wire = serde_json::to_string(&item).unwrap();
    assert!(!wire.contains("base64"), "no byte payloads on the wire");
    assert_eq!(
        serde_json::from_str::<SocialMediaItem>(&wire).expect("round trip"),
        item
    );

    let bare = SocialMediaItem {
        media_kind: SocialMediaKind::parse("video").expect("a kind"),
        blob: blob_ref(),
        alt_text: None,
    };
    let bare_wire = serde_json::to_string(&bare).unwrap();
    assert!(!bare_wire.contains("alt_text"), "None serializes as absent");
}

#[test]
fn relation_names_its_target_post() {
    let relation = SocialRelation {
        relation_kind: SocialRelationKind::parse("quote").expect("a relation kind"),
        target_post_id: EntityLocalId::parse("9988776655").expect("a provider post id"),
    };
    let wire = serde_json::to_string(&relation).unwrap();
    assert_eq!(
        serde_json::from_str::<SocialRelation>(&wire).expect("round trip"),
        relation
    );

    // Unknown relation kinds survive like unknown platforms do.
    let pinned = SocialRelation {
        relation_kind: SocialRelationKind::parse("pinned").expect("grammatical"),
        target_post_id: EntityLocalId::parse("1").expect("a provider post id"),
    };
    let pinned_wire = serde_json::to_string(&pinned).unwrap();
    assert_eq!(
        serde_json::from_str::<SocialRelation>(&pinned_wire)
            .expect("unknown kinds round trip")
            .relation_kind
            .as_str(),
        "pinned"
    );
}

#[test]
fn folder_membership_is_valid_with_or_without_a_name() {
    let named = SocialFolderMembership {
        folder_id: EntityLocalId::parse("folder-abc").expect("a provider folder id"),
        folder_name: Some(DisplayName::parse("Read later").expect("a folder name")),
    };
    let wire = serde_json::to_string(&named).unwrap();
    assert_eq!(
        serde_json::from_str::<SocialFolderMembership>(&wire).expect("round trip"),
        named
    );

    let unnamed = SocialFolderMembership {
        folder_id: EntityLocalId::parse("folder-abc").expect("a provider folder id"),
        folder_name: None,
    };
    let unnamed_wire = serde_json::to_string(&unnamed).unwrap();
    assert!(!unnamed_wire.contains("folder_name"));
}
