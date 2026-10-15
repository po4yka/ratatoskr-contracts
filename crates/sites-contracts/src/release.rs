//! The release document `sites/release.v1` and the names it shares with the page.

use std::collections::BTreeSet;

use ratatoskr_identifiers::{DigestHex, MediaType, wire_string_newtype};

use crate::error::SitesContractError;
use crate::manifest::{PublicationId, SiteId};

wire_string_newtype! {
    /// The slug of one page: unique in its release, and the last part of
    /// `/v1/sites/{site}/content/pages/{slug}`.
    ///
    /// Segments of lower-case ASCII letters, digits and `-`, separated by `/`, at most 1024
    /// characters. The release builder takes the frontmatter key `slug`, or else the vault path
    /// without `.md`, lower-cased, with each run of other characters than ASCII letters, digits
    /// and `/` replaced by one `-` (workspace spec `site-publications`).
    pub struct Slug {
        pattern  = r"^[a-z0-9-]+(?:/[a-z0-9-]+)*$",
        max_len  = 1024,
        examples = ["handbook/start", "notes/a-b"],
    }
}

wire_string_newtype! {
    /// The title of a page or of a navigation entry, as the release shows it.
    ///
    /// 1 to 255 characters with no C0 or DEL control character. The pattern carries the
    /// 255-character bound; the length guard is 1020 UTF-8 bytes, the most 255 characters can
    /// take.
    pub struct PageTitle {
        pattern  = r"^[^\x00-\x1f\x7f]{1,255}$",
        max_len  = 1020,
        examples = ["Start here", "Handbook"],
    }
}

/// Contract `sites/release.v1`: one immutable release of a site, as the delivery route
/// `/v1/sites/{site}/content/release` serves it.
///
/// The release id is the lowercase hex SHA-256 of the RFC 8785 form of this document (workspace
/// design D4), so the document does not hold its own id. Every list is sorted and holds each item
/// once, so one content gives one id. The document names pages by slug and assets by SHA-256, never
/// by vault path, and every other member is refused.
///
/// `Deserialize` is hand-written because the order rules span a list and a navigation entry must
/// name a page of the release. It parses a private mirror struct and then calls
/// [`Release::validate`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Release {
    /// The site that the release belongs to.
    pub site_id: SiteId,

    /// The publications whose pages the release holds, in ascending order, each once.
    #[schemars(extend("uniqueItems" = true))]
    pub publication_ids: Vec<PublicationId>,

    /// Every page of the release, in ascending order of slug, each slug once.
    pub pages: Vec<ReleasePage>,

    /// Every asset that a page of the release uses, in ascending order of SHA-256, each once.
    pub assets: Vec<ReleaseAsset>,

    /// The navigation tree of the site, in the order the site shows it.
    pub navigation: Vec<NavigationEntry>,
}

/// One page of a [`Release`].
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReleasePage {
    /// The slug of the page.
    pub slug: Slug,

    /// The title of the page.
    pub title: PageTitle,

    /// The lowercase hex SHA-256 of the `sites/page.v1` bytes that the delivery route serves for
    /// this slug.
    pub page_sha256: DigestHex,
}

/// One asset of a [`Release`].
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReleaseAsset {
    /// The lowercase hex SHA-256 of the asset bytes; the asset URL is `assets/<sha256>`.
    pub sha256: DigestHex,

    /// The media type that the delivery route serves the bytes with.
    pub media_type: MediaType,

    /// The length of the asset bytes, at most 2^53 - 1, the largest integer that a JavaScript
    /// number holds exactly.
    #[serde(deserialize_with = "asset_size")]
    #[schemars(range(max = 9_007_199_254_740_991_u64))]
    pub size_bytes: u64,
}

/// One entry of the navigation tree of a [`Release`].
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct NavigationEntry {
    /// The text that the site shows for the entry.
    pub title: PageTitle,

    /// The page that the entry opens, which is a page of the release. Absent for an entry that
    /// only groups its children.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<Slug>,

    /// The entries under this one, in the order the site shows them.
    pub children: Vec<NavigationEntry>,
}

/// The largest asset size, 2^53 - 1: the largest integer that a JavaScript number holds exactly.
pub const MAX_ASSET_SIZE_BYTES: u64 = 9_007_199_254_740_991;

/// Refuses an asset or bundle file size above [`MAX_ASSET_SIZE_BYTES`].
pub(crate) fn asset_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let size = <u64 as serde::Deserialize>::deserialize(deserializer)?;
    if size <= MAX_ASSET_SIZE_BYTES {
        Ok(size)
    } else {
        Err(serde::de::Error::custom(format!(
            "size_bytes must be at most {MAX_ASSET_SIZE_BYTES}, found {size}"
        )))
    }
}

impl Release {
    /// Checks the order rules and the navigation. `Deserialize` calls this; the release builder
    /// calls it before it hashes a release.
    ///
    /// # Errors
    ///
    /// [`SitesContractError::Unsorted`] and [`SitesContractError::NavigationSlugNotInRelease`].
    pub fn validate(&self) -> Result<(), SitesContractError> {
        strictly_ascending(
            "publication_ids",
            self.publication_ids.iter().map(PublicationId::as_str),
        )?;
        strictly_ascending("pages", self.pages.iter().map(|page| page.slug.as_str()))?;
        strictly_ascending(
            "assets",
            self.assets.iter().map(|asset| asset.sha256.as_str()),
        )?;
        let slugs: BTreeSet<&str> = self.pages.iter().map(|page| page.slug.as_str()).collect();
        let mut pending: Vec<&NavigationEntry> = self.navigation.iter().collect();
        while let Some(entry) = pending.pop() {
            if let Some(slug) = &entry.slug
                && !slugs.contains(slug.as_str())
            {
                return Err(SitesContractError::NavigationSlugNotInRelease {
                    slug: slug.as_str().to_owned(),
                });
            }
            pending.extend(entry.children.iter());
        }
        Ok(())
    }
}

/// Refuses the first item of `items` that does not sort after the item before it, so the list is
/// sorted and holds each item once.
pub(crate) fn strictly_ascending<'a>(
    list: &'static str,
    items: impl Iterator<Item = &'a str>,
) -> Result<(), SitesContractError> {
    let mut previous: Option<&str> = None;
    for item in items {
        if previous.is_some_and(|before| before >= item) {
            return Err(SitesContractError::Unsorted {
                list,
                item: item.to_owned(),
            });
        }
        previous = Some(item);
    }
    Ok(())
}

/// The wire mirror of [`Release`], parsed before the order rules run.
///
/// **Drift guard:** a field added to [`Release`] and not to this mirror would be refused as
/// unknown on parse, and the release fixture fails at once. Do not add a field to one of these
/// structs alone.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseWire {
    site_id: SiteId,
    publication_ids: Vec<PublicationId>,
    pages: Vec<ReleasePage>,
    assets: Vec<ReleaseAsset>,
    navigation: Vec<NavigationEntry>,
}

impl<'de> serde::Deserialize<'de> for Release {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ReleaseWire::deserialize(deserializer)?;
        let release = Self {
            site_id: wire.site_id,
            publication_ids: wire.publication_ids,
            pages: wire.pages,
            assets: wire.assets,
            navigation: wire.navigation,
        };
        release.validate().map_err(serde::de::Error::custom)?;
        Ok(release)
    }
}
