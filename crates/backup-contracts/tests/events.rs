//! The backup-policy acknowledgment payload and its envelope composition.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

mod common;

use ratatoskr_backup_contracts::{BackupContractError, PolicyAcknowledged};
use ratatoskr_event_envelope::{EventEnvelope, EventPayload, EventType};
use ratatoskr_identifiers::dropped_field_pointers;

use common::{MINIMAL_ENVELOPE, acknowledgment_carrying_every_field};

/// The payload is a registered fact with a past-tense action at major 1.
#[test]
fn event_type_constants_are_the_registered_names() {
    assert_eq!(
        PolicyAcknowledged::EVENT_TYPE,
        "vault.backup_policy.acknowledged.v1"
    );

    let parsed = EventType::parse(PolicyAcknowledged::EVENT_TYPE).expect("parses");
    assert_eq!(parsed.bounded_context(), "vault");
    assert_eq!(parsed.aggregate(), "backup_policy");
    assert_eq!(parsed.action(), "acknowledged");
    assert_eq!(parsed.major(), 1);
    assert!(parsed.action_looks_past_tense());
}

/// An accepted acknowledgment travels inside a real envelope and comes back typed and whole.
#[test]
fn accepted_acknowledgment_travels_inside_a_real_envelope() {
    let payload = acknowledgment_carrying_every_field();

    let mut envelope =
        EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes()).expect("the minimal envelope parses");
    envelope
        .set_payload(&payload)
        .expect("the payload serializes to a JSON object");
    assert_eq!(
        envelope.event_type,
        PolicyAcknowledged::event_type(),
        "set_payload binds event_type to the payload type"
    );
    assert_eq!(envelope.aggregate_id.to_wire(), "backup_policy:3");

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
        .payload_as::<PolicyAcknowledged>()
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
        .set_payload(&acknowledgment_carrying_every_field())
        .expect("a JSON object body");

    envelope.event_type =
        EventType::parse("platform.operation.progressed.v1").expect("the other fact parses");

    let error = envelope
        .payload_as::<PolicyAcknowledged>()
        .expect_err("an acknowledgment is not an operation progression");
    assert!(
        error
            .to_string()
            .contains("vault.backup_policy.acknowledged.v1"),
        "the refusal must name the expected payload type, got {error}"
    );
}

/// Every self-contradiction the acknowledgment grammar forbids is refused at parse time; each
/// case is pinned on the wire by an invalid fixture whose `invalid-expectations.toml` row names
/// the same rejection.
#[test]
fn self_contradicting_acknowledgments_are_refused() {
    let cases: [(&str, String, BackupContractError); 6] = [
        (
            "accepted with reasons",
            accepted_with_reasons(),
            BackupContractError::AcceptedWithReason,
        ),
        (
            "rejected without reasons",
            rejected_without_reasons(),
            BackupContractError::RejectedWithoutReason,
        ),
        (
            "reference on a code that forbids one",
            reference_on_forbidding_code(),
            BackupContractError::ReasonReferenceForbidden {
                code: "policy_version_not_monotonic".to_owned(),
            },
        ),
        (
            "code that requires a reference missing it",
            required_reference_missing(),
            BackupContractError::ReasonReferenceRequired {
                code: "repository_unknown_in_catalog".to_owned(),
            },
        ),
        (
            "zero acknowledged version",
            zero_acknowledged_version(),
            BackupContractError::AcknowledgedVersionZero,
        ),
        (
            "acceptance without forward progress",
            acceptance_without_progress(),
            BackupContractError::AcceptedWithoutProgress {
                acknowledged: 5,
                last_applied: 6,
            },
        ),
    ];

    for (label, json, expected) in cases {
        let error = serde_json::from_str::<PolicyAcknowledged>(&json).expect_err(label);
        assert_eq!(
            error.to_string(),
            expected.to_string(),
            "case `{label}` failed for the wrong reason"
        );
    }
}

fn accepted_with_reasons() -> String {
    r#"{
  "acknowledged_policy_version": 5,
  "outcome": "accepted",
  "reasons": [
    {
      "code": "repository_unknown_in_catalog",
      "repository_ref": "repository:018f0000-0000-7000-8000-000000000504"
    }
  ],
  "last_applied_policy_version": 4
}"#
    .to_owned()
}

fn rejected_without_reasons() -> String {
    r#"{
  "acknowledged_policy_version": 5,
  "outcome": "rejected",
  "last_applied_policy_version": 4
}"#
    .to_owned()
}

fn reference_on_forbidding_code() -> String {
    r#"{
  "acknowledged_policy_version": 5,
  "outcome": "rejected",
  "reasons": [
    {
      "code": "policy_version_not_monotonic",
      "repository_ref": "repository:018f0000-0000-7000-8000-000000000504"
    }
  ],
  "last_applied_policy_version": 9
}"#
    .to_owned()
}

fn required_reference_missing() -> String {
    r#"{
  "acknowledged_policy_version": 5,
  "outcome": "rejected",
  "reasons": [
    {
      "code": "repository_unknown_in_catalog"
    }
  ],
  "last_applied_policy_version": 4
}"#
    .to_owned()
}

fn zero_acknowledged_version() -> String {
    r#"{
  "acknowledged_policy_version": 0,
  "outcome": "rejected",
  "reasons": [
    {
      "code": "policy_version_not_monotonic"
    }
  ],
  "last_applied_policy_version": 7
}"#
    .to_owned()
}

fn acceptance_without_progress() -> String {
    r#"{
  "acknowledged_policy_version": 5,
  "outcome": "accepted",
  "last_applied_policy_version": 6
}"#
    .to_owned()
}
