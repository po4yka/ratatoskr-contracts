//! Every way a transfer declaration can violate the protocol's own bounds.

/// The protocol bounds, refused at parse time before any byte moves.
///
/// A *parse* error type: it is never serialized onto the wire — field-level rejection happens in
/// the identifier types, and only the declaration-level rules surface here.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TransferContractError {
    /// A session declared zero bytes. An empty upload is not a transfer: it has no chunks, no
    /// digest verification of streamed bytes, and no blob worth a reference.
    #[error("declared_size_bytes must be greater than zero")]
    ZeroDeclaredSizeBytes,
    /// A chunk size below [`crate::CHUNK_SIZE_MIN_BYTES`], which would make a modest upload pay
    /// per-chunk bookkeeping that dwarfs its bytes.
    #[error("chunk_size_bytes is {got}; the protocol minimum is {min}")]
    ChunkSizeBelowMinimum {
        /// The declared chunk size.
        got: u32,
        /// [`crate::CHUNK_SIZE_MIN_BYTES`].
        min: u32,
    },
    /// A chunk size above [`crate::CHUNK_SIZE_MAX_BYTES`], which would let one delivery hold
    /// more memory than the protocol promises any implementation must buffer.
    #[error("chunk_size_bytes is {got}; the protocol maximum is {max}")]
    ChunkSizeAboveMaximum {
        /// The declared chunk size.
        got: u32,
        /// [`crate::CHUNK_SIZE_MAX_BYTES`].
        max: u32,
    },
    /// A declaration whose size over its chunk size exceeds [`crate::MAX_CHUNK_COUNT`]. The
    /// bound keeps status answers and resume views bounded for every implementation.
    #[error("the declaration derives {got} chunks; the protocol maximum is {max}")]
    TooManyChunks {
        /// The derived chunk count.
        got: u32,
        /// [`crate::MAX_CHUNK_COUNT`].
        max: u32,
    },
}
