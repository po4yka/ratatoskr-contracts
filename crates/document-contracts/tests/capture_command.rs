//! The content capture command (XR-021 CONTRACTS.md S11).

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes write into values the test itself built"
)]

use ratatoskr_document_contracts::ContentCaptureRequested;
use ratatoskr_event_envelope::{CommandEnvelope, CommandPayload};
use serde_json::json;

fn key() -> serde_json::Value {
    json!({
        "algorithm": "sha256",
        "hex": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    })
}

fn blob() -> serde_json::Value {
    json!({
        "owner_service": "ratatoskr-telegram",
        "digest": key(),
        "media_type": "application/pdf",
        "length_bytes": 13264
    })
}

fn url_payload() -> serde_json::Value {
    json!({
        "operation_id": "018f0000-0000-7000-8000-000000000a11",
        "idempotency_key": key(),
        "url": "https://example.org/articles/borrow-checker"
    })
}

fn blob_payload() -> serde_json::Value {
    json!({
        "operation_id": "018f0000-0000-7000-8000-000000000a12",
        "idempotency_key": key(),
        "blob": blob()
    })
}

fn decode(value: serde_json::Value) -> Result<ContentCaptureRequested, serde_json::Error> {
    serde_json::from_value(value)
}

#[test]
fn url_payload_round_trips() {
    assert_eq!(
        ContentCaptureRequested::COMMAND_TYPE,
        "content.capture.requested.v1"
    );
    let decoded = decode(url_payload()).expect("a url capture decodes");
    assert_eq!(
        decoded
            .url
            .as_ref()
            .map(ratatoskr_document_contracts::CaptureUrl::as_str),
        Some("https://example.org/articles/borrow-checker")
    );
    assert!(decoded.blob.is_none());
    decoded.validate().expect("exactly one source");
    assert_eq!(
        serde_json::to_value(&decoded).expect("serializes"),
        url_payload()
    );

    // http is as acceptable as https; the extractor's address policy decides what is fetchable.
    let mut plain = url_payload();
    plain["url"] = json!("http://example.org/a?b=c#d");
    decode(plain).expect("an http url decodes");

    // The command travels inside a command envelope and comes back typed.
    let mut envelope: CommandEnvelope = serde_json::from_value(json!({
        "command_id": "018f0000-0000-7000-8000-000000000b01",
        "command_type": "content.capture.requested.v1",
        "issued_at": "2026-08-27T06:00:00Z",
        "producer": "ratatoskr-platform",
        "aggregate_id": "operation:018f0000-0000-7000-8000-000000000a11",
        "correlation_id": "operation:018f0000-0000-7000-8000-000000000a11",
        "tenant_id": "user:018f0000-0000-7000-8000-000000000005",
        "schema_version": 1,
        "payload": {}
    }))
    .expect("the envelope parses");
    envelope.set_payload(&decoded).expect("an object payload");
    assert_eq!(
        envelope
            .payload_as::<ContentCaptureRequested>()
            .expect("typed read"),
        decoded
    );
}

#[test]
fn blob_payload_round_trips() {
    let decoded = decode(blob_payload()).expect("a blob capture decodes");
    let reference = decoded.blob.as_ref().expect("a blob source");
    assert_eq!(reference.owner_service.as_str(), "ratatoskr-telegram");
    assert_eq!(reference.media_type.as_str(), "application/pdf");
    assert_eq!(reference.length_bytes, 13264);
    assert!(decoded.url.is_none());
    decoded.validate().expect("exactly one source");
    assert_eq!(
        serde_json::to_value(&decoded).expect("serializes"),
        blob_payload()
    );

    let mut with_extension = blob_payload();
    with_extension["origin"] = json!("share_sheet");
    let decoded = decode(with_extension.clone()).expect("an additive member decodes");
    assert_eq!(
        serde_json::to_value(&decoded).expect("serializes"),
        with_extension,
        "additive members are preserved"
    );
}

#[test]
fn payload_with_both_url_and_blob_is_rejected() {
    let mut both = url_payload();
    both["blob"] = blob();
    let error = decode(both).expect_err("two sources are one too many");
    assert!(error.to_string().contains("both were given"), "{error}");
}

#[test]
fn payload_with_neither_is_rejected() {
    let neither = json!({
        "operation_id": "018f0000-0000-7000-8000-000000000a14",
        "idempotency_key": key()
    });
    let error = decode(neither).expect_err("a capture needs a source");
    assert!(error.to_string().contains("neither was given"), "{error}");
}

#[test]
fn blob_with_non_sha256_algorithm_is_rejected() {
    let mut weak = blob_payload();
    weak["blob"]["digest"] = json!({
        "algorithm": "sha1",
        "hex": "da39a3ee5e6b4b0d3255bfef95601890afd80709"
    });
    assert!(
        decode(weak).is_err(),
        "only sha256 digests name stored bytes"
    );

    let mut weak_key = url_payload();
    weak_key["idempotency_key"]["algorithm"] = json!("md5");
    assert!(
        decode(weak_key).is_err(),
        "the idempotency key is a sha256 digest"
    );
}

#[test]
fn non_http_url_is_rejected() {
    for url in [
        "ftp://example.org/archive.pdf",
        "javascript:alert(1)",
        "file:///etc/passwd",
        "example.org/articles",
        "https://",
        "https://example.org/has space",
        "",
    ] {
        let mut payload = url_payload();
        payload["url"] = json!(url);
        assert!(decode(payload).is_err(), "{url:?} must be refused");
    }
}

#[test]
fn oversized_url_is_rejected() {
    let prefix = "https://example.org/";
    let mut at_limit = url_payload();
    at_limit["url"] = json!(format!("{prefix}{}", "a".repeat(2048 - prefix.len())));
    decode(at_limit).expect("2048 characters are the limit");

    let mut over = url_payload();
    over["url"] = json!(format!("{prefix}{}", "a".repeat(2049 - prefix.len())));
    let error = decode(over).expect_err("2049 characters are too many");
    assert!(
        error.to_string().contains("contract maximum is 2048"),
        "{error}"
    );
}
