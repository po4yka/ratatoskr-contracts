//! Normalized AI-archive contracts: what `ratatoskr-chatgpt` and `ratatoskr-claude` publish when
//! an official account export becomes immutable evidence and normalized conversation graphs, and
//! what `ratatoskr-knowledge` consumes to index them.
//!
//! # What is on the wire
//!
//! - [`AiArchiveSnapshot`] — the whole normalized tree of one import: the import head plus every
//!   project and conversation.
//! - [`AiArchiveImport`] — the import head alone: identity, owner, raw-export evidence, timing,
//!   parser stamps, completeness report. Also the payload of `ai_archive.archive.imported.v1`.
//! - [`AiProject`] / [`AiConversation`] / [`AiMessage`] — graph nodes. Conversations reference
//!   their optional project; messages carry optional parent references, so branches and
//!   regenerated answers survive normalization.
//! - [`AiContentPart`] — one shared part grammar for both providers, with an unknown-part channel
//!   that preserves unrecognized provider records verbatim through normalization and re-export.
//! - [`AiCompletenessReport`] — closed completeness vocabulary, verifiable counts, structured gaps.
//! - [`AiConversationAdded`] / [`AiConversationUpdated`] — the per-conversation events, each
//!   carrying the whole conversation so replay converges without earlier events.
//! - [`platform_receipt`] — not a registered wire message: the literals of the loopback archive
//!   receipt hop between Platform's Edge and the `ratatoskr-chatgpt` and `ratatoskr-claude` receivers, the capability
//!   probe both sides spell, and the single warning of an incomplete import.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod asset;
mod content_part;
mod error;
mod events;
mod graph;
pub mod platform_receipt;
mod snapshot;
mod tokens;
mod values;

pub use crate::asset::{
    AiAsset, AiAssetKind, AiCitation, AiSourceUrl, AiToolCall, AiToolName, AiToolOutcome,
    AiToolResult,
};
pub use crate::content_part::AiContentPart;
pub use crate::error::AiArchiveContractError;
pub use crate::events::{
    AiArchiveAnalysisCompleted, AiArchiveProvenance, AiArchiveSubject, AiArchiveTombstone,
    AiArchiveTombstoneParser, AiArchiveTombstoneReason, AiArchiveTombstoneSubject, AiArtifact,
    AiArtifactAdded, AiArtifactUpdated, AiConversationAdded, AiConversationUpdated, AiProjectAdded,
    AiProjectUpdated,
};
pub use crate::graph::{AiAuthorRole, AiConversation, AiMessage, AiModelName, AiProject};
pub use crate::snapshot::{
    AiArchiveCompleteness, AiArchiveImport, AiArchiveOperationSummary, AiArchiveSnapshot,
    AiCompletenessReport, AiGap, AiGapKind,
};
pub use crate::tokens::{AiProvider, ParserName, ParserVersion};
pub use crate::values::{AiFileName, AiText, AiTitle};
