//! The acknowledgement Knowledge sends when a deletion fact has been applied
//! (XR-021 CONTRACTS.md R5-11).
//!
//! # Why it exists
//!
//! The events stream drops its oldest message at 7 days or 1 GiB, and a producer marks a
//! deletion fact published at the broker acknowledgement. Without a reply a tombstone that
//! Knowledge did not consume in time would be lost silently. Knowledge therefore answers every
//! applied, duplicate or refused `ai_archive.subject.tombstoned.v1` and
//! `social.source.removed.v1` with one `KnowledgeDeletionAcknowledged`, and the producer
//! re-sends the deletion fact until the acknowledgement arrives.
//!
//! # What it names
//!
//! Ids, an owner reference and a subject reference, never content. Each of the five producers
//! (`ratatoskr-chatgpt`, `ratatoskr-claude`, `ratatoskr-x`, `ratatoskr-instagram`, `ratatoskr-threads`) sees every acknowledgement, and an acknowledgement
//! that names no row of a consumer is acknowledged and ignored by it.

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{EntityRef, EventId, Extensions, TenantRef, WireTimestamp};

use crate::error::OperationContractError;

/// Which deletion fact an acknowledgement answers.
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
pub enum DeletionAckFamily {
    /// An `ai_archive.subject.tombstoned.v1` fact from `ratatoskr-chatgpt` or `ratatoskr-claude`.
    AiArchive,
    /// A `social.source.removed.v1` fact from X, Instagram or Threads.
    Social,
}

/// What Knowledge did with the deletion fact.
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
pub enum DeletionAckOutcome {
    /// The rows were deleted and the stored blobs were collected, or the fact repeated one that
    /// had been applied.
    Applied,
    /// Knowledge did not delete anything. `KnowledgeDeletionAcknowledged::refusal` says why.
    Refused,
}

/// Why Knowledge refused to apply a deletion fact.
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
pub enum DeletionAckRefusal {
    /// The subject belongs to another owner than the one the fact names. The data stays in
    /// Knowledge and an operator must look.
    ForeignSubject,
}

/// Payload of `knowledge.deletion.acknowledged.v1`: Knowledge tells a producer that one of its
/// deletion facts was applied or refused.
///
/// The enclosing event envelope is produced by `ratatoskr-knowledge`. Its `event_id` is the
/// outbox id, `aggregate_id` equals `subject`, `correlation_id` is copied from
/// the acknowledged envelope, `causation_id` is `event:<acknowledged_event_id>` and `tenant_id`
/// equals `owner`.
///
/// The acknowledgement names no content: only ids, the owner and a subject reference. A
/// producer matches it to the outbox row whose `event_id` equals
/// `acknowledged_event_id` and whose tenant equals
/// `owner`, and ignores one that matches no row. A duplicate acknowledgement
/// changes nothing.
///
/// `validate()` runs on every deserialization, so a payload that violates the
/// outcome rule never reaches a consumer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct KnowledgeDeletionAcknowledged {
    /// The `event_id` of the deletion fact this acknowledgement answers.
    pub acknowledged_event_id: EventId,

    /// Which deletion fact family was answered.
    pub family: DeletionAckFamily,

    /// The owner of the deleted data.
    pub owner: TenantRef,

    /// The subject the deletion fact named, as an opaque reference.
    pub subject: EntityRef,

    /// Whether the deletion was applied or refused.
    pub outcome: DeletionAckOutcome,

    /// Why the deletion was refused. Present exactly when `outcome` is `refused`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<DeletionAckRefusal>,

    /// Instant Knowledge decided the outcome, according to the Knowledge clock.
    pub acknowledged_at: WireTimestamp,

    /// Unknown-but-preserved additive payload members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl KnowledgeDeletionAcknowledged {
    /// Checks that `refusal` is present exactly when `outcome` is `refused`.
    ///
    /// # Errors
    ///
    /// [`OperationContractError::DeletionAckRefusedWithoutRefusal`] for `refused` with no
    /// `refusal`, [`OperationContractError::DeletionAckAppliedWithRefusal`] for `applied` with
    /// one.
    pub fn validate(&self) -> Result<(), OperationContractError> {
        match (self.outcome, self.refusal) {
            (DeletionAckOutcome::Refused, None) => {
                Err(OperationContractError::DeletionAckRefusedWithoutRefusal)
            }
            (DeletionAckOutcome::Applied, Some(_)) => {
                Err(OperationContractError::DeletionAckAppliedWithRefusal)
            }
            _ => Ok(()),
        }
    }
}

#[derive(serde::Deserialize)]
struct Wire {
    acknowledged_event_id: EventId,
    family: DeletionAckFamily,
    owner: TenantRef,
    subject: EntityRef,
    outcome: DeletionAckOutcome,
    #[serde(default)]
    refusal: Option<DeletionAckRefusal>,
    acknowledged_at: WireTimestamp,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for KnowledgeDeletionAcknowledged {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = Wire::deserialize(deserializer)?;
        let payload = Self {
            acknowledged_event_id: wire.acknowledged_event_id,
            family: wire.family,
            owner: wire.owner,
            subject: wire.subject,
            outcome: wire.outcome,
            refusal: wire.refusal,
            acknowledged_at: wire.acknowledged_at,
            extensions: wire.extensions,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}

impl EventPayload for KnowledgeDeletionAcknowledged {
    const EVENT_TYPE: &'static str = "knowledge.deletion.acknowledged.v1";
}
