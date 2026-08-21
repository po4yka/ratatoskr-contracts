//! `BlobRef` — content-addressed reference tests.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::BlobRef;

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn a_reference_without_a_digest_is_refused() {
    let complete = serde_json::json!({
        "owner_service": "ratatoskr-extractor",
        "digest": { "algorithm": "sha256", "hex": DIGEST },
        "media_type": "text/html",
        "length_bytes": 42
    });
    serde_json::from_value::<BlobRef>(complete.clone()).expect("the complete reference is valid");

    let mut missing = complete;
    missing
        .as_object_mut()
        .expect("the fixture is an object")
        .remove("digest");
    let error = serde_json::from_value::<BlobRef>(missing).expect_err("digest is required");
    assert!(
        error.to_string().contains("missing field `digest`"),
        "{error}"
    );
}

#[test]
fn rejects_urls_and_filesystem_paths() {
    for rejected in [
        r#""https://signed.example/blob?sig=abc""#,
        r#""s3://bucket/k""#,
        r#""/var/lib/blob""#,
        r#""blob:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef""#,
    ] {
        assert!(
            serde_json::from_str::<BlobRef>(rejected).is_err(),
            "{rejected} must not parse as a structured BlobRef"
        );
    }
}

#[test]
fn carries_only_resolution_and_verification_facts() {
    let reference: BlobRef = serde_json::from_value(serde_json::json!({
        "owner_service": "ratatoskr-extractor",
        "digest": { "algorithm": "sha256", "hex": DIGEST },
        "media_type": "text/html",
        "length_bytes": 42
    }))
    .expect("a complete reference is valid");
    let wire = serde_json::to_value(reference).expect("the reference serializes");

    assert_eq!(
        wire.get("owner_service"),
        Some(&serde_json::json!("ratatoskr-extractor"))
    );
    assert_eq!(wire.get("length_bytes"), Some(&serde_json::json!(42)));
    assert!(wire.get("path").is_none());
    assert!(wire.get("url").is_none());
}
