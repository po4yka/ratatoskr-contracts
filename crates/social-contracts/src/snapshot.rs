//! [`SocialSourceSnapshot`]: the normalized record plus the facts of one capture of it.

use ratatoskr_error_contracts::WarningEnvelope;
use ratatoskr_identifiers::{
    BlobRef, ContentDigest, EntityLocalId, Extensions, SocialSourceId, TenantRef, WireTimestamp,
    wire_string_newtype,
};

use crate::author::SocialAuthor;
use crate::collection::SocialFolderMembership;
use crate::error::SocialContractError;
use crate::media::SocialMediaItem;
use crate::relation::SocialRelation;
use crate::tokens::Platform;
use crate::values::{PostPermalink, PostText};
use crate::vocabulary::{
    AcquisitionMethod, CaptureCompleteness, SavedAuthority, UpstreamAvailability,
};

wire_string_newtype! {
    /// An opaque sync-checkpoint cursor produced by the capturing service's sync run: where a
    /// later sync of the same provider collection may resume.
    ///
    /// Opaque on purpose. Cursors are provider-defined continuation tokens (base64 blobs, JSON
    /// fragments); any tighter grammar would break on provider evolution, and no consumer may
    /// interpret or rewrite one. The bound is deliberately loose — printable ASCII without
    /// whitespace or control characters — so the value is safe to log and store anywhere.
    pub struct SyncCheckpointCursor {
        pattern  = r"^[!-~]{1,1024}$",
        max_len  = 1024,
        examples = ["eyJib29rbWFya19yZXNwb25zZV9pZCI6IjEifQ=="],
    }
}

/// A normalized social source plus the facts of *this* capture of it
/// (`docs/ARCHITECTURE.md` S7).
///
/// Named `SocialSourceSnapshot`, not `SocialSource`, because this repository owns wire
/// representations, not entities: two snapshots of the same `social_source_id` captured at
/// different `captured_at` instants are both valid and are not expected to be equal.
///
/// Record facts (identity, author, text, media, relations, folders) sit beside capture facts
/// (acquisition, authority, completeness, availability, checkpoint, warnings) in one flat
/// structure, mirroring the operation snapshot precedent. Both event payloads carry this whole
/// struct — state-carried transfer makes at-least-once redelivery idempotent.
///
/// `Deserialize` is hand-written because the invariant below is cross-field and serde has no
/// validation hook. It parses a private mirror struct and then checks. A field added to the
/// public struct and not the mirror would be silently dropped; test
/// `snapshot_roundtrip.rs::roundtrips_a_snapshot_carrying_every_field` fails immediately if
/// that happens.
///
/// # Cross-field invariant
///
/// Re-checkable through [`Self::validate`]:
///
/// - **S1** `completeness = partial` requires at least one warning naming what is missing.
///   The rule is asymmetric on purpose: a complete capture may still carry warnings for
///   problems that did not reduce completeness.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct SocialSourceSnapshot {
    /// Ratatoskr's own identity for this source, stable across captures and updates. Bare
    /// canonical UUID on the wire; never derived from the provider's id.
    pub social_source_id: SocialSourceId,

    /// Platform the source came from, e.g. `x`. Open token.
    pub platform: Platform,

    /// The provider's id for the post, opaque and case-sensitive. Together with `platform`
    /// this identifies the source upstream; deduplication keys on both.
    pub external_post_id: EntityLocalId,

    /// Canonical HTTPS permalink of the source. Absent when the provider exposes none or it
    /// was not resolvable at capture time; never an empty string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permalink: Option<PostPermalink>,

    /// The user whose library holds this source. Every snapshot of this id carries the same
    /// owner.
    pub owner: TenantRef,

    /// Author identity as observed at capture time, denormalized so one payload suffices to
    /// attribute and index the source. Absent when the producing service could not observe any
    /// author account — for example a preserved fallback record for an unavailable source.
    /// Absence means authorship is unknown; it is never a claim that the source has no author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<SocialAuthor>,

    /// Instant the provider says the source was published. Provider-authored: present only
    /// when the provider supplied it, never inferred from capture time. Absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<WireTimestamp>,

    /// Instant the producing service captured or last refreshed this record into the library.
    /// Observed: the producer's clock, not the provider's.
    pub captured_at: WireTimestamp,

    /// Normalized text of the post, with line breaks preserved. Absent for media-only posts
    /// and for sources whose text was not obtainable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<PostText>,

    /// Media attachments by reference. Empty for text-only sources. Ordering follows the
    /// provider's own attachment order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub media: Vec<SocialMediaItem>,

    /// Quote, reply and repost links to other posts on the same platform. Empty when none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<SocialRelation>,

    /// Memberships in provider-native folders, populated only where the platform exposes them
    /// through a supported channel. Empty does not mean uncaptured: an explicit capture with
    /// no folders is a complete representation of what happened.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub folders: Vec<SocialFolderMembership>,

    /// Digest of the normalized content (text plus media references) computed by the
    /// producer. Consumers treat a mismatch on recomputation as corruption of the record, not
    /// as a content change.
    pub content_digest: ContentDigest,

    /// Reference to the raw provider bytes backing this record, when they were stored. This
    /// is the preservation channel: re-parsing the raw form must remain possible even when
    /// this contract cannot represent something the provider sent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_blob: Option<BlobRef>,

    /// How the source entered the library. Closed vocabulary.
    pub acquisition: AcquisitionMethod,

    /// What the saved-state claim is worth. Closed vocabulary; an unknown authority stops
    /// processing rather than being guessed at.
    pub saved_authority: SavedAuthority,

    /// Whether this capture obtained everything the producer knows how to capture. Closed
    /// vocabulary; `partial` requires warnings (invariant S1).
    pub completeness: CaptureCompleteness,

    /// Observed upstream availability of the source at `captured_at`. Closed vocabulary. A
    /// deleted source keeps whatever was captured before it went away.
    pub upstream_availability: UpstreamAvailability,

    /// Sync-checkpoint cursor of the sync run that produced this capture, when one exists.
    /// Opaque; consumers never interpret or rewrite it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<SyncCheckpointCursor>,

    /// Non-blocking problems recorded during capture. Non-empty whenever `completeness` is
    /// `partial`; permitted on `complete` for problems that did not reduce it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<WarningEnvelope>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl SocialSourceSnapshot {
    /// Re-checks every cross-field invariant. `Deserialize` calls this; a producer that
    /// mutates a snapshot in place calls it again before emitting.
    ///
    /// # Errors
    ///
    /// [`SocialContractError::PartialWithoutWarning`] for S1.
    pub fn validate(&self) -> Result<(), SocialContractError> {
        // S1
        if self.completeness == CaptureCompleteness::Partial && self.warnings.is_empty() {
            return Err(SocialContractError::PartialWithoutWarning);
        }
        Ok(())
    }
}

/// The wire mirror of [`SocialSourceSnapshot`], parsed before the cross-field invariant runs.
///
/// Serde offers no post-deserialization validation hook, so the public type's `Deserialize` is
/// hand-written and delegates the field-by-field work here.
///
/// **Drift guard:** a field added to [`SocialSourceSnapshot`] and not to this mirror is
/// silently dropped on parse. Test
/// `crates/social-contracts/tests/snapshot_roundtrip.rs::roundtrips_a_snapshot_carrying_every_field`
/// fails the moment that happens. Do not add a field to one of these structs alone.
#[derive(Debug, serde::Deserialize)]
struct SocialSourceSnapshotWire {
    social_source_id: SocialSourceId,
    platform: Platform,
    external_post_id: EntityLocalId,
    #[serde(default)]
    permalink: Option<PostPermalink>,
    owner: TenantRef,
    #[serde(default)]
    author: Option<SocialAuthor>,
    #[serde(default)]
    published_at: Option<WireTimestamp>,
    captured_at: WireTimestamp,
    #[serde(default)]
    text: Option<PostText>,
    #[serde(default)]
    media: Vec<SocialMediaItem>,
    #[serde(default)]
    relations: Vec<SocialRelation>,
    #[serde(default)]
    folders: Vec<SocialFolderMembership>,
    content_digest: ContentDigest,
    #[serde(default)]
    raw_blob: Option<BlobRef>,
    acquisition: AcquisitionMethod,
    saved_authority: SavedAuthority,
    completeness: CaptureCompleteness,
    upstream_availability: UpstreamAvailability,
    #[serde(default)]
    checkpoint: Option<SyncCheckpointCursor>,
    #[serde(default)]
    warnings: Vec<WarningEnvelope>,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for SocialSourceSnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = SocialSourceSnapshotWire::deserialize(deserializer)?;
        let snapshot = Self {
            social_source_id: wire.social_source_id,
            platform: wire.platform,
            external_post_id: wire.external_post_id,
            permalink: wire.permalink,
            owner: wire.owner,
            author: wire.author,
            published_at: wire.published_at,
            captured_at: wire.captured_at,
            text: wire.text,
            media: wire.media,
            relations: wire.relations,
            folders: wire.folders,
            content_digest: wire.content_digest,
            raw_blob: wire.raw_blob,
            acquisition: wire.acquisition,
            saved_authority: wire.saved_authority,
            completeness: wire.completeness,
            upstream_availability: wire.upstream_availability,
            checkpoint: wire.checkpoint,
            warnings: wire.warnings,
            extensions: wire.extensions,
        };
        snapshot.validate().map_err(serde::de::Error::custom)?;
        Ok(snapshot)
    }
}
