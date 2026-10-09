//! The document-extracted event payload and its envelope composition (XR-021 CONTRACTS.md S09).

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes run on values the test itself decoded"
)]

use ratatoskr_document_contracts::{
    ContentDocumentExtracted, DOCUMENT_BLOB_OWNER, DocumentExtractedError,
};
use ratatoskr_event_envelope::{EventEnvelope, EventPayload};
use ratatoskr_identifiers::{BlobOwner, dropped_field_pointers};

const FIXTURE: &str =
    include_str!("../../../fixtures/events/content.document.extracted.v1/valid/two-blocks.json");

/// A legal envelope carrying an empty payload, used as the carrier in the composition test.
const ENVELOPE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000c01",
  "event_type": "content.document.extracted.v1",
  "occurred_at": "2026-08-27T06:05:00Z",
  "producer": "ratatoskr-extractor",
  "aggregate_id": "document:018f0000-0000-7000-8000-000000000021",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000a11",
  "causation_id": "command:018f0000-0000-7000-8000-000000000b01",
  "tenant_id": "user:018f0000-0000-7000-8000-000000000005",
  "schema_version": 1,
  "payload": {}
}
"#;

fn fixture() -> ContentDocumentExtracted {
    serde_json::from_str(FIXTURE).expect("the fixture decodes")
}

#[test]
fn document_extracted_round_trips_and_requires_extractor_owned_blob() {
    assert_eq!(
        ContentDocumentExtracted::EVENT_TYPE,
        "content.document.extracted.v1"
    );
    assert_eq!(DOCUMENT_BLOB_OWNER, "ratatoskr-extractor");

    let payload = fixture();
    payload.validate().expect("the fixture is acceptable");
    assert_eq!(
        serde_json::to_value(&payload).expect("serializes"),
        serde_json::from_str::<serde_json::Value>(FIXTURE).expect("the fixture is JSON")
    );

    // The payload travels inside a real envelope and comes back typed and whole.
    let mut envelope = EventEnvelope::from_json(ENVELOPE.as_bytes()).expect("the envelope parses");
    envelope.set_payload(&payload).expect("an object payload");
    assert_eq!(envelope.event_type, ContentDocumentExtracted::event_type());
    let wire = envelope.to_canonical_json().expect("re-serializes");
    let reparsed = EventEnvelope::from_json(wire.as_bytes()).expect("round trips");
    assert_eq!(reparsed, envelope);
    let sent: serde_json::Value = serde_json::from_str(&wire).expect("wire is JSON");
    let received: serde_json::Value = serde_json::to_value(&reparsed).expect("re-serialize");
    assert_eq!(
        dropped_field_pointers(&sent, &received),
        Vec::<String>::new(),
        "the envelope must not discard any member of its own payload"
    );
    assert_eq!(
        reparsed
            .payload_as::<ContentDocumentExtracted>()
            .expect("typed read"),
        payload
    );

    // A blob owned by another service is refused.
    let mut foreign = payload.clone();
    foreign.document_blob.owner_service = BlobOwner::parse("ratatoskr-telegram").expect("a name");
    assert_eq!(
        foreign.validate(),
        Err(DocumentExtractedError::ForeignDocumentBlob {
            actual: "ratatoskr-telegram".to_owned()
        })
    );

    // So is a document that breaks the Document IR invariants. A decoded document cannot (the
    // Document IR refuses duplicate block identifiers itself), but a producer that builds one in
    // code can.
    let mut duplicated = payload.clone();
    let first = duplicated
        .document
        .blocks
        .first()
        .cloned()
        .expect("a block");
    duplicated.document.blocks.push(first);
    assert!(matches!(
        duplicated.validate(),
        Err(DocumentExtractedError::InvalidDocument(_))
    ));
    let wire = serde_json::to_value(&duplicated).expect("serializes");
    assert!(
        serde_json::from_value::<ContentDocumentExtracted>(wire).is_err(),
        "the duplicate does not survive decoding either"
    );
}
