//! The social-source event payloads.

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{ContentDigest, Extensions, SocialSourceId, TenantRef, WireTimestamp};

use crate::snapshot::SocialSourceSnapshot;
use crate::vocabulary::RemovalReason;

/// Payload of `social.source.captured.v1`: a source became part of a user's library.
///
/// A fact, not a request (`AGENTS.md` principle 9). The whole snapshot travels in the payload —
/// state-carried transfer — so a consumer can index the source from this event alone and an
/// at-least-once redelivery is idempotent on `event_id`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SocialSourceCaptured {
    /// The captured source, complete at the instant of the event.
    pub source: SocialSourceSnapshot,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for SocialSourceCaptured {
    const EVENT_TYPE: &'static str = "social.source.captured.v1";
}

/// Payload of `social.source.updated.v1`: an existing source's normalized record changed.
///
/// Carries the full current snapshot rather than a delta, for the same reason as
/// [`SocialSourceCaptured`]: no prior event is needed to interpret this one, so replay after a
/// gap converges to the producer's latest state instead of replaying every intermediate one.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SocialSourceUpdated {
    /// The source's record as it now stands.
    pub source: SocialSourceSnapshot,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for SocialSourceUpdated {
    const EVENT_TYPE: &'static str = "social.source.updated.v1";
}

/// Payload of `social.source.removed.v1`: the user's library stopped holding the source.
///
/// A fact about the library, not about the provider: nothing here claims the post was deleted
/// upstream (`upstream_availability = deleted_upstream` owns that fact). Deliberately minimal —
/// it names what stopped being held, why, and when. It does not re-carry a snapshot, because a
/// removed record is no longer indexable content and a redelivered snapshot must not resurrect
/// it.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SocialSourceRemoved {
    /// The library identity that was removed.
    pub social_source_id: SocialSourceId,

    /// The user whose library no longer holds this source.
    pub owner: TenantRef,

    /// Why the library let go. Closed vocabulary; an unknown reason stops processing.
    pub reason: RemovalReason,

    /// Instant the producing service removed the record. Observed: the producer's clock.
    pub removed_at: WireTimestamp,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for SocialSourceRemoved {
    const EVENT_TYPE: &'static str = "social.source.removed.v1";
}

/// Payload of `knowledge.analysis.completed.v1` for a completed social-source analysis.
/// It is intentionally a linkage fact only: the detailed result and Knowledge-private run
/// identity remain owned by Knowledge, while source services match this fact by source and digest.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SocialSourceAnalysisCompleted {
    /// The user whose source was analysed.
    pub owner: TenantRef,
    /// The analysed source identity.
    pub social_source_id: SocialSourceId,
    /// The exact normalized source revision analysed.
    pub content_digest: ContentDigest,
    /// Instant Knowledge accepted the completed analysis.
    pub completed_at: WireTimestamp,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for SocialSourceAnalysisCompleted {
    const EVENT_TYPE: &'static str = "knowledge.analysis.completed.v1";
}
