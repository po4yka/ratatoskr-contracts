//! The receiver-to-client documents: [`UploadSessionOpened`], [`UploadChunkReceipt`],
//! [`UploadStatusResponse`] and [`UploadCompletionOutcome`].

use ratatoskr_identifiers::{BlobRef, DigestHex, Extensions};

use crate::token::UploadResumptionToken;

/// The receiver's answer to a session declaration: the session exists, here is its handle.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct UploadSessionOpened {
    /// The opaque handle driving every later call. Required; see [`UploadResumptionToken`].
    pub resumption_token: UploadResumptionToken,

    /// The chunk size the session actually runs with. Required; equals the requested value —
    /// version 1 has no negotiation: a receiver that cannot accept the declared chunking refuses
    /// the session outright instead of answering a different arithmetic.
    pub chunk_size_bytes: u32,

    /// Instant after which the receiver refuses the token with
    /// `blob_transfer.session_expired`. Required; server-asserted; RFC 3339 UTC canonical.
    pub expires_at: ratatoskr_identifiers::WireTimestamp,

    /// Unknown-but-preserved additive fields. A constructing receiver leaves this empty
    /// (ADR-0008).
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// One chunk accepted. Answers idempotently: the same bytes delivered twice for one index yield
/// two identical-success receipts and leave the recorded state unchanged.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct UploadChunkReceipt {
    /// The session the chunk joined. Required.
    pub resumption_token: UploadResumptionToken,

    /// The zero-based index the bytes were recorded under. Required.
    pub chunk_index: u32,

    /// How many distinct indices are now recorded, including this one when newly recorded.
    /// Required.
    pub received_chunks_count: u32,

    /// `true` when this delivery replayed an already-recorded index with matching bytes and
    /// changed nothing. Required; lets an at-least-once client distinguish news from echo.
    pub idempotent_replay: bool,

    /// Unknown-but-preserved additive fields. A constructing receiver leaves this empty
    /// (ADR-0008).
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// The phase of the session answering a status or finalize call.
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
pub enum UploadSessionState {
    /// Chunks are still being accepted.
    Open,
    /// Finalize verified the streamed digest; the blob is sealed under its reference.
    Finalized,
    /// Finalize proved the streamed digest differs from the declaration; the session is dead.
    Failed,
}

/// The resume view an interrupted client asks for: exactly which indices are recorded, so the
/// remaining chunks can be sent without duplication or guessed gaps.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct UploadStatusResponse {
    /// The session being surveyed. Required.
    pub resumption_token: UploadResumptionToken,

    /// The phase of the session. Required.
    pub session_state: UploadSessionState,

    /// Every recorded index, sorted ascending. Required; empty only for a fresh open session.
    pub received_chunks: Vec<u32>,

    /// How many indices are recorded. Required; equals `received_chunks`' length.
    pub received_chunks_count: u32,

    /// How many derived indices remain unrecorded. Required.
    pub missing_chunks_count: u32,

    /// Unknown-but-preserved additive fields. A constructing receiver leaves this empty
    /// (ADR-0008).
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// The terminal answer to finalize. Both variants are truthful answers at `200` in the canonical
/// binding: a mismatch is an anticipated fact about the bytes, not a transport fault.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "outcome", rename_all = "snake_case")]
#[non_exhaustive]
pub enum UploadCompletionOutcome {
    /// The streamed digest equaled the declaration. The reference below is everything the
    /// workspace store spec `blob-references` requires to resolve the stored bytes.
    Stored {
        /// The content-addressed reference to the sealed bytes, owned by the receiving service.
        blob_ref: BlobRef,
        /// Unknown-but-preserved additive fields (ADR-0008).
        #[serde(flatten)]
        extensions: Extensions,
    },
    /// The streamed digest differed from the declaration. Terminal-failed: the token is dead
    /// afterwards, and the client restarts with a new session rather than patching chunks.
    DigestMismatch {
        /// The digest the declaration carried.
        declared_sha256_hex: DigestHex,
        /// The digest the streamed bytes actually produced.
        computed_sha256_hex: DigestHex,
        /// Unknown-but-preserved additive fields (ADR-0008).
        #[serde(flatten)]
        extensions: Extensions,
    },
}
