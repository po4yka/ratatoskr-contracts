//! The explicit browser social-capture command payload.

use ratatoskr_event_envelope::CommandPayload;
use ratatoskr_identifiers::{ContentDigest, Extensions, OperationId, WireTimestamp};

use crate::{AcquisitionMethod, PostPermalink, SavedAuthority};

/// The social owner to which Platform routes an explicit capture.
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
pub enum SocialCaptureProvider {
    /// X owns the permalink.
    X,
    /// Instagram owns the permalink.
    Instagram,
    /// Threads owns the permalink.
    Threads,
}

/// Payload for a Platform-to-social-owner explicit public-permalink capture.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SocialCaptureRequested {
    /// Long-running operation Platform created for this user action.
    pub operation_id: OperationId,
    /// Stable digest of the user intent for domain-level idempotency.
    pub idempotency_key: ContentDigest,
    /// The original public HTTPS permalink the user captured.
    pub original_permalink: PostPermalink,
    /// Instant the browser captured the permalink.
    pub captured_at: WireTimestamp,
    /// The service that owns acquisition for this social provider.
    pub provider: SocialCaptureProvider,
    /// This lane is an explicit browser-extension capture.
    pub acquisition: AcquisitionMethod,
    /// This lane never claims native provider Saved/bookmark authority.
    pub saved_authority: SavedAuthority,
    /// Unknown-but-preserved additive payload members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl CommandPayload for SocialCaptureRequested {
    const COMMAND_TYPE: &'static str = "social.capture.requested.v1";
}
