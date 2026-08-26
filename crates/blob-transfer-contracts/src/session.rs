//! The transport-free reference semantics of one upload session: [`UploadSession`].
//!
//! This is protocol definition, not business logic: no storage, no I/O, no clock. Every
//! receiving service may implement its own bookkeeping, but the observable answers — what is
//! recorded, what a replay yields, when finalize refuses and what it concludes — are the rules
//! this module states, and the acceptance tests run against them.

use std::collections::BTreeMap;

use ratatoskr_identifiers::{BlobOwner, DigestHex, MediaType};
use sha2::{Digest as _, Sha256};

use crate::chunks::UploadPlan;
use crate::codes::TransferFailure;
use crate::receipt::{UploadCompletionOutcome, UploadSessionState, UploadStatusResponse};
use crate::request::UploadSessionRequest;

/// One chunk delivery's effect on the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkRecordOutcome {
    /// The bytes were newly recorded under the index.
    Recorded,
    /// The index was already recorded with the same digest; nothing changed. The receipt a
    /// receiver answers with carries `idempotent_replay = true`.
    IdempotentReplay,
}

/// A single open-or-terminal upload session driven through the protocol's steps.
///
/// Constructed from an [`UploadSessionRequest`] (whose declaration bounds were already checked),
/// advanced by [`Self::record`], surveyed by [`Self::status_view`], and sealed by
/// [`Self::finalize`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadSession {
    plan: UploadPlan,
    media_type: MediaType,
    declared_hex: DigestHex,
    recorded: BTreeMap<u32, DigestHex>,
    state: UploadSessionState,
}

impl UploadSession {
    /// Opens the reference session for a validated declaration.
    ///
    /// # Errors
    ///
    /// The declaration's own bounds — zero size, out-of-bounds chunk size, or too many derived
    /// chunks.
    pub fn new(request: &UploadSessionRequest) -> Result<Self, crate::TransferContractError> {
        Ok(Self {
            plan: UploadPlan::new(request.declared_size_bytes, request.chunk_size_bytes)?,
            media_type: request.media_type.clone(),
            declared_hex: request.digest.hex.clone(),
            recorded: BTreeMap::new(),
            state: UploadSessionState::Open,
        })
    }

    /// Records one delivery: exact-length check against the index, then idempotent replay or
    /// conflict against the already-recorded digest.
    ///
    /// # Errors
    ///
    /// [`TransferFailure::SessionUnknown`] on a terminal session,
    /// [`TransferFailure::ChunkIndexOutOfRange`], [`TransferFailure::ChunkLengthInvalid`],
    /// [`TransferFailure::ChunkConflict`]. A refused delivery never changes the session.
    pub fn record(
        &mut self,
        chunk_index: u32,
        chunk_bytes: &[u8],
    ) -> Result<ChunkRecordOutcome, TransferFailure> {
        if self.state != UploadSessionState::Open {
            return Err(TransferFailure::SessionUnknown);
        }
        let Some(expected_len) = self.plan.chunk_len(chunk_index) else {
            return Err(TransferFailure::ChunkIndexOutOfRange {
                index: chunk_index,
                count: self.plan.expected_chunk_count(),
            });
        };
        let got_len = u32::try_from(chunk_bytes.len()).unwrap_or(u32::MAX);
        if got_len != expected_len {
            return Err(TransferFailure::ChunkLengthInvalid {
                index: chunk_index,
                expected_bytes: expected_len,
                got_bytes: got_len,
            });
        }
        let digest = chunk_digest_hex(chunk_bytes);
        if let Some(recorded) = self.recorded.get(&chunk_index) {
            if *recorded == digest {
                return Ok(ChunkRecordOutcome::IdempotentReplay);
            }
            return Err(TransferFailure::ChunkConflict { index: chunk_index });
        }
        self.recorded.insert(chunk_index, digest);
        Ok(ChunkRecordOutcome::Recorded)
    }

    /// The resume view for `token`, naming exactly which indices are recorded.
    #[must_use]
    pub fn status_view(
        &self,
        resumption_token: &crate::UploadResumptionToken,
    ) -> UploadStatusResponse {
        UploadStatusResponse {
            resumption_token: resumption_token.clone(),
            session_state: self.state,
            received_chunks: self.recorded.keys().copied().collect(),
            received_chunks_count: self.recorded_len(),
            missing_chunks_count: self.missing_chunks_count(),
            extensions: ratatoskr_identifiers::Extensions::new(),
        }
    }

    /// How many derived indices remain unrecorded.
    #[must_use]
    pub fn missing_chunks_count(&self) -> u32 {
        self.plan.expected_chunk_count() - self.recorded_len()
    }

    /// The recorded-set length as the wire's `u32`: recording is capped by
    /// [`MAX_CHUNK_COUNT`](crate::MAX_CHUNK_COUNT), so the narrowing cannot truncate.
    fn recorded_len(&self) -> u32 {
        // At most MAX_CHUNK_COUNT (10_000) entries are recordable, so the narrowing cannot
        // truncate; statement-level allow because attributes on expressions are unstable.
        #![allow(clippy::cast_possible_truncation)]
        let _ = 0;
        self.recorded.len() as u32
    }

    /// Seals the session once every derived index is recorded: verifies the receiver's streamed
    /// digest against the declaration and answers the truthful terminal outcome. The session is
    /// terminal afterwards: further deliveries answer `SessionUnknown`.
    ///
    /// # Errors
    ///
    /// [`TransferFailure::SessionUnknown`] on a terminal session;
    /// [`TransferFailure::FinalizeIncomplete`] — retriable — while indices remain unrecorded; the
    /// session stays open in that case.
    pub fn finalize(
        &mut self,
        owner_service: BlobOwner,
        streamed_sha256_hex: DigestHex,
    ) -> Result<UploadCompletionOutcome, TransferFailure> {
        if self.state != UploadSessionState::Open {
            return Err(TransferFailure::SessionUnknown);
        }
        let missing = self.missing_chunks_count();
        if missing > 0 {
            return Err(TransferFailure::FinalizeIncomplete {
                missing_chunks_count: missing,
            });
        }
        if streamed_sha256_hex == self.declared_hex {
            self.state = UploadSessionState::Finalized;
            Ok(UploadCompletionOutcome::Stored {
                blob_ref: ratatoskr_identifiers::BlobRef {
                    owner_service,
                    digest: ratatoskr_identifiers::ContentDigest {
                        algorithm: ratatoskr_identifiers::DigestAlgorithm::Sha256,
                        hex: streamed_sha256_hex,
                    },
                    media_type: self.media_type.clone(),
                    length_bytes: self.plan.declared_size_bytes(),
                },
                extensions: ratatoskr_identifiers::Extensions::new(),
            })
        } else {
            self.state = UploadSessionState::Failed;
            Ok(UploadCompletionOutcome::DigestMismatch {
                declared_sha256_hex: self.declared_hex.clone(),
                computed_sha256_hex: streamed_sha256_hex,
                extensions: ratatoskr_identifiers::Extensions::new(),
            })
        }
    }
}

/// The lowercase-hex SHA-256 of delivered bytes, exactly how every participant must derive it.
///
/// # Panics
///
/// Never for any input that reaches the parse: a SHA-256 digest is always 64 lowercase hex, and
/// the local allow documents why the grammar check cannot fail.
#[allow(
    clippy::missing_panics_doc,
    reason = "64 hexadecimal digits always satisfy DigestHex's grammar"
)]
#[must_use]
pub fn chunk_digest_hex(chunk_bytes: &[u8]) -> DigestHex {
    let hashed = Sha256::digest(chunk_bytes);
    #[allow(
        clippy::expect_used,
        reason = "64 hexadecimal digits always satisfy DigestHex's grammar"
    )]
    {
        DigestHex::parse(&hex_encode(&hashed)).expect("a SHA-256 digest is 64 lowercase hex")
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use core::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}
