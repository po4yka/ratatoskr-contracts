//! Content-addressed references to bytes owned by a Ratatoskr service.

use crate::wire_string_newtype;

wire_string_newtype! {
    /// Deployment identity of the service that owns and resolves a blob.
    pub struct BlobOwner {
        pattern  = r"^[a-z][a-z0-9-]{1,63}$",
        max_len  = 64,
        examples = ["ratatoskr-extractor", "ratatoskr-vault"],
    }
}

wire_string_newtype! {
    /// Lowercase hexadecimal bytes of a content digest.
    pub struct DigestHex {
        pattern  = r"^[0-9a-f]{64}$",
        max_len  = 64,
        examples = ["0000000000000000000000000000000000000000000000000000000000000000"],
    }
}

wire_string_newtype! {
    /// Internet media type of stored bytes, without parameters.
    pub struct MediaType {
        pattern  = r"^[a-z0-9][a-z0-9!#$&^_.+-]{0,126}/[a-z0-9][a-z0-9!#$&^_.+-]{0,126}$",
        max_len  = 255,
        examples = ["text/html", "application/pdf"],
    }
}

/// Algorithm used to calculate a content digest.
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
pub enum DigestAlgorithm {
    /// SHA-256.
    Sha256,
}

/// A content digest with an explicit algorithm.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ContentDigest {
    /// Hash algorithm used to produce `hex`.
    pub algorithm: DigestAlgorithm,

    /// Lowercase hexadecimal digest bytes.
    pub hex: DigestHex,
}

/// Reference to content-addressed bytes owned by one service.
///
/// This is a reference, not a storage API. The owner writes and resolves the bytes under its own
/// content-addressed directory. No host, filesystem path, signed URL, credentials, or expiry can
/// appear in this contract.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct BlobRef {
    /// Service that owns the bytes and their lifecycle.
    pub owner_service: BlobOwner,

    /// Digest that names the bytes and lets a reader verify them.
    pub digest: ContentDigest,

    /// Media type of the stored bytes.
    pub media_type: MediaType,

    /// Exact byte length of the stored value.
    pub length_bytes: u64,
}
