//! The site manifest `ratatoskr-site.toml` and the vocabulary it uses.

use ratatoskr_identifiers::wire_string_newtype;

use crate::error::SitesContractError;

wire_string_newtype! {
    /// The id of one site: 3 to 40 lower-case ASCII letters, digits and `-`.
    ///
    /// The grammar is the `site:<site id>` form of `WriteOrigin` in `ratatoskr-notes-contracts`.
    /// The owner chooses the id, and it appears in every route under `/v1/sites/{site}/`.
    pub struct SiteId {
        pattern  = r"^[a-z0-9-]{3,40}$",
        max_len  = 40,
        examples = ["docs", "garden-2026"],
    }
}

wire_string_newtype! {
    /// The id of one publication of a site: 3 to 40 lower-case ASCII letters, digits and `-`.
    ///
    /// The owner chooses the id and writes it in the manifest and in the path of
    /// `PUT /v1/sites/{site}/publications/{publication}`.
    pub struct PublicationId {
        pattern  = r"^[a-z0-9-]{3,40}$",
        max_len  = 40,
        examples = ["handbook", "garden-notes"],
    }
}

wire_string_newtype! {
    /// The DNS hostname that serves a site, in lower case, without a scheme, a port or a final dot.
    ///
    /// Two or more labels separated by `.`; each label has 1 to 63 lower-case ASCII letters,
    /// digits and `-`, and does not start or end with `-`. At most 253 characters.
    pub struct Hostname {
        pattern  = r"^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)+$",
        max_len  = 253,
        examples = ["docs.example.test", "garden.example.test"],
    }
}

/// One capability that a site manifest can request.
///
/// **Closed on purpose** (workspace spec `site-manifest`, "Capabilities are a closed set with a
/// fixed audience"): a manifest that names any other value is refused. The first four are
/// available to an anonymous visitor; the other four only to a `site_owner` session.
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
#[non_exhaustive]
pub enum SiteCapability {
    /// Read the pages and the navigation of the current release. Anyone.
    #[serde(rename = "pages.read")]
    PagesRead,
    /// Read the assets of the current release. Anyone.
    #[serde(rename = "assets.read")]
    AssetsRead,
    /// Full-text search over the current release only. Anyone.
    #[serde(rename = "search.publication")]
    SearchPublication,
    /// Receive `release.published` events over SSE. Anyone.
    #[serde(rename = "live.release")]
    LiveRelease,
    /// Read unpublished changes of the site's publications. Owner session only.
    #[serde(rename = "draft.read")]
    DraftRead,
    /// Write the source notes of the site's publications. Owner session only.
    #[serde(rename = "notes.edit")]
    NotesEdit,
    /// Read the owner's knowledge library. Owner session only.
    #[serde(rename = "library.read")]
    LibraryRead,
    /// Search across the owner's knowledge, not only the publication. Owner session only.
    #[serde(rename = "knowledge.search")]
    KnowledgeSearch,
}

impl SiteCapability {
    /// The wire word for this capability.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PagesRead => "pages.read",
            Self::AssetsRead => "assets.read",
            Self::SearchPublication => "search.publication",
            Self::LiveRelease => "live.release",
            Self::DraftRead => "draft.read",
            Self::NotesEdit => "notes.edit",
            Self::LibraryRead => "library.read",
            Self::KnowledgeSearch => "knowledge.search",
        }
    }

    /// `true` for the four capabilities that an anonymous visitor can use; `false` for the four
    /// that only a `site_owner` session can use.
    #[must_use]
    pub fn is_public(self) -> bool {
        matches!(
            self,
            Self::PagesRead | Self::AssetsRead | Self::SearchPublication | Self::LiveRelease
        )
    }
}

/// Contract `sites/site-manifest.v1`: the file `ratatoskr-site.toml` that a site repository
/// carries.
///
/// It names the site, exactly one hostname, the publications that the site reads and the
/// capabilities that it requests. The sites service stores every valid submission by the SHA-256
/// of its bytes, and Platform approves one digest; the effective set of a site is defined by the
/// workspace spec `site-manifest`, not by this type. Every other key is refused.
///
/// `Deserialize` is hand-written because two rules span a list: `publications` names at least one
/// publication, and neither list names an item twice. It parses a private mirror struct and then
/// calls [`SiteManifest::validate`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SiteManifest {
    /// The site that this manifest describes.
    pub site_id: SiteId,

    /// The one hostname that serves the site.
    pub hostname: Hostname,

    /// The publications that the site reads; at least one, each once.
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub publications: Vec<PublicationId>,

    /// The capabilities that the site requests, each once. A request grants nothing until the
    /// owner approves the manifest in Platform.
    #[schemars(extend("uniqueItems" = true))]
    pub capabilities: Vec<SiteCapability>,
}

impl SiteManifest {
    /// Checks the rules that span a list. `Deserialize` calls this; a producer that builds a
    /// manifest calls it before it writes one.
    ///
    /// # Errors
    ///
    /// [`SitesContractError::NoPublication`] and [`SitesContractError::Duplicate`].
    pub fn validate(&self) -> Result<(), SitesContractError> {
        if self.publications.is_empty() {
            return Err(SitesContractError::NoPublication);
        }
        no_duplicate(
            "publications",
            self.publications.iter().map(PublicationId::as_str),
        )?;
        no_duplicate(
            "capabilities",
            self.capabilities
                .iter()
                .map(|capability| capability.as_str()),
        )
    }
}

/// Refuses the first item that `items` yields twice.
pub(crate) fn no_duplicate<'a>(
    list: &'static str,
    items: impl Iterator<Item = &'a str>,
) -> Result<(), SitesContractError> {
    let mut seen = std::collections::BTreeSet::new();
    for item in items {
        if !seen.insert(item) {
            return Err(SitesContractError::Duplicate {
                list,
                item: item.to_owned(),
            });
        }
    }
    Ok(())
}

/// The wire mirror of [`SiteManifest`], parsed before the list rules run.
///
/// **Drift guard:** a field added to [`SiteManifest`] and not to this mirror would be refused as
/// unknown on parse, and the manifest fixtures fail at once. Do not add a field to one of these
/// structs alone.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SiteManifestWire {
    site_id: SiteId,
    hostname: Hostname,
    publications: Vec<PublicationId>,
    capabilities: Vec<SiteCapability>,
}

impl<'de> serde::Deserialize<'de> for SiteManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = SiteManifestWire::deserialize(deserializer)?;
        let manifest = Self {
            site_id: wire.site_id,
            hostname: wire.hostname,
            publications: wire.publications,
            capabilities: wire.capabilities,
        };
        manifest.validate().map_err(serde::de::Error::custom)?;
        Ok(manifest)
    }
}
