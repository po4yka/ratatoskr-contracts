//! [`SocialMediaItem`]: one media attachment, by reference.

use ratatoskr_identifiers::{BlobRef, wire_string_newtype};

use crate::values::PostText;

wire_string_newtype! {
    /// What kind of bytes a media item refers to, e.g. `image`, `video`, `animated`.
    ///
    /// **Open on purpose**, like [`Platform`](crate::Platform): providers add kinds without
    /// warning, and an unknown kind is rendered generically or skipped — never a reason to
    /// discard the record. Consumers branch on equality; treat everything else generically.
    pub struct SocialMediaKind {
        pattern  = r"^[a-z][a-z0-9_]{0,31}$",
        max_len  = 32,
        examples = ["image", "video", "animated"],
    }
}

/// One media attachment of a social source, described by reference.
///
/// The bytes live behind the [`BlobRef`]; this contract never carries image, video or other
/// media bytes, so a payload stays indexable and loggable without a retention decision.
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
pub struct SocialMediaItem {
    /// What kind of bytes the reference names. Open token; see [`SocialMediaKind`].
    pub media_kind: SocialMediaKind,

    /// Content-addressed reference to the stored bytes.
    pub blob: BlobRef,

    /// Provider- or user-authored alternative text describing the media for accessibility and
    /// indexing. Absent when neither supplied one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<PostText>,
}
