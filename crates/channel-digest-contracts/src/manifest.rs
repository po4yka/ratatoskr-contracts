//! The immutable channel-digest source manifest, its canonical bytes and the loopback read
//! binding between `ratatoskr-knowledge` and `ratatoskr-channel-digests` (XR-021 CONTRACTS.md
//! S08).
//!
//! A manifest is a closed, digest-verified artifact: `ratatoskr-channel-digests` serializes one
//! with [`ChannelDigestManifest::to_canonical_bytes`], stores those exact bytes, and answers
//! every read with them unchanged. `ratatoskr-knowledge` hashes what it received with
//! [`sha256_hex`], compares the hash with the digest carried by the recap request, and decodes
//! with [`ChannelDigestManifest::from_canonical_bytes`], which refuses any byte sequence that is
//! not the canonical rendering of the value it parses to.
//!
//! # Canonical bytes
//!
//! Compact UTF-8 JSON, no trailing newline, every object rebuilt with its keys inserted in
//! lexicographic order. The key order is produced explicitly, so a `preserve_order` feature
//! enabled anywhere in a consumer's dependency graph cannot change the bytes.
//!
//! # HTTP surface
//!
//! See `docs/CHANNEL_DIGEST_MANIFEST.md` for the routes, headers and statuses.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use ratatoskr_identifiers::{ContentDigest, DigestAlgorithm, TenantRef, WireTimestamp};
use regex::Regex;
use sha2::{Digest as _, Sha256};

use crate::{
    ChannelDigestContractError, ChannelDigestManifestRef, ChannelDigestRunId, DigestWindow,
};

/// Path template of the manifest read route on the channel-digests API listener.
pub const MANIFEST_PATH_TEMPLATE: &str = "/v1/manifests/{manifest_id}";

/// Path of the readiness route on the channel-digests API listener.
pub const READY_PATH: &str = "/ready";

/// Request header carrying the bare lowercase UUID of the owning user, not `user:<uuid>`.
pub const HEADER_OWNER_ID: &str = "x-ratatoskr-owner-id";

/// Request header carrying the bare UUID of the digest run the caller expects the manifest to
/// belong to.
pub const HEADER_DIGEST_RUN_ID: &str = "x-ratatoskr-digest-run-id";

/// Request header carrying the 64 lowercase hexadecimal characters of the SHA-256 the caller
/// expects the response body to have.
pub const HEADER_MANIFEST_DIGEST: &str = "x-ratatoskr-manifest-digest";

/// Most sources one manifest may carry.
pub const MAX_SOURCES: usize = 100;

/// Most distinct channels one manifest may represent.
pub const MAX_CHANNELS: usize = 20;

/// Largest `content` of one source, in UTF-8 bytes.
pub const MAX_CONTENT_BYTES: usize = 16_384;

const UUID: &str = "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";
const MANIFEST_REF_PREFIX: &str = "channel-digest-manifest:";

/// The route a manifest is read from: `/v1/manifests/<bare uuid>`.
///
/// The manifest reference travels in the path, so there is no separate manifest-reference header.
#[must_use]
pub fn manifest_path(manifest_ref: &ChannelDigestManifestRef) -> String {
    let wire = manifest_ref.as_str();
    let bare = wire.strip_prefix(MANIFEST_REF_PREFIX).unwrap_or(wire);
    format!("/v1/manifests/{bare}")
}

/// Lowercase hexadecimal SHA-256 of `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Version marker of the manifest encoding.
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
pub enum ChannelDigestManifestSchema {
    /// The first and only manifest encoding.
    #[serde(rename = "channel_digest_manifest.v1")]
    V1,
}

/// One immutable selected public-channel post revision.
///
/// `content` is complete normalized untrusted text and is never truncated: a post that does not
/// fit [`MAX_CONTENT_BYTES`] is not selectable.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ChannelDigestManifestSource {
    /// Stable immutable revision reference, `channel-post-revision:<uuid>`, at most 160
    /// characters.
    #[schemars(length(max = 160))]
    pub revision_ref: String,

    /// Stable owner-independent public-channel reference, `telegram-public-channel:<uuid>` where
    /// the UUID is the channel digest service's own channel identity, at most 160 characters.
    #[schemars(length(max = 160))]
    pub channel_ref: String,

    /// Source-attribution label for the channel: its display name, else its username, truncated
    /// to 80 characters. 1 to 80 characters.
    #[schemars(length(min = 1, max = 80))]
    pub channel_label: String,

    /// Provider message identity within the channel, a decimal integer of 1 to 32 digits.
    #[schemars(length(min = 1, max = 32))]
    pub message_id: String,

    /// Provider-authored publication instant of the message, canonical UTC.
    pub published_at: WireTimestamp,

    /// Complete normalized post revision, 1 to 16384 UTF-8 bytes.
    #[schemars(length(min = 1, max = 16384))]
    pub content: String,

    /// SHA-256 of the exact UTF-8 bytes of `content`.
    pub content_digest: ContentDigest,

    /// Canonical public link, `https://t.me/<username>/<message_id>`, at most 512 characters
    /// and carrying no query string or fragment. Absent when the channel has no public username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(max = 512))]
    pub public_link: Option<String>,

    /// Ordinal of this body among all observed revisions of the same message in the same channel,
    /// starting at 1 and ordered by observation instant, then revision identity.
    pub revision: u32,
}

/// The immutable manifest of the source revisions one digest run selected.
///
/// Closed: an unknown member is a decoding failure, not something to preserve.
/// `Deserialize` runs [`ChannelDigestManifest::validate`], so a decoded manifest always satisfies
/// the cross-field rules.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChannelDigestManifest {
    /// Encoding marker.
    pub schema: ChannelDigestManifestSchema,

    /// Owner-authorized opaque reference of this manifest.
    pub manifest_ref: ChannelDigestManifestRef,

    /// The user the manifest belongs to.
    pub owner: TenantRef,

    /// The digest run that selected the sources.
    pub digest_run_id: ChannelDigestRunId,

    /// Closed-open publication window every source falls in.
    pub window: DigestWindow,

    /// The selected revisions: 1 to 100 entries, at most 20 distinct channels, strictly ascending
    /// by publication instant, channel reference, message identity compared as a decimal
    /// integer, and revision.
    #[schemars(length(min = 1, max = 100))]
    pub sources: Vec<ChannelDigestManifestSource>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ChannelDigestManifestWire {
    schema: ChannelDigestManifestSchema,
    manifest_ref: ChannelDigestManifestRef,
    owner: TenantRef,
    digest_run_id: ChannelDigestRunId,
    window: DigestWindow,
    sources: Vec<ChannelDigestManifestSource>,
}

impl From<ChannelDigestManifestWire> for ChannelDigestManifest {
    fn from(wire: ChannelDigestManifestWire) -> Self {
        Self {
            schema: wire.schema,
            manifest_ref: wire.manifest_ref,
            owner: wire.owner,
            digest_run_id: wire.digest_run_id,
            window: wire.window,
            sources: wire.sources,
        }
    }
}

impl<'de> serde::Deserialize<'de> for ChannelDigestManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let manifest = Self::from(ChannelDigestManifestWire::deserialize(deserializer)?);
        manifest.validate().map_err(serde::de::Error::custom)?;
        Ok(manifest)
    }
}

impl ChannelDigestManifest {
    /// Checks every cross-field rule of the manifest.
    ///
    /// The rules: 1 to 100 sources; at most 20 distinct `channel_ref`; every `published_at` in
    /// `[window.start_at, window.end_at)`; unique `revision_ref`; unique
    /// `(channel_ref, message_id, revision)`; sources strictly ascending by
    /// `(published_at, channel_ref, message_id compared as a decimal integer, revision)`; and the
    /// per-source rules documented on [`ChannelDigestManifestSource`].
    ///
    /// # Errors
    ///
    /// [`ChannelDigestContractError::ManifestInvalid`] for the first rule a source set breaks.
    pub fn validate(&self) -> Result<(), ChannelDigestContractError> {
        if self.sources.is_empty() || self.sources.len() > MAX_SOURCES {
            return Err(ChannelDigestContractError::ManifestInvalid);
        }
        let mut channels = BTreeSet::new();
        let mut revision_refs = BTreeSet::new();
        let mut identities = BTreeSet::new();
        let mut previous: Option<OrderKey<'_>> = None;
        for source in &self.sources {
            let message = validate_source(source, &self.window)?;
            if !revision_refs.insert(source.revision_ref.as_str()) {
                return Err(ChannelDigestContractError::ManifestInvalid);
            }
            channels.insert(source.channel_ref.as_str());
            if channels.len() > MAX_CHANNELS {
                return Err(ChannelDigestContractError::ManifestInvalid);
            }
            if !identities.insert((source.channel_ref.as_str(), message, source.revision)) {
                return Err(ChannelDigestContractError::ManifestInvalid);
            }
            let key = (
                source.published_at,
                source.channel_ref.as_str(),
                message,
                source.revision,
            );
            if previous.as_ref().is_some_and(|earlier| *earlier >= key) {
                return Err(ChannelDigestContractError::ManifestInvalid);
            }
            previous = Some(key);
        }
        Ok(())
    }

    /// The canonical bytes: compact UTF-8 JSON with every object's keys in lexicographic order
    /// and no trailing newline.
    ///
    /// # Errors
    ///
    /// [`ChannelDigestContractError::ManifestInvalid`] when the manifest fails
    /// [`ChannelDigestManifest::validate`] (an invalid manifest is never emitted), and
    /// [`ChannelDigestContractError::ManifestEncoding`] when it cannot be serialized.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, ChannelDigestContractError> {
        self.validate()?;
        let value =
            serde_json::to_value(self).map_err(|_| ChannelDigestContractError::ManifestEncoding)?;
        serde_json::to_vec(&sort_keys(value))
            .map_err(|_| ChannelDigestContractError::ManifestEncoding)
    }

    /// Decodes bytes that must be the canonical rendering of the manifest they contain.
    ///
    /// # Errors
    ///
    /// [`ChannelDigestContractError::ManifestEncoding`] when the bytes are not JSON of the closed
    /// manifest shape, [`ChannelDigestContractError::ManifestInvalid`] when the decoded manifest
    /// breaks a cross-field rule, and [`ChannelDigestContractError::ManifestIntegrity`] when
    /// re-canonicalizing the decoded value does not reproduce the input bytes (reordered keys,
    /// pretty printing, a trailing newline, duplicate keys or an explicit null).
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, ChannelDigestContractError> {
        let wire: ChannelDigestManifestWire = serde_json::from_slice(bytes)
            .map_err(|_| ChannelDigestContractError::ManifestEncoding)?;
        let manifest = Self::from(wire);
        if manifest.to_canonical_bytes()? != bytes {
            return Err(ChannelDigestContractError::ManifestIntegrity);
        }
        Ok(manifest)
    }
}

type OrderKey<'a> = (WireTimestamp, &'a str, u128, u32);

fn sort_keys(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(members) => {
            let mut ordered: Vec<(String, serde_json::Value)> = members.into_iter().collect();
            ordered.sort_by(|left, right| left.0.cmp(&right.0));
            let mut rebuilt = serde_json::Map::new();
            for (key, member) in ordered {
                rebuilt.insert(key, sort_keys(member));
            }
            serde_json::Value::Object(rebuilt)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(sort_keys).collect())
        }
        scalar => scalar,
    }
}

/// Checks one source against the window and the per-field rules; returns its message identity
/// as an integer for ordering.
fn validate_source(
    source: &ChannelDigestManifestSource,
    window: &DigestWindow,
) -> Result<u128, ChannelDigestContractError> {
    let invalid = ChannelDigestContractError::ManifestInvalid;
    if source.revision_ref.len() > 160
        || source.channel_ref.len() > 160
        || !revision_ref_pattern().is_match(&source.revision_ref)
        || !channel_ref_pattern().is_match(&source.channel_ref)
        || !(1..=80).contains(&source.channel_label.chars().count())
        || source.revision == 0
        || source.published_at < window.start_at
        || source.published_at >= window.end_at
        || source.content.is_empty()
        || source.content.len() > MAX_CONTENT_BYTES
        || source.content_digest.algorithm != DigestAlgorithm::Sha256
        || source.content_digest.hex.as_str() != sha256_hex(source.content.as_bytes())
    {
        return Err(invalid);
    }
    if let Some(link) = &source.public_link
        && (link.len() > 512 || !public_link_pattern().is_match(link))
    {
        return Err(invalid);
    }
    if !message_id_pattern().is_match(&source.message_id) {
        return Err(invalid);
    }
    source.message_id.parse::<u128>().map_err(|_| invalid)
}

#[allow(
    clippy::expect_used,
    reason = "the static patterns are compiled by the manifest tests"
)]
fn compiled(pattern: &str) -> Regex {
    Regex::new(pattern).expect("a static manifest pattern compiles")
}

fn revision_ref_pattern() -> &'static Regex {
    static COMPILED: LazyLock<Regex> =
        LazyLock::new(|| compiled(&format!("^channel-post-revision:{UUID}$")));
    &COMPILED
}

fn channel_ref_pattern() -> &'static Regex {
    static COMPILED: LazyLock<Regex> =
        LazyLock::new(|| compiled(&format!("^telegram-public-channel:{UUID}$")));
    &COMPILED
}

fn message_id_pattern() -> &'static Regex {
    static COMPILED: LazyLock<Regex> = LazyLock::new(|| compiled("^[0-9]{1,32}$"));
    &COMPILED
}

fn public_link_pattern() -> &'static Regex {
    static COMPILED: LazyLock<Regex> =
        LazyLock::new(|| compiled(r"^https://t\.me/[A-Za-z0-9_]{1,64}/[0-9]{1,32}$"));
    &COMPILED
}
