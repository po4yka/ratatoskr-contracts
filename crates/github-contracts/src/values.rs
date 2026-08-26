//! Immutable repository-revision values used by analysis request and terminal facts.

use ratatoskr_identifiers::{BlobRef, ContentDigest};

/// The only repository-analysis family available in this contract.
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
pub enum RepositoryAnalysisContract {
    /// The first repository-analysis family.
    RepositoryAnalysis,
}

/// Immutable metadata and README state of the repository input.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RepositoryAnalysisRevision {
    /// Digest of the normalized repository attributes observed by GitHub Catalog.
    pub attributes_digest: ContentDigest,
    /// README state for this exact repository revision.
    pub readme: ReadmeRevision,
}

/// README input state for repository analysis.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReadmeRevision {
    /// A content-addressed README is available to the authorized consumer.
    Present {
        /// Reference to the exact README bytes. The reference carries its content digest.
        content_ref: BlobRef,
    },
    /// No README bytes are an input for this revision.
    Absent {
        /// Why a README is absent, without leaking provider diagnostics or private content.
        reason: ReadmeAbsenceReason,
    },
}

/// Safe reasons why a README is absent from the immutable input.
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
pub enum ReadmeAbsenceReason {
    /// GitHub reported that the repository has no README at this revision.
    NotFound,
    /// Catalog policy did not authorize the README to leave the GitHub boundary.
    NotAuthorized,
    /// The README could not be preserved into a content-addressed reference.
    NotPreserved,
}

/// Safe terminal failure categories published by Knowledge.
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
pub enum AnalysisFailureCode {
    /// The immutable input cannot be resolved by the authorized analysis worker.
    SourceUnavailable,
    /// The request violates the repository-analysis contract.
    ContractInvalid,
    /// The analysis dependency was unavailable before an analysis result existed.
    DependencyUnavailable,
    /// The requested work was not authorized by Knowledge policy.
    NotAuthorized,
}
