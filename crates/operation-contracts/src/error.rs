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

    /// An identifier inside an operation contract failed to parse.
    #[error(transparent)]
    Identifier(#[from] ratatoskr_identifiers::IdentifierError),
}
