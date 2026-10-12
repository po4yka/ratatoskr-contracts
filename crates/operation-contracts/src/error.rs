//! The parse and cross-field validation error type for operation contracts.

/// Every way an operation contract value can fail to be constructed or parsed.
///
/// A *parse* error type. It is never serialized onto the wire — that is
/// [`ErrorEnvelope`](ratatoskr_error_contracts::ErrorEnvelope)'s job.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum OperationContractError {
    /// A completion estimate outside the inclusive 0..=100 range.
    #[error("progress_percent must be 0..=100; got {got}")]
    ProgressPercentOutOfRange {
        /// The offending value.
        got: u8,
    },

    /// Invariant I1: a terminal status without a termination instant, or the reverse.
    #[error(
        "terminated_at must be present exactly when status is terminal; status is {status}, \
         terminated_at present = {present}"
    )]
    TerminationMismatch {
        /// Wire token of the status that was present.
        status: &'static str,
        /// Whether `terminated_at` was present.
        present: bool,
    },

    /// Invariant I2: `failed` with no structured error to explain it.
    #[error("status `failed` requires at least one error")]
    FailedWithoutError,

    /// Invariant I3: `succeeded` carrying terminal errors.
    #[error("status `succeeded` forbids errors; found {count}")]
    SucceededWithError {
        /// How many errors were present.
        count: usize,
    },

    /// Invariant I4: `partially_succeeded` with nothing that says which effects were not produced.
    #[error("status `partially_succeeded` requires at least one warning or error")]
    PartialWithoutDiagnostic,

    /// Invariant I5: an instant that precedes the instant the operation was accepted.
    #[error("{later} ({later_value}) is earlier than accepted_at ({accepted_at})")]
    TimestampsOutOfOrder {
        /// Name of the field that is out of order.
        later: &'static str,
        /// Canonical wire rendering of the offending instant.
        later_value: String,
        /// Canonical wire rendering of `accepted_at`.
        accepted_at: String,
    },

    /// An AI archive summary was attached to a result of another kind.
    #[error("ai_archive_import_summary requires result_kind `ai_archive.import`; got `{actual}`")]
    AiArchiveSummaryUnexpectedResultKind {
        /// The result kind carried by the invalid result.
        actual: String,
    },

    /// An AI archive summary identifies a different archive from the result target.
    #[error(
        "ai_archive_import_summary archive id must match target; expected `{expected}`, got `{actual}`"
    )]
    AiArchiveSummaryTargetMismatch {
        /// The archive reference derived from the typed summary.
        expected: String,
        /// The target reference supplied by the result.
        actual: String,
    },

    /// A deletion acknowledgement with `outcome` `refused` and no `refusal`.
    #[error("a deletion acknowledgement with outcome `refused` requires a refusal")]
    DeletionAckRefusedWithoutRefusal,

    /// A deletion acknowledgement with `outcome` `applied` that carries a `refusal`.
    #[error("a deletion acknowledgement with outcome `applied` forbids a refusal")]
    DeletionAckAppliedWithRefusal,

    /// A stream erasure request whose `request_ref` is not a `privacy_request`.
    #[error("stream erasure request_ref must be of kind `privacy_request`; got `{actual}`")]
    StreamErasureRequestKind {
        /// The kind that was present.
        actual: String,
    },

    /// A stream erasure request that names no sequence.
    #[error("stream erasure requires at least one stream sequence")]
    EmptyStreamSequences,

    /// A stream erasure request that names more sequences than one command may carry.
    #[error("stream erasure carries at most {max} stream sequences; got {got}")]
    TooManyStreamSequences {
        /// The most sequences one command may name.
        max: usize,
        /// How many sequences were present.
        got: usize,
    },

    /// A stream sequence of zero; the broker numbers messages from 1.
    #[error("stream sequences start at 1; got 0 at index {index}")]
    ZeroStreamSequence {
        /// Position of the offending entry.
        index: usize,
    },

    /// A stream sequence a JSON consumer cannot read exactly.
    #[error("stream sequence {got} at index {index} exceeds 2^53 - 1")]
    StreamSequenceAboveJsonSafeRange {
        /// Position of the offending entry.
        index: usize,
        /// The offending value.
        got: u64,
    },

    /// Stream sequences that are not strictly ascending: a descending pair or a duplicate.
    #[error(
        "stream sequences must be strictly ascending; {next} follows {previous} at index {index}"
    )]
    StreamSequencesNotAscending {
        /// Position of the entry that breaks the order.
        index: usize,
        /// The entry before it.
        previous: u64,
        /// The offending entry.
        next: u64,
    },

    /// An identifier inside an operation contract failed to parse.
    #[error(transparent)]
    Identifier(#[from] ratatoskr_identifiers::IdentifierError),
}
