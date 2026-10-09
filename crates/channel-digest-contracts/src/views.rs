//! Owner read-API views served by `ratatoskr-channel-digests` and consumed by Platform's typed
//! loopback client (XR-021 CONTRACTS.md S08).
//!
//! The routes are `GET /v1/subscriptions`, `GET /v1/results` and `GET /v1/results/{result_id}`
//! on the digest service's API listener; every request carries the service bearer and the owner
//! header documented in `docs/CHANNEL_DIGEST_MANIFEST.md`. Listings take `page_size` from 1 to
//! 100 (default 50), are scoped to the caller, and list the newest first. Summaries never carry
//! recap content: a recap is fetched through [`ChannelDigestResultView`] only.

use ratatoskr_identifiers::{ContentDigest, Extensions, WireTimestamp, wire_string_newtype};

use crate::{
    ChannelDigestResultId, ChannelDigestRunId, ChannelDigestSubscriptionId, ChannelUsername,
};

wire_string_newtype! {
    /// Content-free operational class explaining why a digest run or result failed, such as
    /// `provider_unavailable` or `manifest_invalid`.
    ///
    /// Open on purpose: the digest service owns the vocabulary and a consumer treats an
    /// unrecognized class as opaque display text.
    pub struct ChannelDigestFailureClass {
        pattern  = r"^[a-z][a-z0-9_.-]{0,63}$",
        max_len  = 64,
        examples = ["provider_unavailable", "manifest_invalid"],
    }
}

/// How a digest result ended.
///
/// Closed on purpose: a client branches on it to decide whether a recap exists, and an
/// unrecognized outcome must stop processing instead of being read as a success.
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
pub enum ChannelDigestOutcome {
    /// Every selected source entered the recap.
    Completed,
    /// A recap exists, but some selected sources were omitted from it.
    Partial,
    /// No recap exists; `safe_failure_class` says why.
    Failed,
}

/// One owner subscription to a public channel.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ChannelDigestSubscriptionView {
    /// Identity of the subscription.
    pub subscription_id: ChannelDigestSubscriptionId,

    /// Canonical public channel username the owner subscribed to.
    pub channel_username: ChannelUsername,

    /// Whether the subscription currently feeds digest runs.
    pub enabled: bool,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// Response of `GET /v1/subscriptions`: the owner's subscriptions, newest first.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ChannelDigestSubscriptionPage {
    /// The subscriptions in this page.
    pub subscriptions: Vec<ChannelDigestSubscriptionView>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// One entry of the result listing: linkage and outcome only, never recap content.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ChannelDigestResultSummary {
    /// Identity of the result.
    pub result_id: ChannelDigestResultId,

    /// The digest run the result belongs to.
    pub run_id: ChannelDigestRunId,

    /// How the result ended.
    pub outcome: ChannelDigestOutcome,

    /// Why a failed result failed. Present exactly for failed results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_failure_class: Option<ChannelDigestFailureClass>,

    /// Instant the digest service recorded the result, on its own clock.
    pub created_at: WireTimestamp,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// Response of `GET /v1/results`: the owner's results, newest first, summaries only.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ChannelDigestResultPage {
    /// The result summaries in this page.
    pub results: Vec<ChannelDigestResultSummary>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// The recap of a digest result: an opaque JSON object owned by Knowledge.
///
/// The digest service relays the object after checking its identity and digest against the stored
/// completion fact, and never interprets it. The contract names it so that an absent recap is a
/// typed optional member; its members are Knowledge's to define. On the wire it is exactly the
/// object, with no wrapper member.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ChannelDigestRecapDocument {
    /// The members of the recap object, preserved verbatim.
    #[serde(flatten)]
    pub members: serde_json::Map<String, serde_json::Value>,
}

/// Response of `GET /v1/results/{result_id}`.
///
/// A completed or partial result carries the local linkage and the Knowledge-owned recap, which
/// the digest service returns only after the analysis identity and digest match the stored
/// completion fact. A failed result is local and minimized to `result_id`, `run_id`, `outcome`
/// and `safe_failure_class`. Absent optionals are not serialized.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ChannelDigestResultView {
    /// Identity of the result.
    pub result_id: ChannelDigestResultId,

    /// The digest run the result belongs to.
    pub run_id: ChannelDigestRunId,

    /// How the result ended.
    pub outcome: ChannelDigestOutcome,

    /// Knowledge's analysis identity of the recap, a bare canonical UUID. Absent for a failed
    /// result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recap_id: Option<uuid::Uuid>,

    /// Number of citations the recap carries. Absent for a failed result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<u16>,

    /// SHA-256 identity of the stored recap result. Absent for a failed result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_digest: Option<ContentDigest>,

    /// The recap itself, an opaque JSON object owned by Knowledge. Absent for a failed result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recap: Option<ChannelDigestRecapDocument>,

    /// Why a failed result failed. Present exactly for failed results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_failure_class: Option<ChannelDigestFailureClass>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}
