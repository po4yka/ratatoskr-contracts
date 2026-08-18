//! The operation lifecycle event payload.

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::Extensions;

use crate::snapshot::OperationSnapshot;

/// Payload of `platform.operation.progressed.v1` (`README.md`, `ARCHITECTURE.md` S9.1).
///
/// A fact: the operation **has** reached `operation.status`. It is not a request to change it
/// (`AGENTS.md` principle 9). State-carried transfer — a consumer needs no prior event to
/// interpret this one, which is what makes at-least-once redelivery idempotent.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct OperationProgressed {
    /// Full public snapshot of the operation at the instant the event was produced.
    pub operation: OperationSnapshot,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for OperationProgressed {
    const EVENT_TYPE: &'static str = "platform.operation.progressed.v1";
}
