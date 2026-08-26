//! Repository-analysis event payloads.

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{
    ContentDigest, EntityRef, Extensions, RepositoryAnalysisRequestId, RepositoryId, TenantRef,
    WireTimestamp,
};

use crate::values::{AnalysisFailureCode, RepositoryAnalysisContract, RepositoryAnalysisRevision};

/// Payload of `knowledge.repository_analysis.requested.v1`.
///
/// GitHub Catalog requests admission only. Knowledge owns budget admission, work execution,
/// retries and the analysis result; a request remains pending until it receives one terminal fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RepositoryAnalysisRequested {
    /// User whose repository policy caused this request.
    pub owner: TenantRef,

    /// Catalog-owned identity of the repository to analyse.
    pub repository_id: RepositoryId,

    /// Stable GitHub numeric repository identity, never derived from `owner/name`.
    pub github_repository_numeric_id: u64,

    /// Join key minted once for this accepted request.
    pub request_id: RepositoryAnalysisRequestId,

    /// Immutable input revision Knowledge must analyse.
    pub source_revision: RepositoryAnalysisRevision,

    /// Exact analysis family requested. This first contract admits no alternative major.
    pub requested_contract: RepositoryAnalysisContract,

    /// Catalog-generated idempotency key for the same repository revision and contract.
    pub idempotency_key: ContentDigest,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for RepositoryAnalysisRequested {
    const EVENT_TYPE: &'static str = "knowledge.repository_analysis.requested.v1";
}

/// Payload of `knowledge.repository_analysis.completed.v1` for a repository analysis.
///
/// The opaque result stays owned by Knowledge. GitHub Catalog records the reference after matching
/// this fact against the accepted request identity and immutable source revision.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RepositoryAnalysisCompleted {
    /// User who owns the analysed repository.
    pub owner: TenantRef,
    /// Catalog-owned repository identity.
    pub repository_id: RepositoryId,
    /// Stable GitHub numeric repository identity.
    pub github_repository_numeric_id: u64,
    /// Join key from the corresponding accepted request.
    pub request_id: RepositoryAnalysisRequestId,
    /// Exact immutable input revision that was analysed.
    pub source_revision: RepositoryAnalysisRevision,
    /// Opaque Knowledge-owned reference to the completed analysis result.
    pub analysis_result_ref: EntityRef,
    /// Instant Knowledge accepted the completed analysis result.
    pub completed_at: WireTimestamp,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for RepositoryAnalysisCompleted {
    const EVENT_TYPE: &'static str = "knowledge.repository_analysis.completed.v1";
}

/// Payload of `knowledge.repository_analysis.failed.v1` for a terminal analysis failure.
///
/// A retryable failure is terminal for this request identity; Knowledge may later admit a fresh
/// request under its own scheduling policy rather than leaving the Catalog's pending state stuck.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RepositoryAnalysisFailed {
    /// User who owns the repository whose analysis failed.
    pub owner: TenantRef,
    /// Catalog-owned repository identity.
    pub repository_id: RepositoryId,
    /// Stable GitHub numeric repository identity.
    pub github_repository_numeric_id: u64,
    /// Join key from the corresponding accepted request.
    pub request_id: RepositoryAnalysisRequestId,
    /// Exact immutable input revision that could not be analysed.
    pub source_revision: RepositoryAnalysisRevision,
    /// Closed, safe reason vocabulary; no provider diagnostic is carried here.
    pub failure_code: AnalysisFailureCode,
    /// Whether Knowledge considers a later, fresh request eligible for retry.
    pub retryable: bool,
    /// Instant Knowledge accepted the terminal failure.
    pub failed_at: WireTimestamp,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for RepositoryAnalysisFailed {
    const EVENT_TYPE: &'static str = "knowledge.repository_analysis.failed.v1";
}
