//! The raised notification and its envelope composition.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

mod common;

use ratatoskr_event_envelope::{EventEnvelope, EventPayload, EventType};
use ratatoskr_identifiers::{canonical_json, dropped_field_pointers};
use ratatoskr_notification_contracts::{
    NotificationClass, NotificationPriority, NotificationRaised, QuietHoursHint,
};

use common::{
    MINIMAL_ENVELOPE, payload_carrying_every_field, producer_constructed_payload, recipient,
};

/// The payload is a registered fact with a past-tense action at major 1.
#[test]
fn event_type_constants_are_the_registered_names() {
    assert_eq!(
        NotificationRaised::EVENT_TYPE,
        "platform.notification.raised.v1"
    );

    let parsed =
        EventType::parse(NotificationRaised::EVENT_TYPE).expect("the registered name parses");
    assert_eq!(parsed.bounded_context(), "platform");
    assert_eq!(parsed.aggregate(), "notification");
    assert_eq!(parsed.action(), "raised");
    assert_eq!(parsed.major(), 1);
    assert!(parsed.action_looks_past_tense());
}

/// A fully populated notification travels inside a real envelope and comes back typed and whole.
#[test]
fn notification_travels_inside_a_real_envelope() {
    let payload = payload_carrying_every_field(
        NotificationClass::AnalysisReady,
        Some(NotificationPriority::High),
        Some(QuietHoursHint::new(82_800, 25_200).expect("overnight window")),
    );

    let mut envelope =
        EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes()).expect("the minimal envelope parses");
    // Tenancy mirrors the recipient: the envelope names no other user.
    envelope.tenant_id = Some(recipient());
    envelope
        .set_payload(&payload)
        .expect("the payload serializes to a JSON object");
    assert_eq!(
        envelope.event_type,
        NotificationRaised::event_type(),
        "set_payload binds event_type to the payload type"
    );
    assert!(
        envelope.aggregate_id.to_wire().starts_with("notification:"),
        "the aggregate names the raised notification itself"
    );
    assert_eq!(
        envelope.tenant_id.as_ref().map(|tenant| tenant.user_id()),
        Some(payload.recipient.user_id()),
        "envelope tenancy must mirror the payload recipient"
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
        .payload_as::<NotificationRaised>()
        .expect("the payload comes back typed");
    assert_eq!(decoded, payload);
}

/// A consumer that asks a typed read from an envelope answering a different fact is refused,
/// not given a half-interpreted body.
#[test]
fn mismatched_payload_read_is_refused() {
    let mut envelope =
        EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes()).expect("the envelope parses");
    envelope
        .set_payload(&payload_carrying_every_field(
            NotificationClass::OperationCompleted,
            None,
            None,
        ))
        .expect("a JSON object body");

    envelope.event_type =
        EventType::parse("platform.operation.progressed.v1").expect("the other fact parses");

    let error = envelope
        .payload_as::<NotificationRaised>()
        .expect_err("an acknowledgment is not a raised notification");
    assert!(
        error
            .to_string()
            .contains("platform.notification.raised.v1"),
        "the refusal must name the expected payload type, got {error}"
    );
}

/// ADR-0008: `extensions` is a preservation channel, never an authoring channel. The testable
/// form is emptiness on the values a producer constructs.
#[test]
fn constructed_payload_authors_nothing_through_extensions() {
    let payload = producer_constructed_payload(NotificationClass::OperationFailed);
    assert!(
        payload.extensions.is_empty(),
        "a producer-constructed payload must author no extension keys"
    );

    let rendered = canonical_json(&payload).expect("serializes canonically");
    let value: serde_json::Value = serde_json::from_str(&rendered).expect("is JSON");
    let object = value.as_object().expect("payload is an object");
    let documented: std::collections::HashSet<&str> = [
        "notification_id",
        "class_registry_version",
        "class",
        "recipient",
        "title",
        "message",
        "operation_ref",
        "analysis_ref",
        "priority_hint",
        "quiet_hours",
    ]
    .into_iter()
    .collect();
    for key in object.keys() {
        assert!(
            documented.contains(key.as_str()),
            "constructed payload serialized undocumented member `{key}`"
        );
    }
}

/// A relay of a later producer's additive field re-emits it verbatim under the preserve policy.
#[test]
fn unknown_additive_members_survive_a_relay() {
    let json = r#"{
  "notification_id": "018f0000-0000-7000-8000-000000000715",
  "class_registry_version": 2,
  "class": "carrier_pigeon",
  "recipient": "user:018f0000-0000-7000-8000-000000000005",
  "title": "From a later producer",
  "delivery_channel_hint": "sms"
}"#;

    let parsed: NotificationRaised = serde_json::from_str(json).expect("preserved on parse");
    let rendered = canonical_json(&parsed).expect("re-serializes");

    let sent: serde_json::Value = serde_json::from_str(json).expect("input is JSON");
    let received: serde_json::Value = serde_json::from_str(&rendered).expect("output is JSON");
    assert_eq!(
        dropped_field_pointers(&sent, &received),
        Vec::<String>::new(),
        "an unknown additive member must survive byte-identically"
    );
    assert_eq!(
        received.get("delivery_channel_hint"),
        Some(&serde_json::json!("sms"))
    );
}
