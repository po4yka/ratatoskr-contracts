//! Chunk addressing: the total arithmetic every participant derives from one declaration.

use crate::error::TransferContractError;

/// Smallest legal chunk size in bytes. Below it, per-chunk bookkeeping dwarfs the bytes a
/// delivery carries.
pub const CHUNK_SIZE_MIN_BYTES: u32 = 65_536;

/// Largest legal chunk size in bytes. One delivery never asks an implementation to buffer more
/// than this.
pub const CHUNK_SIZE_MAX_BYTES: u32 = 16_777_216;

/// Largest derived chunk count. Bounds status answers and resume views for every implementation.
pub const MAX_CHUNK_COUNT: u32 = 10_000;

/// The addressing plan one declaration fixes: zero-based chunk indices over a fixed size, where
/// every chunk except the last is exactly [`Self::chunk_size_bytes`] and the last is the
/// remainder.
///
/// Byte ranges were rejected for v1 on purpose: ranges make "the same bytes twice" ambiguous,
/// force receivers to reconcile overlapping or misaligned deliveries, and buy parallelism the
/// protocol does not use. An index plus a declared size leaves nothing to reconcile — replay
/// keys on `(token, index)`, divergence compares one recorded digest, and gaps are exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UploadPlan {
    declared_size_bytes: u64,
    chunk_size_bytes: u32,
    chunk_count: u32,
}

impl UploadPlan {
    /// Derives the plan from a session declaration, refusing out-of-bounds declarations before
    /// any byte moves.
    ///
    /// # Errors
    ///
    /// [`TransferContractError::ZeroDeclaredSizeBytes`],
    /// [`TransferContractError::ChunkSizeBelowMinimum`],
    /// [`TransferContractError::ChunkSizeAboveMaximum`],
    /// [`TransferContractError::TooManyChunks`].
    pub fn new(
        declared_size_bytes: u64,
        chunk_size_bytes: u32,
    ) -> Result<Self, TransferContractError> {
        if declared_size_bytes == 0 {
            return Err(TransferContractError::ZeroDeclaredSizeBytes);
        }
        if chunk_size_bytes < CHUNK_SIZE_MIN_BYTES {
            return Err(TransferContractError::ChunkSizeBelowMinimum {
                got: chunk_size_bytes,
                min: CHUNK_SIZE_MIN_BYTES,
            });
        }
        if chunk_size_bytes > CHUNK_SIZE_MAX_BYTES {
            return Err(TransferContractError::ChunkSizeAboveMaximum {
                got: chunk_size_bytes,
                max: CHUNK_SIZE_MAX_BYTES,
            });
        }
        let chunk_count = expected_chunk_count(declared_size_bytes, chunk_size_bytes)?;
        Ok(Self {
            declared_size_bytes,
            chunk_size_bytes,
            chunk_count,
        })
    }

    /// The declared total byte size.
    #[must_use]
    pub const fn declared_size_bytes(&self) -> u64 {
        self.declared_size_bytes
    }

    /// The declared fixed chunk size.
    #[must_use]
    pub const fn chunk_size_bytes(&self) -> u32 {
        self.chunk_size_bytes
    }

    /// The number of chunks the declaration derives; also the exclusive upper bound of legal
    /// indices.
    #[must_use]
    pub const fn expected_chunk_count(&self) -> u32 {
        self.chunk_count
    }

    /// The exact length chunk `index` must have, or `None` when the index is out of range. Every
    /// chunk except the last is the full fixed size; the last is the remainder.
    #[must_use]
    pub fn chunk_len(&self, index: u32) -> Option<u32> {
        if index >= self.chunk_count {
            return None;
        }
        if index == self.chunk_count - 1 {
            // The remainder of a division by `chunk_size_bytes` is strictly below it, and
            // `chunk_size_bytes` is already a `u32`, so the narrowing cannot truncate.
            #[allow(
                clippy::cast_possible_truncation,
                reason = "remainder < chunk_size_bytes <= u32::MAX"
            )]
            let remainder = (self.declared_size_bytes % u64::from(self.chunk_size_bytes)) as u32;
            Some(if remainder == 0 {
                self.chunk_size_bytes
            } else {
                remainder
            })
        } else {
            Some(self.chunk_size_bytes)
        }
    }
}

/// The chunk count a declaration derives, refused when it exceeds [`MAX_CHUNK_COUNT`].
///
/// # Errors
///
/// [`TransferContractError::TooManyChunks`].
pub(crate) fn expected_chunk_count(
    declared_size_bytes: u64,
    chunk_size_bytes: u32,
) -> Result<u32, TransferContractError> {
    let size = chunk_size_bytes;
    let count = declared_size_bytes.div_ceil(u64::from(size));
    let count = u32::try_from(count).map_err(|_| TransferContractError::TooManyChunks {
        got: u32::MAX,
        max: MAX_CHUNK_COUNT,
    })?;
    if count > MAX_CHUNK_COUNT {
        return Err(TransferContractError::TooManyChunks {
            got: count,
            max: MAX_CHUNK_COUNT,
        });
    }
    Ok(count)
}
