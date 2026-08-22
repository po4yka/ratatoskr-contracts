//! The two social-source event payloads.

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::Extensions;

use crate::snapshot::SocialSourceSnapshot;

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
