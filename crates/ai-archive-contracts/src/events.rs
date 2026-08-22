//! The three AI-archive event payloads.

use ratatoskr_event_envelope::EventPayload;

use crate::graph::AiConversation;
use crate::snapshot::AiArchiveImport;
use ratatoskr_identifiers::{AiArchiveId, Extensions};

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
    /// The import this conversation was parsed by.
    pub ai_archive_id: AiArchiveId,

    /// The conversation's record as it now stands.
    pub conversation: AiConversation,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
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
    /// The import this conversation was parsed by.
    pub ai_archive_id: AiArchiveId,

    /// The conversation's record as it now stands.
    pub conversation: AiConversation,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for AiConversationUpdated {
    const EVENT_TYPE: &'static str = "ai_archive.conversation.updated.v1";
}
