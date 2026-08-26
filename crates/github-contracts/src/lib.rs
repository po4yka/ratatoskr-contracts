//! Repository-analysis requests and terminal facts exchanged by GitHub Catalog and Knowledge.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod events;
mod values;

pub use crate::events::{
    RepositoryAnalysisCompleted, RepositoryAnalysisFailed, RepositoryAnalysisRequested,
};
pub use crate::values::{
    AnalysisFailureCode, ReadmeAbsenceReason, ReadmeRevision, RepositoryAnalysisContract,
    RepositoryAnalysisRevision,
};
