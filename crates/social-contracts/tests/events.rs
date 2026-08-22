//! The captured/updated event payloads and their envelope composition (spec: events
//! requirement).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

mod common;

use ratatoskr_event_envelope::{EventEnvelope, EventPayload, EventType};
use ratatoskr_identifiers::{Extensions, dropped_field_pointers};
use ratatoskr_social_contracts::{
    CaptureCompleteness, SocialSourceCaptured, SocialSourceSnapshot, SocialSourceUpdated,
};

use common::{MINIMAL_ENVELOPE, snapshot_carrying_every_field};

/// Both payloads are registered facts with past-tense actions at major 1.
#[test]
fn event_type_constants_are_the_registered_names() {
    assert_eq!(
        SocialSourceCaptured::EVENT_TYPE,
        "social.source.captured.v1"
    );
    assert_eq!(SocialSourceUpdated::EVENT_TYPE, "social.source.updated.v1");

    let captured = EventType::parse(SocialSourceCaptured::EVENT_TYPE).expect("parses");
    assert_eq!(captured.bounded_context(), "social");
    assert_eq!(captured.aggregate(), "source");
    assert_eq!(captured.action(), "captured");
    assert_eq!(captured.major(), 1);
    assert!(captured.action_looks_past_tense());

    let updated = EventType::parse(SocialSourceUpdated::EVENT_TYPE).expect("parses");
    assert_eq!(updated.action(), "updated");
    assert_eq!(updated.major(), 1);
    assert!(updated.action_looks_past_tense());
}

/// A captured payload travels inside a real envelope and comes back typed and whole.
#[test]
fn captured_payload_travels_inside_a_real_envelope() {
    let payload = SocialSourceCaptured {
        source: snapshot(),
        extensions: Extensions::new(),
    };

    let mut envelope =
        EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes()).expect("the minimal envelope parses");
    envelope
        .set_payload(&payload)
        .expect("the payload serializes to a JSON object");
    assert_eq!(
        envelope.event_type,
        SocialSourceCaptured::event_type(),
        "set_payload binds event_type to the payload type"
    );

    let wire = envelope
        .to_canonical_json()
        .expect("the envelope re-serializes");
    let reparsed = EventEnvelope::from_json(wire.as_bytes()).expect("the envelope round trips");
    assert_eq!(reparsed, envelope);

    let sent: serde_json::Value = serde_json::from_str(&wire).expect("the wire form is JSON");
    let received: serde_json::Value = serde_json::to_value(&reparsed).expect("re-serialize");
    assert_eq!(
        dropped_field_pointers(&sent, &received),
        Vec::<String>::new(),
        "the envelope must not discard any member of its own payload"
    );

    let decoded = reparsed
        .payload_as::<SocialSourceCaptured>()
        .expect("the payload comes back typed");
    assert_eq!(decoded, payload);
}

/// An updated event re-publishes the full record: the payload alone indexes the source.
#[test]
fn updated_payload_carries_the_whole_snapshot() {
    let payload = SocialSourceUpdated {
        source: snapshot(),
        extensions: Extensions::new(),
    };

    let mut envelope =
        EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes()).expect("the minimal envelope parses");
    envelope
        .set_payload(&payload)
        .expect("the payload serializes to a JSON object");

    let decoded = envelope
        .payload_as::<SocialSourceUpdated>()
        .expect("typed read straight off the parsed envelope");
    assert_eq!(decoded.source.social_source_id, snapshot().social_source_id);
    assert!(decoded.source.text.is_some());
    assert_eq!(decoded.source.media.len(), 2);
}

/// A consumer that asks for the wrong social payload from an envelope is refused, not given a
/// half-interpreted body.
#[test]
fn mismatched_social_payload_type_is_refused() {
    let payload = SocialSourceCaptured {
        source: snapshot(),
        extensions: Extensions::new(),
    };
    let mut envelope =
        EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes()).expect("the envelope parses");
    envelope.set_payload(&payload).expect("a JSON object body");

    let error = envelope
        .payload_as::<SocialSourceUpdated>()
        .expect_err("a captured event is not an updated event");
    assert!(
        error.to_string().contains("social.source.updated.v1"),
        "unexpected error: {error}"
    );
}

/// A consumer that asks for a social payload from an unrelated envelope is refused.
#[test]
fn unrelated_envelope_is_refused() {
    let envelope_body = MINIMAL_ENVELOPE.replace(
        "\"event_type\": \"social.source.captured.v1\"",
        "\"event_type\": \"platform.operation.progressed.v1\"",
    );
    let envelope = EventEnvelope::from_json(envelope_body.as_bytes()).expect("parses");

    let error = envelope
        .payload_as::<SocialSourceCaptured>()
        .expect_err("a progressed operation is not a captured source");
    assert!(
        error
            .to_string()
            .contains("platform.operation.progressed.v1"),
        "unexpected error: {error}"
    );
}

/// The richest legal body shape serves both payloads; completeness stays complete with its one
/// non-blocking warning.
fn snapshot() -> SocialSourceSnapshot {
    let mut snapshot = snapshot_carrying_every_field();
    snapshot.completeness = CaptureCompleteness::Complete;
    snapshot
}
