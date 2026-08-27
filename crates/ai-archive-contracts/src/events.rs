//! The three AI-archive event payloads.

use ratatoskr_event_envelope::EventPayload;

use crate::asset::AiAssetKind;
use crate::error::AiArchiveContractError;
use crate::graph::{AiConversation, AiProject};
use crate::snapshot::AiArchiveImport;
use crate::tokens::{AiProvider, ParserName, ParserVersion};
use crate::values::AiTitle;
use ratatoskr_identifiers::{
    AiArchiveId, AiConversationId, AiProjectId, BlobRef, ContentDigest, EntityLocalId, EntityRef,
    Extensions, TenantRef, WireTimestamp, wire_string_newtype,
};

wire_string_newtype! {
    /// An authoritative reason why an AI-archive subject was tombstoned.
    pub struct AiArchiveTombstoneReason {
        pattern = r"^(provider_deletion_event|compliance_event|reconciliation_policy|user_requested)$",
        max_len = 32,
        examples = [
            "provider_deletion_event",
            "compliance_event",
            "reconciliation_policy",
            "user_requested",
        ],
    }
}

/// The archive record made unavailable by an authoritative tombstone fact.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "subject_kind", rename_all = "snake_case")]
pub enum AiArchiveTombstoneSubject {
    /// The entire archive import named by the enclosing `ai_archive_id`.
    Archive,
    /// One conversation belonging to the enclosing archive import.
    Conversation {
        /// Ratatoskr identity of the tombstoned conversation.
        ai_conversation_id: AiConversationId,
    },
    /// One project belonging to the enclosing archive import.
    Project {
        /// Ratatoskr identity of the tombstoned project.
        ai_project_id: AiProjectId,
    },
    /// One provider Artifact belonging to the enclosing archive import.
    Artifact {
        /// Provider-minted identity of the tombstoned Artifact.
        external_artifact_id: EntityLocalId,
    },
}

/// Optional parser identity for a tombstone created during parser-driven reconciliation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiArchiveTombstoneParser {
    /// Parser that created the tombstone record.
    pub parser_name: ParserName,
    /// Version of the parser that created the tombstone record.
    pub parser_version: ParserVersion,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// Immutable import evidence repeated beside one conversation fact.
///
/// A retained or replayed conversation fact can therefore be checked without
/// requiring a retained earlier import event or access to a producer database.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiArchiveProvenance {
    /// Ratatoskr identity of the import that normalized this conversation.
    pub ai_archive_id: AiArchiveId,
    /// Provider whose export produced the import.
    pub provider: AiProvider,
    /// Tenant that owns the import and conversation.
    pub owner: TenantRef,
    /// Immutable raw export backing the import, including its content digest.
    pub source_export: BlobRef,
    /// Instant the producer completed the normalized import on its own clock.
    pub imported_at: WireTimestamp,
    /// Parser that normalized the import.
    pub parser_name: ParserName,
    /// Version of the parser that normalized the import.
    pub parser_version: ParserVersion,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl AiArchiveProvenance {
    /// Makes provenance from the immutable import head.
    #[must_use]
    pub fn from_import(import: &AiArchiveImport) -> Self {
        Self {
            ai_archive_id: import.ai_archive_id,
            provider: import.provider.clone(),
            owner: import.owner,
            source_export: import.source_export.clone(),
            imported_at: import.imported_at,
            parser_name: import.parser_name.clone(),
            parser_version: import.parser_version.clone(),
            extensions: Extensions::new(),
        }
    }

    /// Verifies that the conversation belongs to this import and parser revision.
    ///
    /// # Errors
    ///
    /// Returns [`AiArchiveContractError::ConversationProvenanceMismatch`] when
    /// the conversation's provider, owner, parser name, or parser version does
    /// not agree with this immutable import provenance.
    pub fn validate_conversation(
        &self,
        conversation: &AiConversation,
    ) -> Result<(), AiArchiveContractError> {
        if self.provider != conversation.provider
            || self.owner != conversation.owner
            || self.parser_name != conversation.parser_name
            || self.parser_version != conversation.parser_version
        {
            return Err(AiArchiveContractError::ConversationProvenanceMismatch);
        }
        Ok(())
    }

    /// Verifies that the project was normalized by this import's provider and parser revision.
    ///
    /// # Errors
    ///
    /// Returns [`AiArchiveContractError::ProjectProvenanceMismatch`] when the
    /// project's provider or parser identity disagrees with this immutable
    /// import provenance.
    pub fn validate_project(&self, project: &AiProject) -> Result<(), AiArchiveContractError> {
        if self.provider != project.provider
            || self.parser_name != project.parser_name
            || self.parser_version != project.parser_version
        {
            return Err(AiArchiveContractError::ProjectProvenanceMismatch);
        }
        Ok(())
    }
}

/// Payload of `ai_archive.archive.imported.v1`: one provider export finished importing as
/// immutable evidence and normalized nodes.
///
/// A fact, not a request (`AGENTS.md` principle 9). The payload **is** the import head —
/// identity, owner, evidence reference, timing, stamps, completeness report — so a consumer
/// learns what exists and how whole it is without receiving every conversation body.
/// Conversations arrive as their own added/updated events. Keeping the head itself as the
/// payload type (no wrapper) means the event and [`AiArchiveSnapshot::import`] are one Rust
/// type and cannot disagree about what an import claims.
///
/// [`AiArchiveSnapshot::import`]: crate::AiArchiveSnapshot::import
impl EventPayload for AiArchiveImport {
    const EVENT_TYPE: &'static str = "ai_archive.archive.imported.v1";
}

/// Payload of `ai_archive.conversation.added.v1`: a conversation entered the index.
///
/// Carries the whole conversation graph plus the owning import's identity (state-carried
/// transfer), so at-least-once redelivery is idempotent on `event_id` and no earlier event is
/// needed to interpret this one.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiConversationAdded {
    /// Immutable evidence for the import that normalized this conversation.
    pub import_provenance: AiArchiveProvenance,

    /// The conversation's record as it now stands.
    pub conversation: AiConversation,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl AiConversationAdded {
    /// Verifies that the embedded conversation agrees with its import evidence.
    ///
    /// # Errors
    ///
    /// Returns [`AiArchiveContractError::ConversationProvenanceMismatch`] when
    /// the conversation and its immutable import provenance disagree.
    pub fn validate(&self) -> Result<(), AiArchiveContractError> {
        self.import_provenance
            .validate_conversation(&self.conversation)
    }
}

impl EventPayload for AiConversationAdded {
    const EVENT_TYPE: &'static str = "ai_archive.conversation.added.v1";
}

/// Payload of `ai_archive.conversation.updated.v1`: an existing conversation changed.
///
/// Carries the full current record rather than a delta, for the same reason as
/// [`AiConversationAdded`]: replay after a gap converges to the producer's latest state instead
/// of replaying every intermediate one.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiConversationUpdated {
    /// Immutable evidence for the import that normalized this conversation.
    pub import_provenance: AiArchiveProvenance,

    /// The conversation's record as it now stands.
    pub conversation: AiConversation,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl AiConversationUpdated {
    /// Verifies that the embedded conversation agrees with its import evidence.
    ///
    /// # Errors
    ///
    /// Returns [`AiArchiveContractError::ConversationProvenanceMismatch`] when
    /// the conversation and its immutable import provenance disagree.
    pub fn validate(&self) -> Result<(), AiArchiveContractError> {
        self.import_provenance
            .validate_conversation(&self.conversation)
    }
}

impl EventPayload for AiConversationUpdated {
    const EVENT_TYPE: &'static str = "ai_archive.conversation.updated.v1";
}

/// Payload of `ai_archive.subject.tombstoned.v1`: authoritative deletion evidence.
///
/// The fact is emitted only from provider deletion, compliance deletion, an
/// approved reconciliation policy, or an authenticated owner privacy request.
/// It never represents an object merely missing from one snapshot.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiArchiveTombstone {
    /// Archive import that contained the subject when the evidence was recorded.
    pub ai_archive_id: AiArchiveId,
    /// Provider that owned the archive subject.
    pub provider: AiProvider,
    /// Tenant that owned the archive subject.
    pub owner: TenantRef,
    /// The exact archive subject made unavailable.
    pub subject: AiArchiveTombstoneSubject,
    /// Authoritative source of the deletion decision.
    pub reason: AiArchiveTombstoneReason,
    /// Immutable raw evidence for the deletion decision.
    pub evidence_ref: BlobRef,
    /// Instant the producer observed the authoritative deletion evidence.
    pub observed_at: WireTimestamp,
    /// Parser identity when a parser created the tombstone during reconciliation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parser: Option<AiArchiveTombstoneParser>,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for AiArchiveTombstone {
    const EVENT_TYPE: &'static str = "ai_archive.subject.tombstoned.v1";
}

/// One first-class provider Artifact, carried by reference rather than executable content.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiArtifact {
    /// Provider-minted artifact identity, opaque and case-sensitive.
    pub external_artifact_id: EntityLocalId,
    /// Provider that emitted this Artifact.
    pub provider: AiProvider,
    /// User whose archive contains the Artifact.
    pub owner: TenantRef,
    /// Artifact title when the provider supplied one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<AiTitle>,
    /// Provider-defined Artifact kind, such as `artifact` or `canvas`.
    pub artifact_kind: AiAssetKind,
    /// Content-addressed stored Artifact bytes; bytes never travel in the event.
    pub content_blob: BlobRef,
    /// Digest of the immutable Artifact content represented by `content_blob`.
    pub content_digest: ContentDigest,
    /// Owning project when the provider recorded one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_ref: Option<EntityRef>,
    /// Owning conversation when the provider recorded one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_ref: Option<EntityRef>,
    /// Parser that normalized this Artifact.
    pub parser_name: ParserName,
    /// Version of [`Self::parser_name`].
    pub parser_version: ParserVersion,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// One typed identity that can be removed from the archive search corpus.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "subject_kind", rename_all = "snake_case")]
pub enum AiArchiveSubject {
    /// A project and every search document that depends on it.
    Project {
        /// Ratatoskr-side project identity.
        ai_project_id: AiProjectId,
    },
    /// A conversation and every search document that depends on it.
    Conversation {
        /// Ratatoskr-side conversation identity.
        ai_conversation_id: AiConversationId,
    },
    /// A provider Artifact belonging to this archive.
    Artifact {
        /// Provider-minted Artifact identity.
        external_artifact_id: EntityLocalId,
    },
}

/// State-carried payload of `ai_archive.project.added.v1`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiProjectAdded {
    /// Immutable evidence for the import that normalized `project`.
    pub import_provenance: AiArchiveProvenance,
    /// Current complete project record.
    pub project: AiProject,
    /// Digest of the canonical normalized project representation.
    pub content_digest: ContentDigest,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl AiProjectAdded {
    /// Verifies that the embedded project agrees with its import evidence.
    ///
    /// # Errors
    ///
    /// Returns [`AiArchiveContractError::ProjectProvenanceMismatch`] when the
    /// project and its immutable import provenance disagree.
    pub fn validate(&self) -> Result<(), AiArchiveContractError> {
        self.import_provenance.validate_project(&self.project)
    }
}

impl EventPayload for AiProjectAdded {
    const EVENT_TYPE: &'static str = "ai_archive.project.added.v1";
}

/// State-carried payload of `ai_archive.project.updated.v1`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiProjectUpdated {
    /// Immutable evidence for the import that normalized `project`.
    pub import_provenance: AiArchiveProvenance,
    /// Current complete project record.
    pub project: AiProject,
    /// Digest of the canonical normalized project representation.
    pub content_digest: ContentDigest,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl AiProjectUpdated {
    /// Verifies that the embedded project agrees with its import evidence.
    ///
    /// # Errors
    ///
    /// Returns [`AiArchiveContractError::ProjectProvenanceMismatch`] when the
    /// project and its immutable import provenance disagree.
    pub fn validate(&self) -> Result<(), AiArchiveContractError> {
        self.import_provenance.validate_project(&self.project)
    }
}

impl EventPayload for AiProjectUpdated {
    const EVENT_TYPE: &'static str = "ai_archive.project.updated.v1";
}

/// State-carried payload of `ai_archive.artifact.added.v1`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiArtifactAdded {
    /// Immutable evidence for the import that normalized `artifact`.
    pub import_provenance: AiArchiveProvenance,
    /// Current complete Artifact record.
    pub artifact: AiArtifact,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for AiArtifactAdded {
    const EVENT_TYPE: &'static str = "ai_archive.artifact.added.v1";
}

/// State-carried payload of `ai_archive.artifact.updated.v1`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiArtifactUpdated {
    /// Immutable evidence for the import that normalized `artifact`.
    pub import_provenance: AiArchiveProvenance,
    /// Current complete Artifact record.
    pub artifact: AiArtifact,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for AiArtifactUpdated {
    const EVENT_TYPE: &'static str = "ai_archive.artifact.updated.v1";
}

/// Payload of `knowledge.ai_archive_analysis.completed.v1`: Knowledge accepted one revision.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiArchiveAnalysisCompleted {
    /// Import that supplied the analysed state.
    pub ai_archive_id: AiArchiveId,
    /// User whose record was analysed.
    pub owner: TenantRef,
    /// Exact record whose state was accepted by Knowledge.
    pub subject: AiArchiveSubject,
    /// Immutable normalized revision that was analysed.
    pub content_digest: ContentDigest,
    /// Instant Knowledge accepted the completed analysis.
    pub completed_at: WireTimestamp,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for AiArchiveAnalysisCompleted {
    const EVENT_TYPE: &'static str = "knowledge.ai_archive_analysis.completed.v1";
}
