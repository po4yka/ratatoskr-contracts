//! The schedule registration command (XR-021 CONTRACTS.md S08).

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "assertions in a test binary; the index writes into a value the test built"
)]

use ratatoskr_event_envelope::{CommandEnvelope, CommandPayload};
use ratatoskr_operation_contracts::PlatformScheduleRegistrationRequested;
use serde_json::json;

const OWNER: &str = "018f0000-0000-7000-8000-000000000005";

/// Platform's own registration example for GitHub (its `crates/scheduling/tests/registration.rs`).
fn github() -> serde_json::Value {
    json!({
        "service_name": "ratatoskr-github",
        "name": "nightly-stars",
        "owner_user_id": OWNER,
        "cron_expression": "0 3 * * *",
        "command_type": "github.sync.requested.v1",
        "operation_kind": "github.sync",
        "payload": { "account": "example_account" },
        "enabled": true
    })
}

/// The channel-digests daily registration of CONTRACTS.md S08.
fn channel_digest() -> serde_json::Value {
    json!({
        "service_name": "ratatoskr-channel-digests",
        "name": "daily-digest",
        "owner_user_id": OWNER,
        "cron_expression": "0 6 * * *",
        "command_type": "channel_digest.schedule.occurrence_requested.v1",
        "operation_kind": "channel_digest.schedule.occurrence",
        "payload": {},
        "enabled": true
    })
}

#[test]
fn registration_payload_round_trips_the_github_and_channel_digest_examples() {
    assert_eq!(
        PlatformScheduleRegistrationRequested::COMMAND_TYPE,
        "platform.schedule.registration_requested.v1"
    );
    for example in [github(), channel_digest()] {
        let decoded: PlatformScheduleRegistrationRequested =
            serde_json::from_value(example.clone()).expect("the example decodes");
        assert_eq!(
            serde_json::to_value(&decoded).expect("serializes"),
            example,
            "the example is reproduced exactly"
        );
    }

    let decoded: PlatformScheduleRegistrationRequested =
        serde_json::from_value(channel_digest()).expect("decodes");
    assert_eq!(decoded.service_name.as_str(), "ratatoskr-channel-digests");
    assert_eq!(decoded.name.as_str(), "daily-digest");
    assert_eq!(decoded.cron_expression.as_str(), "0 6 * * *");
    assert_eq!(
        decoded.command_type.to_wire(),
        "channel_digest.schedule.occurrence_requested.v1"
    );
    assert!(decoded.payload.is_empty() && decoded.enabled);

    // The payload travels inside a command envelope and comes back typed.
    let mut envelope: CommandEnvelope = serde_json::from_value(json!({
        "command_id": "018f0000-0000-7000-8000-000000000b01",
        "command_type": "platform.schedule.registration_requested.v1",
        "issued_at": "2026-08-27T06:00:00Z",
        "producer": "ratatoskr-channel-digests",
        "aggregate_id": "schedule:018f0000-0000-7000-8000-000000000b02",
        "correlation_id": "operation:018f0000-0000-7000-8000-000000000b03",
        "schema_version": 1,
        "payload": {}
    }))
    .expect("the envelope parses");
    envelope.set_payload(&decoded).expect("an object payload");
    let reread: PlatformScheduleRegistrationRequested = envelope.payload_as().expect("typed read");
    assert_eq!(reread, decoded);
}

#[test]
fn registration_rejects_bad_label_and_non_object_payload() {
    let with = |member: &str, value: serde_json::Value| {
        let mut example = github();
        example[member] = value;
        serde_json::from_value::<PlatformScheduleRegistrationRequested>(example)
    };

    for bad_label in [
        "nightly stars",
        "Nightly-Stars",
        "1nightly",
        "",
        "-nightly",
        "nightly.stars",
        &"a".repeat(65),
    ] {
        assert!(
            with("name", json!(bad_label)).is_err(),
            "name {bad_label:?} must be refused"
        );
        assert!(
            with("service_name", json!(bad_label)).is_err(),
            "service_name {bad_label:?} must be refused"
        );
    }
    assert!(
        with("name", json!("a".repeat(64))).is_ok(),
        "64 characters fit"
    );

    for not_an_object in [json!([]), json!("template"), json!(null), json!(7)] {
        assert!(
            with("payload", not_an_object.clone()).is_err(),
            "payload {not_an_object} must be refused"
        );
    }

    for bad_cron in ["bad cron", "* * * *", "0 3 * * * *", ""] {
        assert!(
            with("cron_expression", json!(bad_cron)).is_err(),
            "cron {bad_cron:?} is not five fields"
        );
    }
    assert!(with("operation_kind", json!("Github.Sync")).is_err());
    assert!(with("command_type", json!("github.sync.requested")).is_err());
    assert!(with("owner_user_id", json!("not-a-uuid")).is_err());

    let mut missing = github();
    missing.as_object_mut().unwrap().remove("payload");
    assert!(
        serde_json::from_value::<PlatformScheduleRegistrationRequested>(missing).is_err(),
        "the payload template is required, even when empty"
    );
}
