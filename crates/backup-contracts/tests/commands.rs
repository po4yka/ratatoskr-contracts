//! The Vault policy command payload and its envelope composition (XR-021 CONTRACTS.md S09).

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "assertions in a test binary; the index writes into a value the test decoded"
)]

use ratatoskr_backup_contracts::{MirrorCadence, VaultBackupPolicyApplyRequested};
use ratatoskr_event_envelope::{CommandEnvelope, CommandPayload};
use serde_json::json;

const FULL: &str = include_str!(
    "../../../fixtures/commands/vault.backup_policy.apply_requested.v1/valid/full-policy.json"
);

/// A legal envelope carrying an empty payload, used as the carrier in the composition test: the
/// producer is GitHub, the aggregate and correlation name the policy version, and there is no
/// tenant because the policy is catalog-wide.
const ENVELOPE: &str = r#"{
  "command_id": "018f0000-0000-7000-8000-000000000d01",
  "command_type": "vault.backup_policy.apply_requested.v1",
  "issued_at": "2026-08-20T09:00:00Z",
  "producer": "ratatoskr-github",
  "aggregate_id": "backup_policy:3",
  "correlation_id": "backup_policy:3",
  "schema_version": 1,
  "payload": {}
}
"#;

#[test]
fn apply_requested_round_trips_a_desired_policy() {
    assert_eq!(
        VaultBackupPolicyApplyRequested::COMMAND_TYPE,
        "vault.backup_policy.apply_requested.v1"
    );

    let wire: serde_json::Value = serde_json::from_str(FULL).expect("the fixture is JSON");
    let decoded: VaultBackupPolicyApplyRequested =
        serde_json::from_value(wire.clone()).expect("the fixture decodes");
    assert_eq!(decoded.policy.policy_version, 3);
    assert_eq!(decoded.policy.repositories.len(), 2);
    assert_eq!(
        decoded
            .policy
            .repositories
            .first()
            .map(|entry| entry.mirror_cadence),
        Some(MirrorCadence::Daily)
    );
    assert_eq!(
        serde_json::to_value(&decoded).expect("serializes"),
        wire,
        "the command is reproduced exactly"
    );

    // The command travels inside a command envelope and comes back typed and whole.
    let mut envelope = CommandEnvelope::from_json(ENVELOPE.as_bytes()).expect("a legal envelope");
    envelope.set_payload(&decoded).expect("an object payload");
    assert_eq!(
        envelope.command_type,
        VaultBackupPolicyApplyRequested::command_type()
    );
    let wire = envelope.to_canonical_json().expect("re-serializes");
    let reparsed = CommandEnvelope::from_json(wire.as_bytes()).expect("round trips");
    assert_eq!(reparsed, envelope);
    assert_eq!(
        reparsed
            .payload_as::<VaultBackupPolicyApplyRequested>()
            .expect("typed read"),
        decoded
    );
}

#[test]
fn apply_requested_refuses_an_empty_or_inconsistent_payload() {
    let error = serde_json::from_value::<VaultBackupPolicyApplyRequested>(json!({}))
        .expect_err("a command without a policy is not a command");
    assert!(
        error.to_string().contains("missing field `policy`"),
        "{error}"
    );

    let mut zero: serde_json::Value = serde_json::from_str(FULL).expect("the fixture is JSON");
    zero["policy"]["policy_version"] = json!(0);
    let error = serde_json::from_value::<VaultBackupPolicyApplyRequested>(zero)
        .expect_err("the embedded policy keeps its own invariants");
    assert!(
        error
            .to_string()
            .contains("policy_version must be greater than zero"),
        "{error}"
    );
}
