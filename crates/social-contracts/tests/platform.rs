//! `Platform` and `SocialMediaKind` — the open validated tokens (spec: platform and media-kind
//! requirements).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::IdentifierError;
use ratatoskr_social_contracts::{Platform, SocialMediaKind};

#[test]
fn platform_accepts_the_three_known_tokens() {
    for known in ["x", "instagram", "threads"] {
        let parsed = Platform::parse(known).expect("a known platform parses");
        assert_eq!(parsed.as_str(), known);
        assert_eq!(parsed.to_string(), known);
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            format!("\"{known}\"")
        );
        let decoded: Platform = serde_json::from_str(&format!("\"{known}\"")).unwrap();
        assert_eq!(decoded, parsed);
    }
}

#[test]
fn platform_preserves_an_unknown_but_grammatical_token() {
    // A platform this build has never heard of survives verbatim: a new provider must not
    // break a running consumer.
    let parsed = Platform::parse("bluesky").expect("a grammatical token parses");
    assert_eq!(parsed.as_str(), "bluesky");

    let wire = serde_json::to_string(&parsed).unwrap();
    let decoded: Platform = serde_json::from_str(&wire).expect("round trip");
    assert_eq!(decoded.as_str(), "bluesky");
}

#[test]
fn platform_rejects_tokens_outside_the_grammar() {
    assert!(matches!(
        Platform::parse(""),
        Err(IdentifierError::Empty { .. })
    ));
    for rejected in ["X", "Instagram", "x threads", "x-threads", "018f"] {
        assert!(
            matches!(
                Platform::parse(rejected),
                Err(IdentifierError::PatternMismatch { .. })
            ),
            "{rejected:?} must not parse as a Platform"
        );
    }
}

#[test]
fn media_kind_accepts_known_and_preserves_unknown() {
    for known in ["image", "video", "animated"] {
        assert_eq!(
            SocialMediaKind::parse(known)
                .expect("a known kind parses")
                .as_str(),
            known
        );
    }

    let unknown = SocialMediaKind::parse("audio_room").expect("a grammatical kind parses");
    let wire = serde_json::to_string(&unknown).unwrap();
    let decoded: SocialMediaKind = serde_json::from_str(&wire).expect("round trip");
    assert_eq!(decoded.as_str(), "audio_room");

    assert!(matches!(
        SocialMediaKind::parse(""),
        Err(IdentifierError::Empty { .. })
    ));
    assert!(matches!(
        SocialMediaKind::parse("Image"),
        Err(IdentifierError::PatternMismatch { .. })
    ));
}
