//! The single enumeration point for every root type this repository publishes.
//!
//! Explicit and hand-written. No reflection, no build script, no `inventory`/`linkme`, no glob
//! over `.rs` files: every one of those makes the generated output depend on link order or
//! filesystem order, which is exactly what `ARCHITECTURE.md` S11 forbids.
//!
//! The dependency of this crate on the contract crates **is** the enumeration mechanism: a
//! contract that is not reachable from here cannot be generated, and `contracts.toml` rule R2
//! proves the two lists are the same set in both directions.

use std::collections::BTreeMap;

use ratatoskr_event_envelope::{CommandPayload, EventPayload};

/// One publishable root type.
pub struct RootType {
    /// Must match a `contract.root_type.rust_path` in `contracts.toml` byte for byte.
    ///
    /// **Authored**, not `stringify!`d, so the R2 cross-check compares two authored strings and
    /// cannot be defeated by token spacing.
    pub rust_path: &'static str,

    /// Builds the root schema for this type with the supplied generator.
    pub schema: fn(&mut schemars::SchemaGenerator) -> schemars::Schema,

    /// Deserialize then re-serialize; drives the fixture round-trip and lossless-parse checks.
    pub roundtrip: fn(&serde_json::Value) -> Result<serde_json::Value, String>,
}

impl core::fmt::Debug for RootType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RootType")
            .field("rust_path", &self.rust_path)
            .finish_non_exhaustive()
    }
}

impl RootType {
    /// The Rust type's short name, used as the schema `title` and as the lint's declaring-type
    /// key, e.g. `EventEnvelope` for `ratatoskr_event_envelope::EventEnvelope`.
    #[must_use]
    pub fn short_name(&self) -> &'static str {
        short_name(self.rust_path)
    }
}

/// The short name of a `::`-qualified Rust path.
#[must_use]
pub fn short_name(rust_path: &str) -> &str {
    rust_path.rsplit("::").next().unwrap_or(rust_path)
}

macro_rules! root_types {
    ($($path:literal => $ty:ty),+ $(,)?) => {
        /// Every root type, sorted by `rust_path`; test `M-2` enforces the sort.
        #[must_use]
        pub fn root_types() -> Vec<RootType> {
            vec![$(RootType {
                rust_path: $path,
                schema: |generator| generator.root_schema_for::<$ty>(),
                roundtrip: |value| serde_json::from_value::<$ty>(value.clone())
                    .map_err(|error| error.to_string())
                    .and_then(|typed| serde_json::to_value(&typed).map_err(|e| e.to_string())),
            }),+]
        }
    };
}

root_types! {
    "ratatoskr_ai_archive_contracts::AiArchiveAnalysisCompleted"
        => ratatoskr_ai_archive_contracts::AiArchiveAnalysisCompleted,
    "ratatoskr_ai_archive_contracts::AiArchiveImport"
        => ratatoskr_ai_archive_contracts::AiArchiveImport,
    "ratatoskr_ai_archive_contracts::AiArchiveSnapshot"
        => ratatoskr_ai_archive_contracts::AiArchiveSnapshot,
    "ratatoskr_ai_archive_contracts::AiArchiveTombstone"
        => ratatoskr_ai_archive_contracts::AiArchiveTombstone,
    "ratatoskr_ai_archive_contracts::AiArtifactAdded"
        => ratatoskr_ai_archive_contracts::AiArtifactAdded,
    "ratatoskr_ai_archive_contracts::AiArtifactUpdated"
        => ratatoskr_ai_archive_contracts::AiArtifactUpdated,
    "ratatoskr_ai_archive_contracts::AiConversation"
        => ratatoskr_ai_archive_contracts::AiConversation,
    "ratatoskr_ai_archive_contracts::AiConversationAdded"
        => ratatoskr_ai_archive_contracts::AiConversationAdded,
    "ratatoskr_ai_archive_contracts::AiConversationUpdated"
        => ratatoskr_ai_archive_contracts::AiConversationUpdated,
    "ratatoskr_ai_archive_contracts::AiMessage"  => ratatoskr_ai_archive_contracts::AiMessage,
    "ratatoskr_ai_archive_contracts::AiProject"  => ratatoskr_ai_archive_contracts::AiProject,
    "ratatoskr_ai_archive_contracts::AiProjectAdded"
        => ratatoskr_ai_archive_contracts::AiProjectAdded,
    "ratatoskr_ai_archive_contracts::AiProjectUpdated"
        => ratatoskr_ai_archive_contracts::AiProjectUpdated,
    "ratatoskr_backup_contracts::DesiredBackupPolicy"
        => ratatoskr_backup_contracts::DesiredBackupPolicy,
    "ratatoskr_backup_contracts::PolicyAcknowledged"
        => ratatoskr_backup_contracts::PolicyAcknowledged,
    "ratatoskr_backup_contracts::VaultBackupPolicyApplyRequested"
        => ratatoskr_backup_contracts::VaultBackupPolicyApplyRequested,
    "ratatoskr_blob_transfer_contracts::UploadChunkReceipt"
        => ratatoskr_blob_transfer_contracts::UploadChunkReceipt,
    "ratatoskr_blob_transfer_contracts::UploadCompletionOutcome"
        => ratatoskr_blob_transfer_contracts::UploadCompletionOutcome,
    "ratatoskr_blob_transfer_contracts::UploadFinalizeRequest"
        => ratatoskr_blob_transfer_contracts::UploadFinalizeRequest,
    "ratatoskr_blob_transfer_contracts::UploadSessionOpened"
        => ratatoskr_blob_transfer_contracts::UploadSessionOpened,
    "ratatoskr_blob_transfer_contracts::UploadSessionRequest"
        => ratatoskr_blob_transfer_contracts::UploadSessionRequest,
    "ratatoskr_blob_transfer_contracts::UploadStatusResponse"
        => ratatoskr_blob_transfer_contracts::UploadStatusResponse,
    "ratatoskr_channel_digest_contracts::ChannelDigestManifest"
        => ratatoskr_channel_digest_contracts::ChannelDigestManifest,
    "ratatoskr_channel_digest_contracts::ChannelDigestResultPage"
        => ratatoskr_channel_digest_contracts::ChannelDigestResultPage,
    "ratatoskr_channel_digest_contracts::ChannelDigestResultView"
        => ratatoskr_channel_digest_contracts::ChannelDigestResultView,
    "ratatoskr_channel_digest_contracts::ChannelDigestRunRequested"
        => ratatoskr_channel_digest_contracts::ChannelDigestRunRequested,
    "ratatoskr_channel_digest_contracts::ChannelDigestScheduleOccurrenceRequested"
        => ratatoskr_channel_digest_contracts::ChannelDigestScheduleOccurrenceRequested,
    "ratatoskr_channel_digest_contracts::ChannelDigestSubscriptionPage"
        => ratatoskr_channel_digest_contracts::ChannelDigestSubscriptionPage,
    "ratatoskr_channel_digest_contracts::ChannelDigestSubscriptionSetRequested"
        => ratatoskr_channel_digest_contracts::ChannelDigestSubscriptionSetRequested,
    "ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapCompleted"
        => ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapCompleted,
    "ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapFailed"
        => ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapFailed,
    "ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapRequested"
        => ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapRequested,
    "ratatoskr_document_contracts::ContentCaptureRequested"
        => ratatoskr_document_contracts::ContentCaptureRequested,
    "ratatoskr_document_contracts::ContentDocumentExtracted"
        => ratatoskr_document_contracts::ContentDocumentExtracted,
    "ratatoskr_document_contracts::Document"     => ratatoskr_document_contracts::Document,
    "ratatoskr_error_contracts::ErrorEnvelope"         => ratatoskr_error_contracts::ErrorEnvelope,
    "ratatoskr_event_envelope::CommandEnvelope"        => ratatoskr_event_envelope::CommandEnvelope,
    "ratatoskr_event_envelope::EventEnvelope"          => ratatoskr_event_envelope::EventEnvelope,
    "ratatoskr_github_contracts::RepositoryActionRequest"
        => ratatoskr_github_contracts::RepositoryActionRequest,
    "ratatoskr_github_contracts::RepositoryActionResult"
        => ratatoskr_github_contracts::RepositoryActionResult,
    "ratatoskr_github_contracts::RepositoryAnalysisCompleted"
        => ratatoskr_github_contracts::RepositoryAnalysisCompleted,
    "ratatoskr_github_contracts::RepositoryAnalysisFailed"
        => ratatoskr_github_contracts::RepositoryAnalysisFailed,
    "ratatoskr_github_contracts::RepositoryAnalysisRequested"
        => ratatoskr_github_contracts::RepositoryAnalysisRequested,
    "ratatoskr_github_contracts::RepositoryPreviewRequest"
        => ratatoskr_github_contracts::RepositoryPreviewRequest,
    "ratatoskr_github_contracts::RepositoryPreviewResponse"
        => ratatoskr_github_contracts::RepositoryPreviewResponse,
    "ratatoskr_identifiers::BlobRef"                   => ratatoskr_identifiers::BlobRef,
    "ratatoskr_notes_contracts::ConflictRaised"  => ratatoskr_notes_contracts::ConflictRaised,
    "ratatoskr_notes_contracts::ConflictResolved"
        => ratatoskr_notes_contracts::ConflictResolved,
    "ratatoskr_notes_contracts::FileChanged"     => ratatoskr_notes_contracts::FileChanged,
    "ratatoskr_notes_contracts::FileWriteRequested"
        => ratatoskr_notes_contracts::FileWriteRequested,
    "ratatoskr_notes_contracts::VaultConnected"  => ratatoskr_notes_contracts::VaultConnected,
    "ratatoskr_notes_contracts::VaultDisconnected"
        => ratatoskr_notes_contracts::VaultDisconnected,
    "ratatoskr_notification_contracts::NotificationRaised"
        => ratatoskr_notification_contracts::NotificationRaised,
    "ratatoskr_operation_contracts::AccountErasureAcknowledged"
        => ratatoskr_operation_contracts::AccountErasureAcknowledged,
    "ratatoskr_operation_contracts::AccountErasureRequested"
        => ratatoskr_operation_contracts::AccountErasureRequested,
    "ratatoskr_operation_contracts::KnowledgeDeletionAcknowledged"
        => ratatoskr_operation_contracts::KnowledgeDeletionAcknowledged,
    "ratatoskr_operation_contracts::OperationProgressed"
        => ratatoskr_operation_contracts::OperationProgressed,
    "ratatoskr_operation_contracts::OperationReported"
        => ratatoskr_operation_contracts::OperationReported,
    "ratatoskr_operation_contracts::OperationSnapshot"
        => ratatoskr_operation_contracts::OperationSnapshot,
    "ratatoskr_operation_contracts::PlatformScheduleRegistrationRequested"
        => ratatoskr_operation_contracts::PlatformScheduleRegistrationRequested,
    "ratatoskr_operation_contracts::PlatformStreamErasureRequested"
        => ratatoskr_operation_contracts::PlatformStreamErasureRequested,
    "ratatoskr_operational_contracts::AuditEventPage"
        => ratatoskr_operational_contracts::AuditEventPage,
    "ratatoskr_operational_contracts::OperationInspectionPage"
        => ratatoskr_operational_contracts::OperationInspectionPage,
    "ratatoskr_operational_contracts::PublicStatusDocument"
        => ratatoskr_operational_contracts::PublicStatusDocument,
    "ratatoskr_operational_contracts::ScheduleInspectionPage"
        => ratatoskr_operational_contracts::ScheduleInspectionPage,
    "ratatoskr_sites_contracts::BundleManifest" => ratatoskr_sites_contracts::BundleManifest,
    "ratatoskr_sites_contracts::DeployActivated"
        => ratatoskr_sites_contracts::DeployActivated,
    "ratatoskr_sites_contracts::DeployStored"   => ratatoskr_sites_contracts::DeployStored,
    "ratatoskr_sites_contracts::DraftChanged"   => ratatoskr_sites_contracts::DraftChanged,
    "ratatoskr_sites_contracts::Page"           => ratatoskr_sites_contracts::Page,
    "ratatoskr_sites_contracts::Release"        => ratatoskr_sites_contracts::Release,
    "ratatoskr_sites_contracts::ReleaseBuildRequested"
        => ratatoskr_sites_contracts::ReleaseBuildRequested,
    "ratatoskr_sites_contracts::ReleasePublished"
        => ratatoskr_sites_contracts::ReleasePublished,
    "ratatoskr_sites_contracts::SiteApprovalChanged"
        => ratatoskr_sites_contracts::SiteApprovalChanged,
    "ratatoskr_sites_contracts::SiteEvent"      => ratatoskr_sites_contracts::SiteEvent,
    "ratatoskr_sites_contracts::SiteManifest"   => ratatoskr_sites_contracts::SiteManifest,
    "ratatoskr_social_contracts::SocialCaptureRequested"
        => ratatoskr_social_contracts::SocialCaptureRequested,
    "ratatoskr_social_contracts::SocialSourceAnalysisCompleted"
        => ratatoskr_social_contracts::SocialSourceAnalysisCompleted,
    "ratatoskr_social_contracts::SocialSourceCaptured"
        => ratatoskr_social_contracts::SocialSourceCaptured,
    "ratatoskr_social_contracts::SocialSourceRemoved"
        => ratatoskr_social_contracts::SocialSourceRemoved,
    "ratatoskr_social_contracts::SocialSourceSnapshot"
        => ratatoskr_social_contracts::SocialSourceSnapshot,
    "ratatoskr_social_contracts::SocialSourceUpdated"
        => ratatoskr_social_contracts::SocialSourceUpdated,
}

/// Builds the map from authored `rust_path` strings to a payload trait's wire type constant.
///
/// The paths are authored, not `stringify!`d, for the same reason as in [`root_types`]: rule R2
/// compares two authored strings, so token spacing cannot defeat the cross-check.
macro_rules! payload_types {
    ($payload:ident::$constant:ident; $($path:literal => $ty:ty),+ $(,)?) => {{
        let mut declared = BTreeMap::new();
        $(declared.insert($path, <$ty as $payload>::$constant);)+
        declared
    }};
}

/// `EventPayload::EVENT_TYPE` for every registered root type that is an event payload.
///
/// Kept beside [`root_types`] rather than inside [`RootType`] because only event payloads have
/// one. Metadata rule R9 compares `[contract.event].event_type` against this map, so a
/// `contracts.toml` entry cannot claim an event name the payload type does not declare.
#[must_use]
pub fn event_payload_types() -> BTreeMap<&'static str, &'static str> {
    payload_types! {
        EventPayload::EVENT_TYPE;
        "ratatoskr_ai_archive_contracts::AiArchiveAnalysisCompleted"
            => ratatoskr_ai_archive_contracts::AiArchiveAnalysisCompleted,
        "ratatoskr_ai_archive_contracts::AiArchiveImport"
            => ratatoskr_ai_archive_contracts::AiArchiveImport,
        "ratatoskr_ai_archive_contracts::AiArchiveTombstone"
            => ratatoskr_ai_archive_contracts::AiArchiveTombstone,
        "ratatoskr_ai_archive_contracts::AiConversationAdded"
            => ratatoskr_ai_archive_contracts::AiConversationAdded,
        "ratatoskr_ai_archive_contracts::AiConversationUpdated"
            => ratatoskr_ai_archive_contracts::AiConversationUpdated,
        "ratatoskr_ai_archive_contracts::AiArtifactAdded"
            => ratatoskr_ai_archive_contracts::AiArtifactAdded,
        "ratatoskr_ai_archive_contracts::AiArtifactUpdated"
            => ratatoskr_ai_archive_contracts::AiArtifactUpdated,
        "ratatoskr_ai_archive_contracts::AiProjectAdded"
            => ratatoskr_ai_archive_contracts::AiProjectAdded,
        "ratatoskr_ai_archive_contracts::AiProjectUpdated"
            => ratatoskr_ai_archive_contracts::AiProjectUpdated,
        "ratatoskr_backup_contracts::PolicyAcknowledged"
            => ratatoskr_backup_contracts::PolicyAcknowledged,
        "ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapCompleted"
            => ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapCompleted,
        "ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapFailed"
            => ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapFailed,
        "ratatoskr_github_contracts::RepositoryAnalysisCompleted"
            => ratatoskr_github_contracts::RepositoryAnalysisCompleted,
        "ratatoskr_github_contracts::RepositoryAnalysisFailed"
            => ratatoskr_github_contracts::RepositoryAnalysisFailed,
        "ratatoskr_github_contracts::RepositoryAnalysisRequested"
            => ratatoskr_github_contracts::RepositoryAnalysisRequested,
        "ratatoskr_notes_contracts::ConflictRaised" => ratatoskr_notes_contracts::ConflictRaised,
        "ratatoskr_notes_contracts::ConflictResolved"
            => ratatoskr_notes_contracts::ConflictResolved,
        "ratatoskr_notes_contracts::FileChanged" => ratatoskr_notes_contracts::FileChanged,
        "ratatoskr_notes_contracts::VaultConnected" => ratatoskr_notes_contracts::VaultConnected,
        "ratatoskr_notes_contracts::VaultDisconnected"
            => ratatoskr_notes_contracts::VaultDisconnected,
        "ratatoskr_notification_contracts::NotificationRaised"
            => ratatoskr_notification_contracts::NotificationRaised,
        "ratatoskr_operation_contracts::AccountErasureAcknowledged"
            => ratatoskr_operation_contracts::AccountErasureAcknowledged,
        "ratatoskr_operation_contracts::KnowledgeDeletionAcknowledged"
            => ratatoskr_operation_contracts::KnowledgeDeletionAcknowledged,
        "ratatoskr_operation_contracts::OperationProgressed"
            => ratatoskr_operation_contracts::OperationProgressed,
        "ratatoskr_operation_contracts::OperationReported"
            => ratatoskr_operation_contracts::OperationReported,
        "ratatoskr_sites_contracts::DraftChanged" => ratatoskr_sites_contracts::DraftChanged,
        "ratatoskr_sites_contracts::ReleasePublished"
            => ratatoskr_sites_contracts::ReleasePublished,
        "ratatoskr_sites_contracts::SiteApprovalChanged"
            => ratatoskr_sites_contracts::SiteApprovalChanged,
        "ratatoskr_social_contracts::SocialSourceAnalysisCompleted"
            => ratatoskr_social_contracts::SocialSourceAnalysisCompleted,
        "ratatoskr_social_contracts::SocialSourceCaptured"
            => ratatoskr_social_contracts::SocialSourceCaptured,
        "ratatoskr_social_contracts::SocialSourceRemoved"
            => ratatoskr_social_contracts::SocialSourceRemoved,
        "ratatoskr_social_contracts::SocialSourceUpdated"
            => ratatoskr_social_contracts::SocialSourceUpdated,
        "ratatoskr_document_contracts::ContentDocumentExtracted"
            => ratatoskr_document_contracts::ContentDocumentExtracted,
    }
}

/// `CommandPayload::COMMAND_TYPE` for every registered root type that is a command payload.
///
/// Kept beside [`root_types`] rather than inside [`RootType`] because only command payloads have
/// this static type. Metadata rule R10 compares `[contract.command].command_type` against this
/// map, so a declaration cannot claim a command name the payload does not declare.
#[must_use]
pub fn command_payload_types() -> BTreeMap<&'static str, &'static str> {
    payload_types! {
        CommandPayload::COMMAND_TYPE;
        "ratatoskr_channel_digest_contracts::ChannelDigestRunRequested"
            => ratatoskr_channel_digest_contracts::ChannelDigestRunRequested,
        "ratatoskr_channel_digest_contracts::ChannelDigestScheduleOccurrenceRequested"
            => ratatoskr_channel_digest_contracts::ChannelDigestScheduleOccurrenceRequested,
        "ratatoskr_channel_digest_contracts::ChannelDigestSubscriptionSetRequested"
            => ratatoskr_channel_digest_contracts::ChannelDigestSubscriptionSetRequested,
        "ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapRequested"
            => ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapRequested,
        "ratatoskr_operation_contracts::AccountErasureRequested"
            => ratatoskr_operation_contracts::AccountErasureRequested,
        "ratatoskr_social_contracts::SocialCaptureRequested"
            => ratatoskr_social_contracts::SocialCaptureRequested,
        "ratatoskr_operation_contracts::PlatformScheduleRegistrationRequested"
            => ratatoskr_operation_contracts::PlatformScheduleRegistrationRequested,
        "ratatoskr_operation_contracts::PlatformStreamErasureRequested"
            => ratatoskr_operation_contracts::PlatformStreamErasureRequested,
        "ratatoskr_document_contracts::ContentCaptureRequested"
            => ratatoskr_document_contracts::ContentCaptureRequested,
        "ratatoskr_backup_contracts::VaultBackupPolicyApplyRequested"
            => ratatoskr_backup_contracts::VaultBackupPolicyApplyRequested,
        "ratatoskr_notes_contracts::FileWriteRequested"
            => ratatoskr_notes_contracts::FileWriteRequested,
        "ratatoskr_sites_contracts::ReleaseBuildRequested"
            => ratatoskr_sites_contracts::ReleaseBuildRequested,
    }
}
