//! `BlobRef` — test I-9. `ARCHITECTURE.md` S14: blob references are opaque and expose no
//! filesystem path or signed storage URL.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::BlobRef;

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// I-9.
#[test]
fn rejects_urls_and_filesystem_paths() {
    for rejected in [
        "https://signed.example/blob?sig=abc",
        "s3://bucket/k",
        "/var/lib/blob",
        "blob:md5-0123456789abcdef0123456789abcdef",
        "blob:sha256:",
        &format!("blob:sha256:{}", DIGEST.to_uppercase()),
        &format!("blob:sha512:{DIGEST}"),
    ] {
        assert!(
            BlobRef::parse(rejected).is_err(),
            "{rejected} must not parse as a BlobRef"
        );
    }
}

/// The handle carries the digest and nothing else.
#[test]
fn exposes_only_the_digest() {
    let wire = format!("blob:sha256:{DIGEST}");
    let handle = BlobRef::parse(&wire).expect("a content-addressed handle is legal");
    assert_eq!(handle.sha256_hex(), DIGEST);
    assert_eq!(handle.as_str(), wire);
    assert_eq!(
        serde_json::to_string(&handle).unwrap(),
        format!("\"{wire}\"")
    );
    assert_eq!(wire.len(), BlobRef::MAX_LEN);
}
