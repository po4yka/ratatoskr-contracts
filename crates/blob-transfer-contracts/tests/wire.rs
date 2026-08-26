//! The wire layer: grammars at the boundary and round trips for every message shape.
//! Tasks 3.1–3.4 of change `add-blob-transfer-contracts`.

#![allow(
    clippy::expect_used,
    reason = "a failed expectation in a test binary is the reporting mechanism"
)]

use ratatoskr_blob_transfer_contracts::{
    UploadChunkReceipt, UploadCompletionOutcome, UploadFinalizeRequest, UploadSessionOpened,
    UploadSessionRequest, UploadSessionState, UploadStatusResponse,
};
use ratatoskr_identifiers::{BlobOwner, ContentDigest, DigestAlgorithm, DigestHex, MediaType};

const TOKEN: &str = "rst_0v8k4a2j9pm1d7n5tp3es6ua";

fn token() -> ratatoskr_blob_transfer_contracts::UploadResumptionToken {
    ratatoskr_blob_transfer_contracts::UploadResumptionToken::parse(TOKEN).expect("grammar holds")
}

fn session_request_json(digest_hex: &str) -> String {
    format!(
        r#"{{
  "declared_size_bytes": 100000,
  "media_type": "application/pdf",
  "digest": {{
    "algorithm": "sha256",
    "hex": "{digest_hex}"
  }},
  "chunk_size_bytes": 65536
}}"#
    )
}

/// 3.1 / 3.2 — malformed tokens and digests stop deserialization.
#[test]
fn token_and_digest_grammars_reject_malformed_input() {
    let receipt = format!(
        r#"{{"resumption_token": "{TOKEN}x", "chunk_index": 0, "received_chunks_count": 1, "idempotent_replay": false}}"#
    );
    assert!(
        serde_json::from_str::<UploadChunkReceipt>(&receipt).is_err(),
        "an uppercase token violates the published pattern"
    );

    assert!(
        ratatoskr_blob_transfer_contracts::UploadResumptionToken::parse(
            "rst_00000000000000000000000",
        )
        .is_err(),
        "23 token characters are below the floor"
    );
    assert!(
        ratatoskr_blob_transfer_contracts::UploadResumptionToken::parse(
            "rst_000000000000000000000000"
        )
        .is_ok(),
        "24 token characters are legal"
    );

    let request =
        session_request_json("111111111111111111111111111111111111111111111111111111111111111F");
    assert!(
        serde_json::from_str::<UploadSessionRequest>(&request).is_err(),
        "a non-lowercase-hex digest violates the grammar"
    );
}

fn opened() -> UploadSessionOpened {
    UploadSessionOpened {
        resumption_token: token(),
        chunk_size_bytes: 65_536,
        expires_at: ratatoskr_identifiers::WireTimestamp::parse("2026-08-26T12:00:00Z")
            .expect("canonical instant"),
        extensions: ratatoskr_identifiers::Extensions::new(),
    }
}

fn status() -> UploadStatusResponse {
    UploadStatusResponse {
        resumption_token: token(),
        session_state: UploadSessionState::Open,
        received_chunks: vec![0, 1, 3],
        received_chunks_count: 3,
        missing_chunks_count: 1,
        extensions: ratatoskr_identifiers::Extensions::new(),
    }
}

fn outcome() -> UploadCompletionOutcome {
    UploadCompletionOutcome::Stored {
        blob_ref: ratatoskr_identifiers::BlobRef {
            owner_service: BlobOwner::parse("ratatoskr-extractor").expect("owner"),
            digest: ContentDigest {
                algorithm: DigestAlgorithm::Sha256,
                hex: DigestHex::parse(
                    "1111111111111111111111111111111111111111111111111111111111111111",
                )
                .expect("digest"),
            },
            media_type: MediaType::parse("application/pdf").expect("media type"),
            length_bytes: 100_000,
        },
        extensions: ratatoskr_identifiers::Extensions::new(),
    }
}

/// Drives one typed value through serialize -> inject an unknown sibling key ->
/// deserialize -> re-serialize, asserting the unknown field survived verbatim.
fn preserves_unknown_fields<T>(value: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    let mut injected = serde_json::to_value(value).expect("shape serializes");
    injected
        .as_object_mut()
        .expect("every shape serializes to an object")
        .insert("future_hint".to_owned(), serde_json::json!(true));
    let text = serde_json::to_string(&injected).expect("render");
    let back: T = serde_json::from_str(&text).expect("typed parse with preserved fields");
    let round = serde_json::to_value(&back).expect("re-render");
    assert_eq!(
        round.get("future_hint"),
        Some(&serde_json::json!(true)),
        "unknown fields are preserved verbatim"
    );
}

/// 3.3 / 3.4 — every message shape round trips, and an unknown sibling field survives
/// re-serialization instead of being dropped.
#[test]
fn every_message_shape_round_trips_and_preserves_unknown_fields() {
    let request =
        session_request_json("1111111111111111111111111111111111111111111111111111111111111111");
    let parsed: UploadSessionRequest = serde_json::from_str(&request).expect("valid declaration");
    assert_eq!(
        serde_json::to_string_pretty(&parsed).expect("render"),
        request.trim_end(),
        "byte-stable round trip"
    );

    preserves_unknown_fields(&opened());
    preserves_unknown_fields(&UploadFinalizeRequest {
        resumption_token: token(),
        extensions: ratatoskr_identifiers::Extensions::new(),
    });
    preserves_unknown_fields(&status());
    preserves_unknown_fields(&outcome());
}
