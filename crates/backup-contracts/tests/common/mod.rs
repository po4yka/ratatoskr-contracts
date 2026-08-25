//! Shared envelope and payload builders for the backup-contracts integration tests.

#![allow(
    dead_code,
    reason = "each test binary consumes a different subset of this module"
)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_backup_contracts::{PolicyAcknowledged, PolicyOutcome};
use ratatoskr_identifiers::Extensions;

/// A legal envelope fragment carrying an empty payload, used as the carrier in composition
/// tests. The aggregate names the policy itself as `backup_policy:<version>`; `set_payload`
/// replaces `event_type` and body.
pub(crate) const MINIMAL_ENVELOPE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000601",
  "event_type": "vault.backup_policy.acknowledged.v1",
  "occurred_at": "2026-08-20T09:05:00Z",
  "producer": "ratatoskr-vault",
  "aggregate_id": "backup_policy:3",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000302",
  "schema_version": 1,
  "payload": {}
}
"#;

/// An acknowledgment in which **every** member is present and non-default, including preserved
/// extensions at the top level.
///
/// This is the drift guard partner of the private wire mirror: a field added to
/// [`PolicyAcknowledged`] and not to its mirror is dropped on parse, and the composition test
/// fails the moment that happens. Do not add a field to one of these alone.
pub(crate) fn acknowledgment_carrying_every_field() -> PolicyAcknowledged {
    let mut extensions = Extensions::new();
    extensions.insert(
        "catalog_snapshot_taken_at",
        serde_json::json!("2026-08-20T08:00:00Z"),
    );

    PolicyAcknowledged {
        acknowledged_policy_version: 3,
        outcome: PolicyOutcome::Accepted,
        reasons: Vec::new(),
        last_applied_policy_version: 2,
        extensions,
    }
}
