//! Account-erasure command and acknowledgement payloads.

use ratatoskr_event_envelope::{CommandPayload, EventPayload};
use ratatoskr_identifiers::{Extensions, OperationId};

/// Platform's request for one owner to erase its Ratatoskr-held account data.
///
/// The enclosing command envelope carries the owner scope, correlation, and
/// delivery identity. This payload carries only the durable operation identity
/// an owner needs to make at-least-once delivery safe.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AccountErasureRequested {
    /// Durable Platform operation that coordinates this account erasure.
    pub operation_id: OperationId,

    /// Unknown-but-preserved additive payload members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl CommandPayload for AccountErasureRequested {
    const COMMAND_TYPE: &'static str = "identity.account.erasure_requested.v1";
}

/// The terminal erasure fact an owner reports to Platform.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AccountErasureOutcome {
    /// The owner verified deletion of all Ratatoskr-held data in its boundary.
    Verified,
    /// Local data was deleted, but the external provider grant could not be revoked.
    IncompleteExternalGrantRevocation,
}

/// An owner's terminal report for one account-erasure operation.
///
/// The enclosing event envelope carries the owner scope and at-least-once event
/// identity. Platform aggregates these reports and alone determines whether the
/// whole account erasure is complete.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AccountErasureAcknowledged {
    /// Durable Platform operation this terminal report belongs to.
    pub operation_id: OperationId,

    /// Whether local deletion was verified or external grant revocation remained incomplete.
    pub outcome: AccountErasureOutcome,

    /// Unknown-but-preserved additive payload members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for AccountErasureAcknowledged {
    const EVENT_TYPE: &'static str = "identity.account.erasure_acknowledged.v1";
}
