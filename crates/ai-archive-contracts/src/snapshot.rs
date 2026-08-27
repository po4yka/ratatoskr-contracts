//! [`AiArchiveSnapshot`], the import head ([`AiArchiveImport`]) and the completeness report.

use ratatoskr_error_contracts::WarningEnvelope;
use ratatoskr_identifiers::{
    AiArchiveId, AiConversationId, BlobRef, EntityLocalId, Extensions, SafeMessage, TenantRef,
    WireTimestamp, wire_string_newtype,
};

use crate::error::AiArchiveContractError;
use crate::graph::{AiConversation, AiProject};
use crate::tokens::{AiProvider, ParserName, ParserVersion};

wire_string_newtype! {
    /// What kind of hole a [`AiGap`] records, e.g. `missing_file`, `truncated_conversation`,
    /// `undecodable_record`.
    ///
    /// **Open on purpose**: providers find new ways to be incomplete, and a consumer must be
    /// able to carry a gap it does not classify rather than drop it. Branch on equality with
    /// known kinds; treat everything else generically.
    ///
    /// [`AiGap`]: crate::AiGap
    pub struct AiGapKind {
        pattern  = r"^[a-z][a-z0-9_]{0,31}$",
        max_len  = 32,
        examples = ["missing_file", "truncated_conversation", "undecodable_record"],
    }
}

/// Whether one import obtained the whole export, stated per `docs/ARCHITECTURE.md` S8.3.
///
/// **Closed on purpose**: indexing depth, retention and re-import scheduling hang off this
/// state, so an unrecognized value must stop processing rather than be read as "whole enough".
/// Completeness is evidence-based: a parser may not mark an import `complete` merely because it
/// parsed every file it knows about.
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
pub enum AiArchiveCompleteness {
    /// Every record parsed and every referenced asset was stored.
    Complete,
    /// Every conversation record parsed; some assets are missing or were refused.
    ConversationsComplete,
    /// Some records did not parse or were not present; the gaps say which.
    StructurallyPartial,
    /// Records parsed but asset storage was incomplete.
    AssetsPartial,
    /// The parser could not determine what the export contained at all.
    Unknown,
    /// The export failed an integrity check before parsing could be trusted.
    FailedValidation,
}

impl AiArchiveCompleteness {
    /// `true` when the state asserts the import is whole. Only this state may carry no gap.
    #[must_use]
    pub fn is_complete(self) -> bool {
        self == Self::Complete
    }
}

/// One named hole in an import: what is missing, and where it was noticed.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct AiGap {
    /// What kind of hole this is. Open token.
    pub gap_kind: AiGapKind,

    /// User-safe explanation of the hole, stable for display and triage.
    pub detail: SafeMessage,

    /// The provider id of the affected record, when one record is identifiable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<EntityLocalId>,

    /// How many records this single gap entry stands for, when the parser batches identical
    /// holes. Absent means one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affected_count: Option<u32>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// The evidence-based completeness report of one import.
///
/// `conversation_count` and `gap_count` are **verifiable**: they are checked against the
/// payload wherever the payload carries the nodes they count (invariant A2/A3). The imported
/// event's head has no tree attached, so there those two counts are producer-asserted like the
/// other two; on a snapshot they are enforced.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AiCompletenessReport {
    /// Whether the import is whole. Closed vocabulary.
    pub completeness: AiArchiveCompleteness,

    /// Conversations in this import.
    pub conversation_count: u32,

    /// Messages across all conversations of this import.
    pub message_count: u32,

    /// Asset references stored during the import (files, artifacts, images).
    pub asset_count: u32,

    /// Gap entries carried in `gaps`. Verified against `gaps` length.
    pub gap_count: u32,

    /// Every hole the parser found, each naming what is missing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<AiGap>,
}

/// The privacy-safe import outcome that an operation result may expose to a client.
///
/// It carries only the immutable archive identity, provider, exact completeness classification,
/// and aggregate counts; gap and warning details remain on the owning archive report.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AiArchiveOperationSummary {
    /// The immutable archive import produced by the operation.
    pub ai_archive_id: AiArchiveId,

    /// The provider whose export the archive represents.
    pub provider: AiProvider,

    /// The producer's exact evidence-based import completeness classification.
    pub completeness: AiArchiveCompleteness,

    /// Number of normalized conversations in this import.
    pub conversation_count: u32,

    /// Number of normalized messages in this import.
    pub message_count: u32,

    /// Number of stored asset references in this import.
    pub asset_count: u32,

    /// Number of known archive-level gaps in this import.
    pub gap_count: u32,

    /// Number of non-gap warnings produced while importing this archive.
    pub warning_count: u32,
}

/// The head of one import: identity, owner, immutable evidence, timing, stamps, completeness.
///
/// This type is both the head member of [`AiArchiveSnapshot`] and the payload of
/// `ai_archive.archive.imported.v1`, so the event and the snapshot cannot disagree about what
/// an import claims.
///
/// # Cross-field invariant
///
/// Re-checkable through [`Self::validate`]:
///
/// - **A1** every completeness state other than `complete` requires at least one gap naming
///   what is missing. The rule is asymmetric on purpose: a complete import carries zero gaps by
///   definition — non-blocking problems travel as warnings instead, exactly like the social
///   snapshot's complete-capture rule.
///
/// `Deserialize` is hand-written below because invariant A1 is cross-field.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct AiArchiveImport {
    /// Ratatoskr's own identity for this import, minted once per parse of one export. Bare
    /// canonical UUID on the wire.
    pub ai_archive_id: AiArchiveId,

    /// The provider whose account was exported. Open token.
    pub provider: AiProvider,

    /// The user whose archive was imported.
    pub owner: TenantRef,

    /// Reference to the immutable raw provider export backing every node of this import. The
    /// preservation channel of last resort: re-parsing these bytes must remain possible even
    /// where this contract cannot represent something the export sent.
    pub source_export: BlobRef,

    /// Instant the producing service finished importing the export into normalized nodes.
    /// Observed: the producer's clock, not the provider's.
    pub imported_at: WireTimestamp,

    /// Which parser produced the normalized nodes of this import.
    pub parser_name: ParserName,

    /// The build of [`AiArchiveImport::parser_name`].
    pub parser_version: ParserVersion,

    /// The evidence-based completeness report.
    pub completeness_report: AiCompletenessReport,

    /// Non-blocking problems recorded during import that did not create a gap.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<WarningEnvelope>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl AiArchiveImport {
    /// Re-checks every cross-field invariant of the head alone: state-versus-gaps (A1). The
    /// count invariants need the node tree and are checked by [`AiArchiveSnapshot::validate`].
    ///
    /// # Errors
    ///
    /// [`AiArchiveContractError::IncompleteWithoutGap`] for A1.
    pub fn validate(&self) -> Result<(), AiArchiveContractError> {
        if !self.completeness_report.completeness.is_complete()
            && self.completeness_report.gaps.is_empty()
        {
            return Err(AiArchiveContractError::IncompleteWithoutGap);
        }
        Ok(())
    }
}

impl From<&AiArchiveImport> for AiArchiveOperationSummary {
    fn from(import: &AiArchiveImport) -> Self {
        Self {
            ai_archive_id: import.ai_archive_id,
            provider: import.provider.clone(),
            completeness: import.completeness_report.completeness,
            conversation_count: import.completeness_report.conversation_count,
            message_count: import.completeness_report.message_count,
            asset_count: import.completeness_report.asset_count,
            gap_count: import.completeness_report.gap_count,
            warning_count: u32::try_from(import.warnings.len()).unwrap_or(u32::MAX),
        }
    }
}

/// One provider export turned into its whole normalized graph: head, projects, conversations.
///
/// This is the canonical normalized form of an import — the shape a bulk load consumes and a
/// re-parse verification compares against. Events publish slices of it (the head alone, or one
/// conversation), never a divergent copy of it.
///
/// `Deserialize` is hand-written below because invariants A2 and A3 are cross-field.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct AiArchiveSnapshot {
    /// The import head.
    pub import: AiArchiveImport,

    /// Every project of the import, empty when the provider export has none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<AiProject>,

    /// Every conversation of the import, empty only when nothing parsed (which the
    /// completeness report then explains).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conversations: Vec<AiConversation>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl AiArchiveSnapshot {
    /// Re-checks every cross-field invariant: the head's own (A1) plus the verifiable counts
    /// (A2, A3).
    ///
    /// # Errors
    ///
    /// [`AiArchiveContractError::IncompleteWithoutGap`] for A1,
    /// [`AiArchiveContractError::ConversationCountMismatch`] for A2,
    /// [`AiArchiveContractError::GapCountMismatch`] for A3.
    pub fn validate(&self) -> Result<(), AiArchiveContractError> {
        self.import.validate()?;
        let report = &self.import.completeness_report;

        let actual_conversations = u32::try_from(self.conversations.len()).unwrap_or(u32::MAX);
        if report.conversation_count != actual_conversations {
            return Err(AiArchiveContractError::ConversationCountMismatch {
                declared: report.conversation_count,
                actual: actual_conversations,
            });
        }

        let actual_gaps = u32::try_from(report.gaps.len()).unwrap_or(u32::MAX);
        if report.gap_count != actual_gaps {
            return Err(AiArchiveContractError::GapCountMismatch {
                declared: report.gap_count,
                actual: actual_gaps,
            });
        }

        Ok(())
    }

    /// The identity of the import this snapshot belongs to.
    #[must_use]
    pub fn archive_id(&self) -> &AiArchiveId {
        &self.import.ai_archive_id
    }

    /// Every conversation carrying the given Ratatoskr identity.
    pub fn conversation_by_id(
        &self,
        ai_conversation_id: &AiConversationId,
    ) -> impl Iterator<Item = &AiConversation> {
        self.conversations
            .iter()
            .filter(move |conversation| &conversation.ai_conversation_id == ai_conversation_id)
    }
}

/// The wire mirror of [`AiArchiveSnapshot`], parsed before the cross-field invariants run.
///
/// Serde offers no post-deserialization validation hook, so the public type's `Deserialize` is
/// hand-written and delegates field-by-field work here. **Drift guard:** a field added to
/// [`AiArchiveSnapshot`] and not to this mirror is silently dropped on parse; test
/// `snapshot_roundtrip.rs` fails the moment that happens. Do not add a field to one alone.
#[derive(Debug, serde::Deserialize)]
struct AiArchiveSnapshotWire {
    import: AiArchiveImport,
    #[serde(default)]
    projects: Vec<AiProject>,
    #[serde(default)]
    conversations: Vec<AiConversation>,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for AiArchiveSnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = AiArchiveSnapshotWire::deserialize(deserializer)?;
        let snapshot = Self {
            import: wire.import,
            projects: wire.projects,
            conversations: wire.conversations,
            extensions: wire.extensions,
        };
        snapshot.validate().map_err(serde::de::Error::custom)?;
        Ok(snapshot)
    }
}

/// The wire mirror of [`AiArchiveImport`]: same shape, plus invariant A1 at parse time.
///
/// The mirror exists because the head is also an event payload; its `Deserialize` must enforce
/// the invariant without dragging the whole snapshot along. Drift guard: test
/// `snapshot_roundtrip.rs` round-trips the head through both paths.
#[derive(Debug, serde::Deserialize)]
struct AiArchiveImportWire {
    ai_archive_id: AiArchiveId,
    provider: AiProvider,
    owner: TenantRef,
    source_export: BlobRef,
    imported_at: WireTimestamp,
    parser_name: ParserName,
    parser_version: ParserVersion,
    completeness_report: AiCompletenessReport,
    #[serde(default)]
    warnings: Vec<WarningEnvelope>,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for AiArchiveImport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = AiArchiveImportWire::deserialize(deserializer)?;
        let import = Self {
            ai_archive_id: wire.ai_archive_id,
            provider: wire.provider,
            owner: wire.owner,
            source_export: wire.source_export,
            imported_at: wire.imported_at,
            parser_name: wire.parser_name,
            parser_version: wire.parser_version,
            completeness_report: wire.completeness_report,
            warnings: wire.warnings,
            extensions: wire.extensions,
        };
        import.validate().map_err(serde::de::Error::custom)?;
        Ok(import)
    }
}
