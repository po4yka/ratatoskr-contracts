//! Repository identity for the owner-approved site build (SITE-028).

use ratatoskr_identifiers::{DigestHex, wire_string_newtype};

use crate::bundle::CommitSha;

wire_string_newtype! {
    /// The canonical lower-case GitHub repository in an approval, as `owner/name`.
    ///
    /// It names one public GitHub repository. The numeric id also has to match its signed identity.
    pub struct GitHubRepositoryName {
        pattern = r"^[a-z0-9][a-z0-9-]{0,38}/[a-z0-9._-]{1,100}$",
        max_len = 140,
        examples = ["owner/docs-site"],
    }
}

wire_string_newtype! {
    /// The immutable positive GitHub repository id, in decimal with no leading zero.
    ///
    /// A string keeps the exact integer in generated JavaScript clients. This is public identity,
    /// not a credential. Verification compares it with the signed certificate claim.
    pub struct GitHubRepositoryId {
        pattern = r"^[1-9][0-9]{0,19}$",
        max_len = 20,
        examples = ["1420131804"],
    }
}

/// One repository whose signed builds the owner approves for a site.
///
/// Both fields are part of the manifest digest and the approval event. A changed name or id needs
/// a new owner approval. Absence in a legacy manifest grants no deploy authority.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct GitHubRepositoryBinding {
    /// The canonical public GitHub repository name that the owner approves.
    pub repository: GitHubRepositoryName,
    /// Its immutable GitHub repository id; a reused name does not keep this identity.
    pub repository_id: GitHubRepositoryId,
}

/// The immutable artifact request of the first-version deploy route (SITE-028).
///
/// The proof is an untrusted public Sigstore bundle. Its presence grants no trust: the server
/// verifies its signature, subject, certificate identity and policy against the fetched bytes.
/// There is no client-selected host, repository path, trust root or raw archive in this body.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct DeployRequest {
    /// The full source commit that the signed certificate must name.
    pub commit: CommitSha,
    /// The lower-case hex SHA-256 of the OCI manifest, without an algorithm prefix.
    /// The server adds `sha256:` only when it constructs the fixed registry reference.
    pub artifact_sha256: DigestHex,
    /// An untrusted public proof object. Transport bounds its bytes; verification checks its
    /// cryptographic meaning. It carries no account or registry credential.
    pub attestation_bundle: std::collections::BTreeMap<String, serde_json::Value>,
}
