//! Wire contracts for public Telegram channel subscriptions and grounded digest recaps.
//!
//! Provider credentials and post bodies are deliberately absent. Commands carry internal owner,
//! operation, run, immutable-manifest, and idempotency authority; terminal facts carry only safe
//! linkage, coverage, and failure classes.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod commands;
mod error;
mod manifest;
mod recap;
mod values;

pub use crate::commands::{
    ChannelDigestRunRequested, ChannelDigestScheduleOccurrenceRequested,
    ChannelDigestSubscriptionSetRequested,
};
pub use crate::error::ChannelDigestContractError;
pub use crate::manifest::{
    ChannelDigestManifest, ChannelDigestManifestSchema, ChannelDigestManifestSource,
    HEADER_DIGEST_RUN_ID, HEADER_MANIFEST_DIGEST, HEADER_OWNER_ID, MANIFEST_PATH_TEMPLATE,
    MAX_CHANNELS, MAX_CONTENT_BYTES, MAX_SOURCES, READY_PATH, manifest_path, sha256_hex,
};
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
