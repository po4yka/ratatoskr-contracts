//! The responses of the two deploy routes of a site.
//!
//! `POST /v1/sites/{site}/deploys` takes the bundle archive itself as its body
//! (`application/x-tar`, with `ratatoskr-bundle.json` at its root) and answers [`DeployStored`].
//! `POST /v1/sites/{site}/deploys/{commit}/activate` takes no body and answers
//! [`DeployActivated`]. The commit is the deploy id: a deploy is idempotent on the site and the
//! commit (workspace spec `site-bundles`).

use crate::bundle::CommitSha;
use crate::manifest::SiteId;
use crate::release::asset_size;

/// Contract `sites/deploy-stored.v1`: the answer of `POST /v1/sites/{site}/deploys`.
///
/// The bundle of `commit` is stored for the site and every file matched the manifest. Every other
/// member is refused.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct DeployStored {
    /// The site that the bundle is stored for.
    pub site_id: SiteId,

    /// The commit of the bundle, which is the deploy id.
    pub commit: CommitSha,

    /// The number of files of the bundle.
    pub file_count: u32,

    /// The total length of the file bytes of the bundle, at most 2^53 - 1.
    #[serde(deserialize_with = "asset_size")]
    #[schemars(range(max = 9_007_199_254_740_991_u64))]
    pub size_bytes: u64,

    /// `true` when this request stored the bundle; `false` when an earlier deploy of the same
    /// commit was already stored and is returned unchanged.
    pub created: bool,
}

/// Contract `sites/deploy-activated.v1`: the answer of
/// `POST /v1/sites/{site}/deploys/{commit}/activate`.
///
/// The deploy of `current_commit` is now the deploy that the gateway serves for the site. Every
/// other member is refused.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct DeployActivated {
    /// The site whose current deploy changed.
    pub site_id: SiteId,

    /// The commit whose deploy is now current.
    pub current_commit: CommitSha,

    /// The commit whose deploy was current before, kept so that activating it again needs no
    /// download. Absent when the site had no current deploy before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_commit: Option<CommitSha>,
}
