//! Sites contracts: the documents and messages of `ratatoskr-sites` (changeset SITE-025). The
//! cross-repository behaviour is defined in the workspace store: change `add-site-publications`,
//! specs `site-manifest`, `site-publications`, `site-release-privacy` and
//! `site-content-delivery`.
//!
//! # What is on the wire
//!
//! - [`SiteManifest`] — contract `sites/site-manifest.v1`, the file `ratatoskr-site.toml`: the
//!   site id, one hostname, the publications and the requested [`SiteCapability`] set.
//! - [`SiteId`] / [`PublicationId`] / [`Hostname`] — validated strings.
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

pub use crate::error::SitesContractError;
pub use crate::manifest::{Hostname, PublicationId, SiteCapability, SiteId, SiteManifest};
