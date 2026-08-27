//! Repository-analysis requests and terminal facts exchanged by GitHub Catalog and Knowledge.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod events;
mod interactions;
mod values;

pub use crate::events::{
    RepositoryAnalysisCompleted, RepositoryAnalysisFailed, RepositoryAnalysisRequested,
};
pub use crate::interactions::{
    ConfirmationEvidenceRef, GitHubAccountRef, GitHubRepositoryNumericId, GitHubRepositoryUrl,
    RepositoryActionAggregate, RepositoryActionCapability, RepositoryActionFailureReason,
    RepositoryActionIdempotencyKey, RepositoryActionRefusalReason, RepositoryActionRequest,
    RepositoryActionResult, RepositoryActionSkipReason, RepositoryDesiredBackupOutcome,
    RepositoryInteractionContractError, RepositoryMetadataOutcome, RepositoryPreviewRequest,
    RepositoryPreviewResponse, RepositoryPreviewTarget, RepositoryProviderStarOutcome,
};
pub use crate::values::{
    AnalysisFailureCode, ReadmeAbsenceReason, ReadmeRevision, RepositoryAnalysisAttributes,
    RepositoryAnalysisContract, RepositoryAnalysisRevision, RepositoryDescription,
    RepositoryFullName, RepositoryLanguage,
};
