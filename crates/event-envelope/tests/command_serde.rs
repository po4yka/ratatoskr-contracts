//! Canonical command-envelope serialization — RED tests for the command contract.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_event_envelope::{CommandEnvelope, CommandError, CommandPayload, CommandType};

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct CaptureProbe {
    permalink: String,
}

impl CommandPayload for CaptureProbe {
    const COMMAND_TYPE: &'static str = "social.capture.requested.v1";
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct OtherProbe;

impl CommandPayload for OtherProbe {
    const COMMAND_TYPE: &'static str = "content.document.extract.v1";
}

const COMMAND: &str = r#"{
  "command_id": "018f0000-0000-7000-8000-000000000001",
  "command_type": "social.capture.requested.v1",
  "issued_at": "2026-08-27T10:00:00Z",
  "producer": "ratatoskr-platform",
  "aggregate_id": "operation:018f0000-0000-7000-8000-000000000002",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000002",
  "tenant_id": "user:018f0000-0000-7000-8000-000000000003",
  "schema_version": 1,
  "payload": {
    "permalink": "https://x.com/ratatoskr/status/1"
  }
}
"#;

#[test]
fn typed_command_round_trips() {
    let command = CommandEnvelope::from_json(COMMAND.as_bytes())
        .expect("a canonical command envelope must parse");

    assert_eq!(
        command.command_type,
        CommandType::parse("social.capture.requested.v1").expect("a valid command type")
    );
    assert_eq!(
        command
            .payload_as::<CaptureProbe>()
            .expect("the typed payload must parse"),
        CaptureProbe {
            permalink: "https://x.com/ratatoskr/status/1".to_owned(),
        }
    );
    assert_eq!(
        command
            .to_canonical_json()
            .expect("command serialization must succeed"),
        COMMAND
    );
}

#[test]
fn typed_command_refuses_a_mismatched_payload() {
    let command = CommandEnvelope::from_json(COMMAND.as_bytes()).expect("the command must parse");

    match command.payload_as::<OtherProbe>() {
        Err(CommandError::PayloadType { expected, found }) => {
            assert_eq!(expected, "content.document.extract.v1");
            assert_eq!(found, "social.capture.requested.v1");
        }
        other => panic!("expected PayloadType, got {other:?}"),
    }
}

#[test]
fn command_preserves_unknown_additive_members() {
    let command = CommandEnvelope::from_json(
        COMMAND
            .replace(
                "  \"payload\": {",
                "  \"dispatch_attempt_count\": 3,\n  \"payload\": {",
            )
            .as_bytes(),
    )
    .expect("an additive command member must parse");

    assert_eq!(
        command.extensions.get("dispatch_attempt_count"),
        Some(&serde_json::json!(3))
    );
}
