//! Wire contracts for public Telegram channel subscriptions and grounded digest recaps.
//!
//! Provider credentials and post bodies are deliberately absent. Commands carry internal owner,
//! operation, run, immutable-manifest, and idempotency authority; terminal facts carry only safe
//! linkage, coverage, and failure classes.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod commands;
mod error;
mod recap;
mod values;

pub use crate::commands::{ChannelDigestRunRequested, ChannelDigestSubscriptionSetRequested};
pub use crate::error::ChannelDigestContractError;
pub use crate::recap::{
    ChannelDigestAnalysisContract, ChannelDigestAnalysisFamily, ChannelDigestRecapCoverage,
    ChannelDigestRecapFailureCode, KnowledgeChannelDigestRecapCompleted,
    KnowledgeChannelDigestRecapFailed, KnowledgeChannelDigestRecapRequested,
};
pub use crate::values::{
    ChannelDigestIdempotencyKey, ChannelDigestManifestRef, ChannelDigestResultId,
    ChannelDigestResultRef, ChannelDigestRunId, ChannelDigestRunTrigger,
    ChannelDigestSubscriptionId, ChannelUsername, DigestChannelCount, DigestOccurrenceRef,
    DigestScheduleRef, DigestSourceCount, DigestWindow, KnowledgeAnalysisRef, OutputLanguage,
    SubscriptionDesiredState,
};
