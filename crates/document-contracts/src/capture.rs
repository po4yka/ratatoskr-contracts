//! The command that asks the extractor to capture a document (XR-021 CONTRACTS.md S11).

use ratatoskr_event_envelope::CommandPayload;
use ratatoskr_identifiers::{BlobRef, ContentDigest, Extensions, OperationId, wire_string_newtype};

wire_string_newtype! {
    /// An absolute `http` or `https` URL to capture, at most 2048 characters and without
    /// spaces or control characters.
    ///
    /// This type fixes the carrier only. Whether the URL is publicly fetchable is decided by the
    /// extractor's own address policy, which refuses a private or non-routable target.
    pub struct CaptureUrl {
        pattern  = r"^https?://[^\x00-\x20\x7f]+$",
        max_len  = 2048,
        examples = ["https://example.org/articles/borrow-checker"],
    }
}

/// Why a [`ContentCaptureRequested`] payload does not name exactly one source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CaptureContractError {
    /// Neither a `url` nor a `blob` was given.
    #[error("a capture must name exactly one source, a url or a blob; neither was given")]
    MissingSource,
    /// Both a `url` and a `blob` were given.
    #[error("a capture must name exactly one source, a url or a blob; both were given")]
    AmbiguousSource,
}

/// Payload of `content.capture.requested.v1`: capture one document and extract it.
///
/// Exactly one source is named:
///
/// - `url`: an `http` or `https` address the extractor fetches.
/// - `blob`: bytes another service already stored, named by a content-addressed [`BlobRef`]. The
///   extractor reads them from the owning service's blob root, so the digest algorithm is
///   `sha256` (the only algorithm [`ContentDigest`] has) and the owner decides what it is willing
///   to hand over.
///
/// A payload with both or neither is refused when it is decoded. The generated JSON Schema lists
/// both members as optional: it cannot say "exactly one" without a root `oneOf`, which the
/// TypeScript projection refuses as unrepresentable, so the Rust type alone enforces the rule
/// (the ADR-0001 lower-bound rule) and the invalid fixtures name the serde layer.
///
/// The enclosing command envelope carries the owner (`tenant_id`), correlation and delivery
/// identity; its `aggregate_id` is `operation:<operation_id>`, and the extractor refuses an
/// envelope whose aggregate disagrees with `operation_id`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct ContentCaptureRequested {
    /// Platform operation this capture belongs to.
    pub operation_id: OperationId,

    /// SHA-256 of the caller's idempotency string, so a redelivery or a repeated request
    /// collapses into one capture. For a capture issued from a social bookmark the string is the
    /// capture identity.
    pub idempotency_key: ContentDigest,

    /// The address to fetch. Absent exactly when `blob` is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<CaptureUrl>,

    /// The stored bytes to extract. Absent exactly when `url` is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<BlobRef>,

    /// Unknown-but-preserved additive payload members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl ContentCaptureRequested {
    /// Checks that exactly one of `url` and `blob` is present.
    ///
    /// `Deserialize` calls this; a producer that builds or mutates a payload calls it again
    /// before emitting.
    ///
    /// # Errors
    ///
    /// [`CaptureContractError::MissingSource`] when neither is present and
    /// [`CaptureContractError::AmbiguousSource`] when both are.
    pub fn validate(&self) -> Result<(), CaptureContractError> {
        match (&self.url, &self.blob) {
            (Some(_), None) | (None, Some(_)) => Ok(()),
            (None, None) => Err(CaptureContractError::MissingSource),
            (Some(_), Some(_)) => Err(CaptureContractError::AmbiguousSource),
        }
    }
}

#[derive(serde::Deserialize)]
struct ContentCaptureRequestedWire {
    operation_id: OperationId,
    idempotency_key: ContentDigest,
    #[serde(default)]
    url: Option<CaptureUrl>,
    #[serde(default)]
    blob: Option<BlobRef>,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for ContentCaptureRequested {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ContentCaptureRequestedWire::deserialize(deserializer)?;
        let payload = Self {
            operation_id: wire.operation_id,
            idempotency_key: wire.idempotency_key,
            url: wire.url,
            blob: wire.blob,
            extensions: wire.extensions,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}

impl CommandPayload for ContentCaptureRequested {
    const COMMAND_TYPE: &'static str = "content.capture.requested.v1";
}
