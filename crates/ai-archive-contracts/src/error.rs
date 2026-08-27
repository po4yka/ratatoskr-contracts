//! [`AiArchiveContractError`]: every way an AI-archive invariant can fail.

/// Every way an AI-archive contract's invariants can fail.
///
/// A *parse* error type: it is never serialized onto the wire — field-level rejection happens
/// in the identifier, token and content-part types, and only the cross-field rules surface
/// here.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AiArchiveContractError {
    /// Invariant A1: an import declared a completeness state other than `complete` but recorded
    /// no gap saying what is missing, which leaves the consumer unable to act on the hole.
    #[error("completeness state that is not `complete` requires at least one gap")]
    IncompleteWithoutGap,
    /// Invariant A2: a report's conversation count disagrees with the conversations actually
    /// carried.
    #[error("conversation_count {declared} does not match the {actual} carried")]
    ConversationCountMismatch {
        /// The count the report declared.
        declared: u32,
        /// The count computable from the payload.
        actual: u32,
    },
    /// Invariant A3: a snapshot's gap count disagrees with its gaps.
    #[error("gap_count {declared} does not match the {actual} carried")]
    GapCountMismatch {
        /// The count the report declared.
        declared: u32,
        /// The count computable from the payload.
        actual: u32,
    },
    /// A conversation fact disagreed with the immutable import provenance
    /// carried beside it.
    #[error("conversation and import provenance disagree")]
    ConversationProvenanceMismatch,
}
