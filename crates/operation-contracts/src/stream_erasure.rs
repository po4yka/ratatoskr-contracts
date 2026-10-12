//! The command a producer sends to Platform Edge to erase messages it published from the events
//! stream (XR-021 CONTRACTS.md R5-12).
//!
//! # Why it exists
//!
//! `ratatoskr-chatgpt` and `ratatoskr-claude` remove only their own outbox rows when a user deletes content. The
//! conversation, project and artifact facts they already published carry message text and stay
//! in `ratatoskr_events` until the stream drops them, and no producer holds a stream-management
//! permission. Edge is the only identity allowed to erase a single message, so a producer asks
//! it to, naming the stream sequences the broker acknowledged when the producer published them.
//!
//! # What the command can reach
//!
//! Only messages the requesting producer itself published for the owner it names. Edge reads
//! each sequence and erases it only when the message subject is one of the seven content-bearing
//! `evt.ai_archive.*` subjects (never the tombstone), the envelope `producer` equals the
//! command's producer and the envelope `tenant_id` equals [`owner`](
//! PlatformStreamErasureRequested::owner). Anything else is counted as refused and left alone.
//! This payload carries sequence numbers and a request reference, never content.

use ratatoskr_event_envelope::CommandPayload;
use ratatoskr_identifiers::{EntityRef, Extensions, TenantRef};

use crate::error::OperationContractError;

/// The most sequences one erasure command may name.
pub const MAX_STREAM_SEQUENCES: usize = 500;

/// The largest sequence a JSON consumer reads exactly: 2^53 - 1.
pub const MAX_JSON_SAFE_SEQUENCE: u64 = 9_007_199_254_740_991;

/// Payload of `platform.stream_erasure.requested.v1`: a producer asks Edge to erase specific
/// messages it published from the events stream.
///
/// The enclosing command envelope carries the requesting `producer` (`ratatoskr-chatgpt` or
/// `ratatoskr-claude`), `command_id` (the outbox id), `aggregate_id` equal to
/// [`request_ref`](Self::request_ref), the deletion request's `correlation_id` and `tenant_id`
/// equal to [`owner`](Self::owner). Delivery is at least once and Edge deduplicates, so a replay
/// erases nothing twice.
///
/// [`validate`](Self::validate) runs on every deserialization.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct PlatformStreamErasureRequested {
    /// The deletion request this erasure belongs to. Its kind is `privacy_request`.
    pub request_ref: EntityRef,

    /// The owner whose published messages are erased.
    pub owner: TenantRef,

    /// Stream sequence numbers of the messages to erase, as returned by the broker when the
    /// producer published them: 1 to 500 entries, each at least 1 and at most 2^53 - 1, strictly
    /// ascending.
    #[schemars(
        length(min = 1, max = 500),
        inner(range(min = 1, max = 9_007_199_254_740_991_u64))
    )]
    pub stream_sequences: Vec<u64>,

    /// Unknown-but-preserved additive payload members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl PlatformStreamErasureRequested {
    /// Checks the request reference kind and the sequence list.
    ///
    /// # Errors
    ///
    /// [`OperationContractError::StreamErasureRequestKind`] for a `request_ref` that is not a
    /// `privacy_request`, [`OperationContractError::EmptyStreamSequences`],
    /// [`OperationContractError::TooManyStreamSequences`],
    /// [`OperationContractError::ZeroStreamSequence`],
    /// [`OperationContractError::StreamSequenceAboveJsonSafeRange`] and
    /// [`OperationContractError::StreamSequencesNotAscending`] for a list that breaks the rules
    /// above.
    pub fn validate(&self) -> Result<(), OperationContractError> {
        let kind = self.request_ref.kind();
        if kind.as_str() != "privacy_request" {
            return Err(OperationContractError::StreamErasureRequestKind {
                actual: kind.as_str().to_owned(),
            });
        }
        let sequences = &self.stream_sequences;
        if sequences.is_empty() {
            return Err(OperationContractError::EmptyStreamSequences);
        }
        if sequences.len() > MAX_STREAM_SEQUENCES {
            return Err(OperationContractError::TooManyStreamSequences {
                max: MAX_STREAM_SEQUENCES,
                got: sequences.len(),
            });
        }
        for (index, &got) in sequences.iter().enumerate() {
            if got == 0 {
                return Err(OperationContractError::ZeroStreamSequence { index });
            }
            if got > MAX_JSON_SAFE_SEQUENCE {
                return Err(OperationContractError::StreamSequenceAboveJsonSafeRange {
                    index,
                    got,
                });
            }
        }
        for (index, pair) in sequences.windows(2).enumerate() {
            if let [previous, next] = *pair
                && next <= previous
            {
                return Err(OperationContractError::StreamSequencesNotAscending {
                    index: index + 1,
                    previous,
                    next,
                });
            }
        }
        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct Wire {
    request_ref: EntityRef,
    owner: TenantRef,
    stream_sequences: Vec<u64>,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for PlatformStreamErasureRequested {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = Wire::deserialize(deserializer)?;
        let payload = Self {
            request_ref: wire.request_ref,
            owner: wire.owner,
            stream_sequences: wire.stream_sequences,
            extensions: wire.extensions,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}

impl CommandPayload for PlatformStreamErasureRequested {
    const COMMAND_TYPE: &'static str = "platform.stream_erasure.requested.v1";
}
