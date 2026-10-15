//! Sites contracts: the documents and messages of `ratatoskr-sites` (changeset SITE-025). The
//! cross-repository behaviour is defined in the workspace store: change `add-site-publications`,
//! specs `site-manifest`, `site-publications`, `site-release-privacy` and
//! `site-content-delivery`.
//!
//! # What is on the wire
//!
//! - [`SiteManifest`] — contract `sites/site-manifest.v1`, the file `ratatoskr-site.toml`: the
//!   site id, one hostname, the publications and the requested [`SiteCapability`] set.
//! - [`Release`] — contract `sites/release.v1`: one immutable release of a site, its pages by slug,
//!   its assets by SHA-256 and its navigation. The release id is the SHA-256 of its RFC 8785 form.
//! - [`Page`] — contract `sites/page.v1`: one page, with its body as an mdast-compatible tree
//!   ([`PageRoot`], [`PageNode`], with `callout`, `wikiLink` and `embed`) and as sanitised HTML.
//! - [`ReleaseBuildRequested`] — payload of the command `sites.release.build_requested.v1`: Edge
//!   asks `ratatoskr-sites` to build a release of one site.
//! - [`ReleasePublished`] — payload of `sites.release.published.v1`: a new release became
//!   current. No title, slug, path or content.
//! - [`SiteApprovalChanged`] — payload of `platform.site_approval.changed.v1` (contract
//!   `platform/site-approval-changed.v1`): Platform approved or withdrew a manifest digest
//!   ([`ApprovalState`]).
//! - [`SiteId`] / [`PublicationId`] / [`Hostname`] / [`Slug`] / [`PageTitle`] / [`Anchor`] /
//!   [`FrontmatterKey`] / [`CalloutKind`] / [`LinkUrl`] / [`ImageUrl`] — validated strings.
//!
//! # What is deliberately absent
//!
//! No vault path, no note that a publication does not select, no credential. Every type refuses a
//! member it does not declare (`unknown_policy = "reject"`), so nothing can ride along in an
//! unknown field.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod manifest;
mod messages;
mod page;
mod release;

pub use crate::error::SitesContractError;
pub use crate::manifest::{Hostname, PublicationId, SiteCapability, SiteId, SiteManifest};
pub use crate::messages::{
    ApprovalState, ReleaseBuildRequested, ReleasePublished, SiteApprovalChanged,
};
pub use crate::page::{
    Anchor, AssetRef, CalloutFold, CalloutKind, EmbedTarget, FrontmatterKey, ImageUrl, LinkUrl,
    Page, PageNode, PageRoot, PageRootType, TableAlign,
};
pub use crate::release::{
    MAX_ASSET_SIZE_BYTES, NavigationEntry, PageTitle, Release, ReleaseAsset, ReleasePage, Slug,
};
