//! `OperationSnapshot` round trips — tests O-2 and O-4.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_error_contracts::{
    ErrorCode, ErrorEnvelope, FieldPath, FieldViolation, TraceId, WarningEnvelope,
};
use ratatoskr_event_envelope::{EventEnvelope, EventPayload, EventType};
use ratatoskr_identifiers::{
    BlobOwner, BlobRef, ContentDigest, DigestAlgorithm, DigestHex, EntityRef, Extensions,
    MediaType, OperationId, SafeMessage, TenantRef, WireTimestamp, canonical_json,
    dropped_field_pointers,
};
use ratatoskr_operation_contracts::{
    OperationKind, OperationProgressed, OperationResultKind, OperationResultRef, OperationSnapshot,
    OperationStage, OperationStatus, ProgressPercent,
};

/// A legal envelope with an empty payload, used as the carrier in the composition test below.
const MINIMAL_ENVELOPE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000031",
  "event_type": "platform.operation.progressed.v1",
  "occurred_at": "2026-08-17T10:00:02Z",
  "producer": "ratatoskr-platform",
  "aggregate_id": "operation:018f0000-0000-7000-8000-000000000121",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000121",
  "schema_version": 1,
  "payload": {}
}
"#;

fn instant(raw: &str) -> WireTimestamp {
    WireTimestamp::parse(raw).expect("a canonical instant")
}

fn code(raw: &str) -> ErrorCode {
    ErrorCode::parse(raw).expect("a legal code")
}

fn message(raw: &str) -> SafeMessage {
    SafeMessage::parse(raw).expect("a legal message")
}

fn blob_ref() -> BlobRef {
    BlobRef {
        owner_service: BlobOwner::parse("ratatoskr-extractor").expect("a legal blob owner"),
        digest: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            hex: DigestHex::parse(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .expect("a legal digest"),
        },
        media_type: MediaType::parse("application/json").expect("a legal media type"),
        length_bytes: 128,
    }
}

/// A snapshot in which **every** member is present and non-default, including the preserved
/// extension maps at every nesting level.
///
/// This is the drift guard for the one hand-written `Deserialize` in the repository: a field added
/// to `OperationSnapshot` and not to its private wire mirror is dropped on parse, and the
/// round-trip assertion below fails the moment that happens.
fn snapshot_carrying_every_field() -> OperationSnapshot {
    let mut extensions = Extensions::new();
    extensions.insert("queue_position", serde_json::json!(3));

    let mut result_extensions = Extensions::new();
    result_extensions.insert("page_count", serde_json::json!(12));

    OperationSnapshot {
        operation_id: OperationId::parse("018f0000-0000-7000-8000-000000000010")
            .expect("a canonical UUID"),
        kind: OperationKind::parse("content.document.extract").expect("a legal kind"),
        status: OperationStatus::PartiallySucceeded,
        stage: Some(OperationStage::parse("normalizing").expect("a legal stage")),
        progress_percent: Some(ProgressPercent::new(87).expect("a legal estimate")),
        results: vec![OperationResultRef {
            result_kind: OperationResultKind::parse("content.document")
                .expect("a legal result kind"),
            target: EntityRef::parse("document:018f0000-0000-7000-8000-000000000021")
                .expect("a legal reference"),
            blob: Some(blob_ref()),
            extensions: result_extensions,
        }],
        errors: vec![ErrorEnvelope {
            code: code("content.extraction.unreadable"),
            message: message("Page 3 could not be read."),
            retryable: true,
            field_violations: vec![FieldViolation {
                field_path: FieldPath::parse("/blocks/3/text").expect("a legal path"),
                code: code("content.extraction.empty_block"),
                message: message("This block carries no text."),
                extensions: Extensions::new(),
            }],
            correlation_id: Some(
                EntityRef::parse("operation:018f0000-0000-7000-8000-000000000010")
                    .expect("a legal reference"),
            ),
            trace_id: Some(TraceId::parse("4bf92f3577b34da6a3ce929d0e0e4736").expect("a legal id")),
            extensions: Extensions::new(),
        }],
        warnings: vec![WarningEnvelope {
            code: code("content.extraction.truncated"),
            message: message("One page was cut short."),
            field_path: Some(FieldPath::parse("/blocks/9").expect("a legal path")),
            extensions: Extensions::new(),
        }],
        retryable: true,
        correlation_id: EntityRef::parse("operation:018f0000-0000-7000-8000-000000000010")
            .expect("a legal reference"),
        tenant_id: Some(
            TenantRef::parse("user:018f0000-0000-7000-8000-000000000005").expect("a legal owner"),
        ),
        accepted_at: instant("2026-08-17T10:00:00Z"),
        status_changed_at: instant("2026-08-17T10:05:00Z"),
        terminated_at: Some(instant("2026-08-17T10:05:00Z")),
        extensions,
    }
}

/// O-2. The drift guard for the hand-written `Deserialize`.
#[test]
fn roundtrips_a_snapshot_carrying_every_field() {
    let snapshot = snapshot_carrying_every_field();
    assert_eq!(snapshot.validate(), Ok(()));

    let rendered = canonical_json(&snapshot).expect("canonical rendering");
    assert!(rendered.ends_with("}\n"));

    let decoded: OperationSnapshot = serde_json::from_str(&rendered).expect("round trip");
    assert_eq!(
        decoded, snapshot,
        "a field present on OperationSnapshot but missing from its private wire mirror is \
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
        "operation_id",
        "kind",
        "status",
        "stage",
        "progress_percent",
        "results",
        "errors",
        "warnings",
        "retryable",
        "correlation_id",
        "tenant_id",
        "accepted_at",
        "status_changed_at",
        "terminated_at",
        "queue_position",
    ] {
        assert!(
            rendered.contains(member),
            "{member} is missing from {rendered}"
        );
    }
}

/// O-4. Absence of a bounded numeric means "the producer cannot estimate", never zero.
#[test]
fn absent_progress_percent_is_not_zero() {
    let mut snapshot = snapshot_carrying_every_field();
    snapshot.progress_percent = None;

    let rendered = canonical_json(&snapshot).expect("canonical rendering");
    assert!(
        !rendered.contains("progress_percent"),
        "None serializes as absent, never as 0 and never as null"
    );

    let decoded: OperationSnapshot = serde_json::from_str(&rendered).expect("round trip");
    assert_eq!(decoded.progress_percent, None);
    assert_ne!(
        decoded.progress_percent,
        Some(ProgressPercent::new(0).expect("zero is a legal estimate"))
    );

    // An explicit null is the same as absence (`DOMAIN.md` invariant 4, spec D12).
    let with_null = rendered.replace(
        "\"stage\": \"normalizing\",",
        "\"stage\": \"normalizing\",\n  \"progress_percent\": null,",
    );
    let from_null: OperationSnapshot =
        serde_json::from_str(&with_null).expect("an explicit null parses");
    assert_eq!(from_null, decoded);

    // Zero is a real estimate and survives the round trip as one.
    let mut zero = snapshot;
    zero.progress_percent = Some(ProgressPercent::new(0).expect("zero is a legal estimate"));
    let rendered_zero = canonical_json(&zero).expect("canonical rendering");
    assert!(rendered_zero.contains("\"progress_percent\": 0"));
    assert_eq!(
        serde_json::from_str::<OperationSnapshot>(&rendered_zero)
            .expect("round trip")
            .progress_percent,
        zero.progress_percent
    );
}

/// The payload is state-carried transfer: the whole snapshot travels inside the event, and the
/// event type constant parses, so `EventPayload::event_type()` cannot panic in a shipped build.
#[test]
fn operation_progressed_carries_the_whole_snapshot() {
    assert_eq!(
        OperationProgressed::EVENT_TYPE,
        "platform.operation.progressed.v1"
    );
    let event_type =
        EventType::parse(OperationProgressed::EVENT_TYPE).expect("the constant is a legal name");
    assert_eq!(OperationProgressed::event_type(), event_type);
    assert_eq!(event_type.major(), 1);
    assert_eq!(event_type.family(), "platform.operation.progressed");

    let payload = OperationProgressed {
        operation: snapshot_carrying_every_field(),
        extensions: Extensions::new(),
    };
    let rendered = canonical_json(&payload).expect("canonical rendering");
    let decoded: OperationProgressed = serde_json::from_str(&rendered).expect("round trip");
    assert_eq!(decoded, payload);
}

/// The composition spec §7.3 calls the end-to-end proof: the **real** payload inside a **real**
/// envelope, not a stand-in.
///
/// E-6 proves `set_payload`/`payload_as` are inverse using a one-field probe declared inside
/// `ratatoskr-event-envelope`'s own test, because that crate must not depend on this one. This is
/// the other half: this crate does depend on the envelope, so the two contracts can be composed
/// here and nowhere else.
#[test]
fn the_real_payload_travels_inside_a_real_envelope() {
    let payload = OperationProgressed {
        operation: snapshot_carrying_every_field(),
        extensions: Extensions::new(),
    };

    let mut envelope =
        EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes()).expect("the minimal envelope parses");
    envelope
        .set_payload(&payload)
        .expect("the payload serializes to a JSON object");
    assert_eq!(
        envelope.event_type,
        OperationProgressed::event_type(),
        "set_payload binds event_type to the payload type"
    );

    let wire = envelope
        .to_canonical_json()
        .expect("the envelope re-serializes");
    let reparsed = EventEnvelope::from_json(wire.as_bytes()).expect("the envelope round trips");
    assert_eq!(reparsed, envelope);
    assert_eq!(
        reparsed
            .payload_as::<OperationProgressed>()
            .expect("the payload comes back typed"),
        payload
    );

    let sent: serde_json::Value = serde_json::from_str(&wire).expect("the wire form is JSON");
    let received: serde_json::Value =
        serde_json::to_value(&reparsed).expect("the envelope re-serializes as a value");
    assert_eq!(
        dropped_field_pointers(&sent, &received),
        Vec::<String>::new(),
        "the envelope must not discard any member of its own payload"
    );
}
