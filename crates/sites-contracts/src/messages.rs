//! The release build command, the release event and the site approval event.

use ratatoskr_event_envelope::{CommandPayload, EventPayload};
use ratatoskr_identifiers::{DigestHex, OperationId, WireTimestamp};

use crate::error::SitesContractError;
use crate::events::SourceRef;
use crate::manifest::{Hostname, PublicationId, SiteCapability, SiteId, no_duplicate};
use crate::provenance::GitHubRepositoryBinding;
use crate::release::{Slug, strictly_ascending};

/// Payload of `sites.release.build_requested.v1`: build a release of one site.
///
/// A command. Edge publishes it from its outbox after the owner calls
/// `POST /v1/sites/{site}/releases`, and mints the operation of the kind `sites.release.build`
/// with the lane `ratatoskr-sites`; `ratatoskr-sites` alone consumes it and reports the operation
/// (workspace spec `site-publications`). The enclosing envelope carries the owner as `tenant_id`
/// (`user:<id>`) and the aggregate `site:<site_id>`. Every other member is refused.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReleaseBuildRequested {
    /// The Platform operation that tracks the build.
    pub operation_id: OperationId,

    /// The site whose publications the build reads.
    pub site_id: SiteId,
}

impl CommandPayload for ReleaseBuildRequested {
    const COMMAND_TYPE: &'static str = "sites.release.build_requested.v1";
}

/// Payload of `sites.release.published.v1`: a new release of a site became current.
///
/// A fact. `ratatoskr-sites` writes it in the same transaction that makes the release current, and
/// never for a build whose release equals the current one. It carries no page title, slug, vault
/// path or page content (workspace spec `site-publications`, "The release event carries no
/// content"); every other member is refused. The enclosing envelope carries the owner as
/// `tenant_id` and the aggregate `site:<site_id>`.
///
/// `Deserialize` is hand-written because `publication_ids` must be sorted and hold each id once.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReleasePublished {
    /// The site whose current release changed.
    pub site_id: SiteId,

    /// The new current release: the lowercase hex SHA-256 of the RFC 8785 form of its
    /// `sites/release.v1` document.
    pub release_id: DigestHex,

    /// The publications whose pages the release holds, in ascending order, each once.
    #[schemars(extend("uniqueItems" = true))]
    pub publication_ids: Vec<PublicationId>,

    /// The number of pages in the release.
    pub page_count: u32,

    /// The instant at which the release became current, from the sites service clock.
    pub published_at: WireTimestamp,
}

impl EventPayload for ReleasePublished {
    const EVENT_TYPE: &'static str = "sites.release.published.v1";
}

impl ReleasePublished {
    /// Checks the order of `publication_ids`. `Deserialize` calls this.
    ///
    /// # Errors
    ///
    /// [`SitesContractError::Unsorted`].
    pub fn validate(&self) -> Result<(), SitesContractError> {
        strictly_ascending(
            "publication_ids",
            self.publication_ids.iter().map(PublicationId::as_str),
        )
    }
}

/// Payload of `sites.draft.changed.v1`: a note in the publications of a site changed in the vault.
///
/// A fact. `ratatoskr-sites` publishes it when it consumes a `notes.file.changed.v1` for a note that
/// a publication of the site selects, and Edge alone reads it to send `draft.changed` to the owner
/// sessions of the site (workspace design D5). It carries no note content and no vault path; every
/// other member is refused. The enclosing envelope carries the owner as `tenant_id` and the
/// aggregate `site:<site_id>`.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct DraftChanged {
    /// The site whose publications hold the note.
    pub site_id: SiteId,

    /// The note that changed.
    pub source_ref: SourceRef,

    /// The lowercase hex SHA-256 of the new bytes of the note.
    pub revision: DigestHex,

    /// The slug of the note in the current release. Absent when the note is not published.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<Slug>,
}

impl EventPayload for DraftChanged {
    const EVENT_TYPE: &'static str = "sites.draft.changed.v1";
}

/// Whether a site approval is in force.
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
pub enum ApprovalState {
    /// The owner approved the manifest digest; it is the approved manifest of the site.
    Approved,
    /// The owner withdrew the approval; the site serves nothing.
    Withdrawn,
}

/// Payload of `platform.site_approval.changed.v1`: the owner approved or withdrew the manifest of
/// one site.
///
/// A fact. Platform writes it with the row of `identity.site_approvals` in one transaction, once
/// per change, and `ratatoskr-sites` keeps a read projection of it (workspace spec
/// `site-manifest`). A withdrawal repeats the digest, the capabilities and the hostname of the
/// approval it ends. The enclosing envelope carries the owner as `tenant_id` and the aggregate
/// `site:<site_id>`.
///
/// `Deserialize` is hand-written because `capabilities` must hold each capability once.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SiteApprovalChanged {
    /// The site whose approval changed.
    pub site_id: SiteId,

    /// The lowercase hex SHA-256 of the manifest bytes that the approval names.
    pub manifest_sha256: DigestHex,

    /// The capabilities of the approved manifest, each once.
    #[schemars(extend("uniqueItems" = true))]
    pub capabilities: Vec<SiteCapability>,

    /// The hostname of the approved manifest.
    pub hostname: Hostname,

    /// Whether the approval is now in force.
    pub state: ApprovalState,

    /// The repository identity covered by this approval. Absent only for legacy documents;
    /// its absence grants no deploy authority (workspace SITE-028).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_repository: Option<GitHubRepositoryBinding>,
}

impl EventPayload for SiteApprovalChanged {
    const EVENT_TYPE: &'static str = "platform.site_approval.changed.v1";
}

impl SiteApprovalChanged {
    /// Checks that `capabilities` holds each capability once. `Deserialize` calls this.
    ///
    /// # Errors
    ///
    /// [`SitesContractError::Duplicate`].
    pub fn validate(&self) -> Result<(), SitesContractError> {
        no_duplicate(
            "capabilities",
            self.capabilities
                .iter()
                .map(|capability| capability.as_str()),
        )
    }
}

/// The wire mirror of [`ReleasePublished`].
///
/// **Drift guard:** a field added to [`ReleasePublished`] and not to this mirror would be refused
/// as unknown on parse. Do not add a field to one of these structs alone.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleasePublishedWire {
    site_id: SiteId,
    release_id: DigestHex,
    publication_ids: Vec<PublicationId>,
    page_count: u32,
    published_at: WireTimestamp,
}

impl<'de> serde::Deserialize<'de> for ReleasePublished {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ReleasePublishedWire::deserialize(deserializer)?;
        let payload = Self {
            site_id: wire.site_id,
            release_id: wire.release_id,
            publication_ids: wire.publication_ids,
            page_count: wire.page_count,
            published_at: wire.published_at,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}

/// The wire mirror of [`SiteApprovalChanged`].
///
/// **Drift guard:** a field added to [`SiteApprovalChanged`] and not to this mirror would be
/// refused as unknown on parse. Do not add a field to one of these structs alone.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SiteApprovalChangedWire {
    site_id: SiteId,
    manifest_sha256: DigestHex,
    capabilities: Vec<SiteCapability>,
    hostname: Hostname,
    state: ApprovalState,
    #[serde(default)]
    github_repository: Option<GitHubRepositoryBinding>,
}

impl<'de> serde::Deserialize<'de> for SiteApprovalChanged {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = SiteApprovalChangedWire::deserialize(deserializer)?;
        let payload = Self {
            site_id: wire.site_id,
            manifest_sha256: wire.manifest_sha256,
            capabilities: wire.capabilities,
            hostname: wire.hostname,
            state: wire.state,
            github_repository: wire.github_repository,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}
