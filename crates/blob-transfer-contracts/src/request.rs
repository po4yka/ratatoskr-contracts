//! The client-to-receiver documents: [`UploadSessionRequest`] and [`UploadFinalizeRequest`].

use ratatoskr_identifiers::{ContentDigest, Extensions, MediaType};

use crate::error::TransferContractError;
use crate::token::UploadResumptionToken;

/// Opens an upload session: the digest-first declaration a client sends before any payload byte
/// moves.
///
/// The declaration is what a receiver refuses on when it refuses at all — size, media type and
/// digest are all known here, so an unwanted upload never transfers its first chunk. The digest
/// algorithm travels inside `digest` and version 1 speaks exactly
/// [`DigestAlgorithm::Sha256`]; the closed enum leaves no stringly-typed escape hatch, and a
/// future algorithm arrives as an additive variant.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct UploadSessionRequest {
    /// Exact byte size of the whole payload. Required; greater than zero; the chunk count and
    /// the last chunk's length derive from this and `chunk_size_bytes` alone.
    pub declared_size_bytes: u64,

    /// Internet media type of the payload, without parameters. Required; grammar per
    /// [`MediaType`]. Whether the type is wanted is the receiver's admission policy, surfaced as
    /// `blob_transfer.media_type_rejected`.
    pub media_type: MediaType,

    /// Digest of the whole payload. Required; finalize verifies the streamed bytes against it.
    pub digest: ContentDigest,

    /// Fixed size of every chunk except, possibly, the last. Required; within
    /// [`CHUNK_SIZE_MIN_BYTES`](crate::CHUNK_SIZE_MIN_BYTES) ..
    /// [`CHUNK_SIZE_MAX_BYTES`](crate::CHUNK_SIZE_MAX_BYTES).
    pub chunk_size_bytes: u32,

    /// Unknown-but-preserved additive fields. A constructing client leaves this empty (ADR-0008).
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl UploadSessionRequest {
    /// Re-checks every declaration-level bound. `Deserialize` calls this; a client that builds a
    /// request directly calls it before sending.
    ///
    /// # Errors
    ///
    /// The bounds of [`crate::UploadPlan::new`]: zero size, out-of-bounds chunk size, or a
    /// derived chunk count above [`crate::MAX_CHUNK_COUNT`].
    pub fn validate(&self) -> Result<(), TransferContractError> {
        crate::UploadPlan::new(self.declared_size_bytes, self.chunk_size_bytes).map(|_| ())
    }
}

/// The wire mirror of [`UploadSessionRequest`], parsed before the bounds run.
///
/// **Drift guard:** a field added to the public struct and not to this mirror would be silently
/// dropped on parse; the full-request fixture round trip fails the moment that happens.
#[derive(Debug, serde::Deserialize)]
struct UploadSessionRequestWire {
    declared_size_bytes: u64,
    media_type: MediaType,
    digest: ContentDigest,
    chunk_size_bytes: u32,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for UploadSessionRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = UploadSessionRequestWire::deserialize(deserializer)?;
        let request = Self {
            declared_size_bytes: wire.declared_size_bytes,
            media_type: wire.media_type,
            digest: wire.digest,
            chunk_size_bytes: wire.chunk_size_bytes,
            extensions: wire.extensions,
        };
        request.validate().map_err(serde::de::Error::custom)?;
        Ok(request)
    }
}

/// Asks the receiver to verify and seal a completed session. Required after every derived chunk
/// index is recorded; premature calls are refused with
/// `blob_transfer.finalize_incomplete` while the session stays open.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct UploadFinalizeRequest {
    /// The session to seal. Required.
    pub resumption_token: UploadResumptionToken,

    /// Unknown-but-preserved additive fields. A constructing client leaves this empty (ADR-0008).
    #[serde(flatten)]
    pub extensions: Extensions,
}
