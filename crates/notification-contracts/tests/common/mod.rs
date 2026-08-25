//! Shared envelope and payload builders for the notification-contracts integration tests.

#![allow(
    dead_code,
    reason = "each test binary consumes a different subset of this module"
)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::{
    EntityKind, EntityLocalId, EntityRef, Extensions, NotificationId, SafeMessage, TenantRef,
    UserId,
};
use ratatoskr_notification_contracts::{
    NotificationClass, NotificationPriority, NotificationRaised, QuietHoursHint,
};

/// A legal envelope fragment carrying an empty payload, used as the carrier in composition
/// tests. The aggregate names the raised notification itself; `set_payload` replaces
/// `event_type` and body.
pub(crate) const MINIMAL_ENVELOPE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000602",
  "event_type": "platform.notification.raised.v1",
  "occurred_at": "2026-08-25T12:00:00Z",
  "producer": "ratatoskr-knowledge",
  "aggregate_id": "notification:018f0000-0000-7000-8000-000000000708",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000302",
  "schema_version": 1,
  "payload": {}
}
"#;

/// A valid recipient shared by every builder in this module.
pub(crate) fn recipient() -> TenantRef {
    TenantRef::of_user(UserId(
        "018f0000-0000-7000-8000-000000000005"
            .parse()
            .expect("uuid"),
    ))
}

/// An operation reference in the open pointer grammar.
pub(crate) fn operation_ref() -> EntityRef {
    let local =
        EntityLocalId::parse("018f0000-0000-7000-8000-000000000302").expect("canonical uuid");
    EntityRef::new(EntityKind::Operation, local)
}

/// An analysis reference in the open pointer grammar.
pub(crate) fn analysis_ref() -> EntityRef {
    let local =
        EntityLocalId::parse("018f0000-0000-7000-8000-000000000401").expect("canonical uuid");
    EntityRef::new(EntityKind::Other("analysis".to_owned()), local)
}

/// Builds a payload carrying **every** member - hints, both correlation references and preserved
/// extensions included.
///
/// This is the drift guard partner of the private wire mirror: a field added to
/// [`NotificationRaised`] and not to its mirror is dropped on parse, and the composition test
/// fails the moment that happens. Do not add a field to one of these alone.
#[allow(
    clippy::too_many_lines,
    reason = "one member per line is the point of an every-field fixture"
)]
pub(crate) fn payload_carrying_every_field(
    class: NotificationClass,
    priority_hint: Option<NotificationPriority>,
    quiet_hours: Option<QuietHoursHint>,
) -> NotificationRaised {
    let mut extensions = Extensions::new();
    extensions.insert("raised_by_rule", serde_json::json!("nightly-backup-report"));

    NotificationRaised {
        notification_id: NotificationId(
            "018f0000-0000-7000-8000-000000000708"
                .parse()
                .expect("uuid"),
        ),
        class_registry_version: 1,
        class,
        recipient: recipient(),
        title: SafeMessage::parse("Analysis ready").expect("carrier-safe"),
        message: Some(
            SafeMessage::parse("Every member of the notification grammar appears here.")
                .expect("carrier-safe"),
        ),
        operation_ref: Some(operation_ref()),
        analysis_ref: Some(analysis_ref()),
        priority_hint,
        quiet_hours,
        extensions,
    }
}

/// Builds a payload exactly as a producer would: no extension keys authored.
///
/// ADR-0008: the extension channel is for relaying upstream keys, never for authoring them. The
/// testable form is `extensions.is_empty()` on constructed values.
pub(crate) fn producer_constructed_payload(class: NotificationClass) -> NotificationRaised {
    NotificationRaised {
        notification_id: NotificationId(
            "018f0000-0000-7000-8000-000000000709"
                .parse()
                .expect("uuid"),
        ),
        class_registry_version:
            ratatoskr_notification_contracts::NOTIFICATION_CLASS_REGISTRY_VERSION,
        class,
        recipient: recipient(),
        title: SafeMessage::parse("Operation completed").expect("carrier-safe"),
        message: None,
        operation_ref: Some(operation_ref()),
        analysis_ref: None,
        priority_hint: None,
        quiet_hours: None,
        extensions: Extensions::new(),
    }
}
