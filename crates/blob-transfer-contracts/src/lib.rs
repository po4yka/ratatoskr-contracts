//! The transfer protocol between upload-capable clients and receiving services' blob stores:
//! one chunked, resumable, digest-first discipline instead of a wire dialect per service.
//!
//! # What is on the wire
//!
//! Six message shapes in two direction families:
//!
//! - Client to receiver: [`UploadSessionRequest`] (the digest-first declaration: total byte
//!   size, media type, whole-payload SHA-256 digest, chunk size) and [`UploadFinalizeRequest`].
//! - Receiver to client: [`UploadSessionOpened`] (opaque resumption token, echoed chunking,
//!   expiry instant), [`UploadChunkReceipt`] (per-chunk acknowledgement), [`UploadStatusResponse`]
//!   (the resume view of recorded chunks), and [`UploadCompletionOutcome`] (`stored`, carrying a
//!   complete [`BlobRef`], or `digest_mismatch`, carrying expected and computed digests).
//!
//! The protocol decisions live with the types: chunk addressing is by zero-based index over the
//! declared fixed chunk size ([`chunks`]), tokens are opaque and grammar-bounded
//! ([`UploadResumptionToken`]), the digest algorithm is closed at `sha256` for version 1, and
//! failures speak stable `blob_transfer.` codes mapped onto the shared error envelope
//! ([`TransferErrorCode`]).
//!
//! # The canonical HTTP binding
//!
//! The types above are transport-honest: no method, status code, header or URL appears in any of
//! them. The canonical binding every HTTP implementation starts from is normative prose:
//!
//! - `POST /v1/<service>/uploads` — body [`UploadSessionRequest`], answer `201` with
//!   [`UploadSessionOpened`];
//! - `PUT /v1/<service>/uploads/{token}/chunks/{index}` — raw chunk bytes as the body with
//!   `Content-Length` equal to the expected chunk length, answer [`UploadChunkReceipt`];
//! - `GET /v1/<service>/uploads/{token}/status` — answer [`UploadStatusResponse`];
//! - `POST /v1/<service>/uploads/{token}/finalize` — body [`UploadFinalizeRequest`], answer
//!   [`UploadCompletionOutcome`] with either variant at `200`.
//!
//! Failures ride the shared error envelope: unknown or expired tokens, conflicts, out-of-range
//! indices and premature finalization are envelope faults carrying a [`TransferErrorCode`]
//! string; the digest mismatch is deliberately *not* an envelope fault but an anticipated,
//! truthful terminal outcome. Identity crosses the hop as the edge-minted claims of platform
//! ADR-0015 (`x-ratatoskr-user-id`, `x-correlation-id`); nothing here re-authenticates.
//!
//! # What is deliberately absent
//!
//! No storage placement, quarantine or anomaly policy (receiving services own their bytes per the
//! workspace store spec `blob-references`), no multipart-form alternative, no bus event and no
//! command envelope (the completed fact is the receiver's own stored blob, announced by its
//! existing surfaces), no parallelism scheduling, and no credential or token minting guidance
//! beyond the entropy floor stated on the token type.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod chunks;
mod codes;
mod error;
mod receipt;
mod request;
mod session;
mod token;

pub use crate::chunks::{CHUNK_SIZE_MAX_BYTES, CHUNK_SIZE_MIN_BYTES, MAX_CHUNK_COUNT, UploadPlan};
pub use crate::codes::{TransferErrorCode, TransferFailure};
pub use crate::error::TransferContractError;
pub use crate::receipt::{
    UploadChunkReceipt, UploadCompletionOutcome, UploadSessionOpened, UploadSessionState,
    UploadStatusResponse,
};
pub use crate::request::{UploadFinalizeRequest, UploadSessionRequest};
pub use crate::session::{ChunkRecordOutcome, UploadSession, chunk_digest_hex};
pub use crate::token::UploadResumptionToken;

pub use ratatoskr_identifiers::{BlobRef, ContentDigest, DigestAlgorithm, DigestHex, MediaType};
