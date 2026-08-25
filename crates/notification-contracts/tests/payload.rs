//! Payload-level invariants of the raised notification: registry versioning and per-class round
//! trips.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

mod common;

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::canonical_json;
use ratatoskr_notification_contracts::{
    NotificationClass, NotificationPriority, NotificationRaised, QuietHoursHint,
};

use common::payload_carrying_every_field;

/// Every self-contradiction the payload grammar forbids is refused at parse time; each case is
/// pinned on the wire by an invalid fixture whose `invalid-expectations.toml` row names the same
/// rejection.
#[test]
fn self_contradicting_payloads_are_refused() {
    let zero_registry = r#"{
  "notification_id": "018f0000-0000-7000-8000-000000000711",
  "class_registry_version": 0,
  "class": "operation_completed",
  "recipient": "user:018f0000-0000-7000-8000-000000000005",
  "title": "No producer speaks vocabulary zero"
}"#;

    let error = serde_json::from_str::<NotificationRaised>(zero_registry).expect_err("refused");
    assert!(
        error
            .to_string()
            .contains("class_registry_version must be greater than zero"),
        "failed for the wrong reason: {error}"
    );
}

/// The payload is registered under exactly one event name at major 1.
#[test]
fn event_type_is_the_registered_name() {
    assert_eq!(
        NotificationRaised::EVENT_TYPE,
        "platform.notification.raised.v1"
    );
}

/// One committed-style payload per known class: each serializes to canonical JSON, reparses
/// equal, and drops nothing. The gate's fixture step proves this for the files under
/// `fixtures/events/platform.notification.raised.v1/valid/`; this test proves it for the typed
/// constructors a producer actually calls.
#[test]
fn every_class_round_trips_losslessly() {
    let classes = [
        NotificationClass::OperationCompleted,
        NotificationClass::OperationFailed,
        NotificationClass::AnalysisReady,
        NotificationClass::BackupOutcome,
        NotificationClass::WatchTriggered,
        NotificationClass::ArchiveImported,
    ];

    for class in classes {
        let payload = payload_carrying_every_field(
            class.clone(),
            Some(NotificationPriority::Normal),
            Some(QuietHoursHint::new(82_800, 25_200).expect("overnight window")),
        );

        let rendered = canonical_json(&payload).expect("the payload serializes canonically");
        let reparsed: NotificationRaised =
            serde_json::from_str(&rendered).expect("the canonical form reparses");
        assert_eq!(
            reparsed, payload,
            "class `{class}` must survive its own wire form"
        );

        let sent: serde_json::Value =
            serde_json::from_str(&rendered).expect("the wire form is JSON");
        let received: serde_json::Value = serde_json::to_value(&reparsed).expect("re-serialize");
        assert_eq!(
            sent, received,
            "class `{class}` must not gain or lose members"
        );
    }
}

/// An unknown class token travels through the typed constructor path just as it does on the wire:
/// preserved verbatim across a full canonical round trip.
#[test]
fn unknown_class_round_trips_through_the_payload() {
    let class = NotificationClass::parse("carrier_pigeon").expect("well-formed token");
    assert!(!class.is_known());

    let payload = payload_carrying_every_field(class, None, None);
    let rendered = canonical_json(&payload).expect("serializes");
    let reparsed: NotificationRaised = serde_json::from_str(&rendered).expect("reparses");
    assert_eq!(reparsed.class.as_str(), "carrier_pigeon");
    assert!(!reparsed.class.is_known());
}
