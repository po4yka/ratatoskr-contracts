//! [`SocialRelation`]: a quote, reply or repost link to another post.

use ratatoskr_identifiers::{EntityLocalId, wire_string_newtype};

wire_string_newtype! {
    /// How this source references another post on the same platform, e.g. `quote`, `reply`,
    /// `repost`.
    ///
    /// **Open on purpose**, like [`SocialMediaKind`](crate::SocialMediaKind): a relation kind a
    /// consumer does not know is skipped while the record is kept, never a reason to discard
    /// the snapshot.
    pub struct SocialRelationKind {
        pattern  = r"^[a-z][a-z0-9_]{0,31}$",
        max_len  = 32,
        examples = ["quote", "reply", "repost"],
    }
}

/// A link from this source to the post it quotes, replies to or reposts.
///
/// The target is named by its provider external id on the same platform — relations never
/// cross platforms, so no platform field is carried. The target's own Ratatoskr identity, if
/// it ever becomes one, is discovered by consumers through `platform` plus this id.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SocialRelation {
    /// What kind of reference this is. Open token; see [`SocialRelationKind`].
    pub relation_kind: SocialRelationKind,

    /// The provider's external id of the target post.
    pub target_post_id: EntityLocalId,
}
