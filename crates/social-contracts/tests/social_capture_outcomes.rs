//! Closed social-capture terminal and partial outcome taxonomy.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_social_contracts::SocialCaptureOutcomeCode;

#[test]
fn rejects_unknown_social_outcome() {
    for raw in [
        "social.source.unavailable",
        "social.source.deleted",
        "social.linked_article.extraction_failed",
    ] {
        let code = SocialCaptureOutcomeCode::parse(raw).expect("a documented social outcome");
        assert_eq!(code.error_code().as_str(), raw);
        assert_eq!(
            serde_json::to_string(&code).expect("serializes"),
            format!("\"{raw}\"")
        );
        assert_eq!(
            serde_json::from_str::<SocialCaptureOutcomeCode>(&format!("\"{raw}\""))
                .expect("round trips"),
            code
        );
    }

    assert!(SocialCaptureOutcomeCode::parse("social.source.rate_limited").is_err());
    assert!(
        serde_json::from_str::<SocialCaptureOutcomeCode>("\"social.source.rate_limited\"").is_err()
    );

    assert!(SocialCaptureOutcomeCode::SourceUnavailable.is_terminal_source_failure());
    assert!(SocialCaptureOutcomeCode::SourceDeleted.is_terminal_source_failure());
    assert!(
        !SocialCaptureOutcomeCode::LinkedArticleExtractionFailed.is_terminal_source_failure(),
        "a linked-article failure describes partial success after the social post was preserved"
    );
}
