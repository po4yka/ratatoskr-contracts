//! The backup-policy acknowledgment event payload and its outcome vocabulary.

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{EntityRef, Extensions};

use crate::error::BackupContractError;

/// Payload of `vault.backup_policy.acknowledged.v1`: Vault answered a desired-backup-policy
/// version.
///
/// A fact, not a request (`AGENTS.md` principle 9): the decision was reached before this event
/// existed. One acknowledgment answers exactly one policy version; the aggregate identifier of
/// the carrying envelope names that version as `backup_policy:<version>`.
///
/// `Deserialize` is hand-written because the acknowledgment's self-consistency rules are
/// cross-field: outcome versus reasons, reason code versus repository reference, and acceptance
/// implying forward progress. It parses a private mirror struct and then checks. A field added
/// to the public struct and not to the mirror would be silently dropped; the envelope
/// composition test round-trips a payload carrying every field and fails the moment that
/// happens.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct PolicyAcknowledged {
    /// The policy version this acknowledgment answers. Required; greater than zero.
    pub acknowledged_policy_version: u64,

    /// The decision Vault reached on that version. Required; see [`PolicyOutcome`].
    pub outcome: PolicyOutcome,

    /// Why a rejected version was refused. Present exactly when `outcome` is `rejected`; every
    /// rejection explains itself.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<PolicyRejectionReason>,

    /// The last policy version Vault had fully applied before this decision. Required. A
    /// rejected acknowledgment may legitimately name a version far below it — that is what a
    /// stale-version refusal looks like.
    pub last_applied_policy_version: u64,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for PolicyAcknowledged {
    const EVENT_TYPE: &'static str = "vault.backup_policy.acknowledged.v1";
}

impl PolicyAcknowledged {
    /// Re-checks every self-consistency invariant. `Deserialize` calls this; a producer that
    /// mutates an acknowledgment in place calls it again before emitting.
    ///
    /// # Errors
    ///
    /// [`BackupContractError::AcknowledgedVersionZero`],
    /// [`BackupContractError::RejectedWithoutReason`],
    /// [`BackupContractError::AcceptedWithReason`],
    /// [`BackupContractError::ReasonReferenceForbidden`],
    /// [`BackupContractError::ReasonReferenceRequired`],
    /// [`BackupContractError::AcceptedWithoutProgress`].
    pub fn validate(&self) -> Result<(), BackupContractError> {
        if self.acknowledged_policy_version == 0 {
            return Err(BackupContractError::AcknowledgedVersionZero);
        }

        if self.outcome == PolicyOutcome::Accepted {
            if !self.reasons.is_empty() {
                return Err(BackupContractError::AcceptedWithReason);
            }
            if self.acknowledged_policy_version <= self.last_applied_policy_version {
                return Err(BackupContractError::AcceptedWithoutProgress {
                    acknowledged: self.acknowledged_policy_version,
                    last_applied: self.last_applied_policy_version,
                });
            }
        } else if self.reasons.is_empty() {
            return Err(BackupContractError::RejectedWithoutReason);
        }

        for reason in &self.reasons {
            let code = reason.code.as_str();
            let demands_reference = reason.code == PolicyRejectionCode::RepositoryUnknownInCatalog;
            if demands_reference && reason.repository_ref.is_none() {
                return Err(BackupContractError::ReasonReferenceRequired {
                    code: code.to_owned(),
                });
            }
            if !demands_reference && reason.repository_ref.is_some() {
                return Err(BackupContractError::ReasonReferenceForbidden {
                    code: code.to_owned(),
                });
            }
        }

        Ok(())
    }
}

/// The wire mirror of [`PolicyAcknowledged`], parsed before the cross-field invariants run.
///
/// Serde offers no post-deserialization validation hook, so the public type's `Deserialize` is
/// hand-written and delegates the field-by-field work here.
///
/// **Drift guard:** a field added to [`PolicyAcknowledged`] and not to this mirror is silently
/// dropped on parse; the envelope composition test fails the moment that happens. Do not add a
/// field to one of these structs alone.
#[derive(Debug, serde::Deserialize)]
struct PolicyAcknowledgedWire {
    acknowledged_policy_version: u64,
    outcome: PolicyOutcome,
    #[serde(default)]
    reasons: Vec<PolicyRejectionReason>,
    last_applied_policy_version: u64,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for PolicyAcknowledged {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = PolicyAcknowledgedWire::deserialize(deserializer)?;
        let payload = Self {
            acknowledged_policy_version: wire.acknowledged_policy_version,
            outcome: wire.outcome,
            reasons: wire.reasons,
            last_applied_policy_version: wire.last_applied_policy_version,
            extensions: wire.extensions,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}

/// The decision Vault reached on one policy version.
///
/// **Closed on purpose**: consumers branch on the outcome to prune or keep state, and an
/// unrecognized outcome must stop processing instead of being read as either branch.
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
pub enum PolicyOutcome {
    /// Vault applied the version; its `last_applied_policy_version` moved to it.
    Accepted,
    /// Vault refused the version; `reasons` say why.
    Rejected,
}

/// Why a policy version was rejected.
///
/// **Closed on purpose**: GitHub branches on the code to fix and republish, and an unrecognized
/// code must stop processing instead of being filed under a default. Adding a variant is an
/// additive, non-breaking expansion.
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
pub enum PolicyRejectionCode {
    /// The candidate version did not strictly exceed the last applied one, so applying it
    /// would replay or rewind state.
    PolicyVersionNotMonotonic,
    /// An entry named a repository absent from Vault's catalog; the reason carries that
    /// repository's reference.
    RepositoryUnknownInCatalog,
}

impl PolicyRejectionCode {
    /// The wire token for this code.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PolicyVersionNotMonotonic => "policy_version_not_monotonic",
            Self::RepositoryUnknownInCatalog => "repository_unknown_in_catalog",
        }
    }
}

/// One machine-actionable rejection reason.
///
/// A flat struct rather than a tagged enum, keeping the schema simple and the pairing rule — a
/// repository reference appears exactly when the code demands one — enforceable at parse time.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct PolicyRejectionReason {
    /// Stable code a consumer branches on. Required; see [`PolicyRejectionCode`].
    pub code: PolicyRejectionCode,

    /// The repository the reason concerns. Present exactly when `code` is
    /// `repository_unknown_in_catalog`; omitted for every other code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_ref: Option<EntityRef>,
}
