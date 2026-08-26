//! The failure taxonomy mapped onto the shared error envelope:
//! [`TransferErrorCode`] and the reference [`TransferFailure`] set.

use ratatoskr_error_contracts::ErrorCode;

/// Every stable, machine-actionable code this protocol can fail with.
///
/// The code is the contract; a consumer branches on it and on nothing else (`AGENTS.md`
/// principle 7). All codes live in the `blob_transfer.` namespace so a bounded-context reader
/// can route them without a registry. The digest mismatch is deliberately absent here: it is an
/// anticipated terminal outcome ([`UploadCompletionOutcome`]), not a fault.
///
/// [`UploadCompletionOutcome`]: crate::UploadCompletionOutcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TransferErrorCode {
    /// No open session answers the presented token: never issued, already terminal, or discarded
    /// by the receiver after its mismatch cleanup.
    SessionUnknown,
    /// The session existed but its expiry instant has passed; the client opens a new session.
    SessionExpired,
    /// The receiver refuses the declared media type under its own admission policy. The wire
    /// shape was valid; the bytes were not wanted.
    MediaTypeRejected,
    /// A chunk arrived whose index lies outside the range the declaration derives.
    ChunkIndexOutOfRange,
    /// A chunk's byte length differs from the exact length its index requires.
    ChunkLengthInvalid,
    /// Bytes with a different digest arrived for an already-recorded index. The recorded chunk
    /// survives and the session stays usable.
    ChunkConflict,
    /// Finalize was called while derived indices remain unrecorded. The session stays open;
    /// sending the missing chunks and finalizing again is the recovery path.
    FinalizeIncomplete,
}

impl TransferErrorCode {
    /// The stable wire spelling inside the shared error envelope.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SessionUnknown => "blob_transfer.session_unknown",
            Self::SessionExpired => "blob_transfer.session_expired",
            Self::MediaTypeRejected => "blob_transfer.media_type_rejected",
            Self::ChunkIndexOutOfRange => "blob_transfer.chunk_index_out_of_range",
            Self::ChunkLengthInvalid => "blob_transfer.chunk_length_invalid",
            Self::ChunkConflict => "blob_transfer.chunk_conflict",
            Self::FinalizeIncomplete => "blob_transfer.finalize_incomplete",
        }
    }

    /// Whether the failing call may succeed unchanged on retry against the same session.
    ///
    /// Only premature finalize is retriable as-is: the recovery is to send the missing chunks
    /// and call again. Everything else needs a different input or a new session, and claiming
    /// retriable would send clients into loops.
    #[must_use]
    pub const fn retryable(&self) -> bool {
        matches!(self, Self::FinalizeIncomplete)
    }

    /// The shared envelope code this taxonomy maps onto.
    ///
    /// # Panics
    ///
    /// Never in practice: every spelling above satisfies the `ErrorCode` grammar by
    /// construction, and a build that broke that would be broken before any wire exists.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        #[allow(
            clippy::expect_used,
            reason = "every code spelling is written to satisfy the ErrorCode grammar; \
                      a mismatch is a compile-time-adjacent bug caught by the taxonomy test"
        )]
        {
            ErrorCode::parse(self.as_str())
                .expect("transfer codes are spelled inside the ErrorCode grammar")
        }
    }
}

/// One refused protocol step, produced by the reference semantics in [`crate::session`].
///
/// Carries what a receiver needs to answer truthfully without leaking anything about storage:
/// the stable [`code`](Self::code), the retriable classification, and the offending values.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TransferFailure {
    /// The token does not name an open session.
    #[error("no open upload session answers this token")]
    SessionUnknown,
    /// The token named a session whose expiry has passed.
    #[error("the upload session has expired")]
    SessionExpired,
    /// The index is outside the derived range.
    #[error("chunk_index {index} is outside the derived range of {count} chunks")]
    ChunkIndexOutOfRange {
        /// The refused index.
        index: u32,
        /// The derived chunk count.
        count: u32,
    },
    /// The delivery's length differs from the required length for its index.
    #[error("chunk_index {index} must carry exactly {expected_bytes} bytes, got {got_bytes}")]
    ChunkLengthInvalid {
        /// The delivered index.
        index: u32,
        /// The exact required length.
        expected_bytes: u32,
        /// The delivered length.
        got_bytes: u32,
    },
    /// Different bytes arrived for a recorded index.
    #[error("chunk_index {index} was already recorded with different bytes")]
    ChunkConflict {
        /// The contested index.
        index: u32,
    },
    /// Finalize ran before every derived index was recorded.
    #[error("finalize requires all {missing_chunks_count} missing chunks to be recorded first")]
    FinalizeIncomplete {
        /// How many indices remain unrecorded.
        missing_chunks_count: u32,
    },
    /// The receiver refuses the declared media type under its own admission policy. Produced by
    /// a receiver's session-opening path, never by the reference state machine, which sees only
    /// already-accepted sessions.
    #[error("the declared media type is not accepted by this receiver")]
    MediaTypeRejected,
}

impl TransferFailure {
    /// The stable code this refusal carries into the shared envelope.
    #[must_use]
    pub fn code(&self) -> TransferErrorCode {
        match self {
            Self::SessionUnknown => TransferErrorCode::SessionUnknown,
            Self::SessionExpired => TransferErrorCode::SessionExpired,
            Self::MediaTypeRejected => TransferErrorCode::MediaTypeRejected,
            Self::ChunkIndexOutOfRange { .. } => TransferErrorCode::ChunkIndexOutOfRange,
            Self::ChunkLengthInvalid { .. } => TransferErrorCode::ChunkLengthInvalid,
            Self::ChunkConflict { .. } => TransferErrorCode::ChunkConflict,
            Self::FinalizeIncomplete { .. } => TransferErrorCode::FinalizeIncomplete,
        }
    }

    /// Whether the same call may succeed unchanged on retry.
    #[must_use]
    pub fn retryable(&self) -> bool {
        self.code().retryable()
    }
}
