//! The site event stream `sites/site-event.v1` and the vocabulary it shares with the draft change
//! message.

use ratatoskr_identifiers::{
    DigestHex, NotesConflictId, OperationId, WireTimestamp, wire_string_newtype,
};

use crate::release::Slug;

wire_string_newtype! {
    /// The opaque reference of one note in the publications of a site.
    ///
    /// The sites service mints it, and only a `site_owner` session of the site ever sees it. It
    /// names no vault path. 1 to 128 ASCII letters, digits, `_` and `-`, so it fits one path
    /// segment of `/v1/sites/{site}/drafts/sources/{source_ref}`.
    pub struct SourceRef {
        pattern  = r"^[A-Za-z0-9_-]{1,128}$",
        max_len  = 128,
        examples = ["src_7Hq2kV9xR4mT1wZp"],
    }
}

/// Contract `sites/site-event.v1`: the data of one event of `GET /v1/sites/{site}/events`.
///
/// Edge sends it as a server-sent event; the event id is the SSE `id`, not a member of this
/// document. The `type` is closed, and each payload refuses every member it does not declare
/// (workspace spec `site-live-events`). `release.published` goes to any caller with
/// `live.release`; `draft.changed`, `edit.completed` and `edit.conflicted` only to a `site_owner`
/// session of the site; `resync` to any caller, and it has no payload.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
#[non_exhaustive]
pub enum SiteEvent {
    /// A new release of the site is current; the site fetches its navigation again.
    #[serde(rename = "release.published")]
    ReleasePublished(ReleaseNotice),
    /// A note in the site's publications changed in the vault.
    #[serde(rename = "draft.changed")]
    DraftChanged(DraftNotice),
    /// An edit of the owner reached the vault.
    #[serde(rename = "edit.completed")]
    EditCompleted(EditCompletedNotice),
    /// An edit of the owner met a revision that it did not see.
    #[serde(rename = "edit.conflicted")]
    EditConflicted(EditConflictedNotice),
    /// The stream lost events the client asked for; the client fetches the current state again.
    #[serde(rename = "resync")]
    Resync,
}

/// The payload of `release.published`: the release id and the publication time, and nothing else.
///
/// No page list, slug, title, source reference, vault path or draft content: an anonymous watcher
/// must not learn which pages changed.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReleaseNotice {
    /// The new current release: the lowercase hex SHA-256 of its `sites/release.v1` document.
    pub release_id: DigestHex,

    /// The instant at which the release became current, from the sites service clock.
    pub published_at: WireTimestamp,
}

/// The payload of `draft.changed`, for a `site_owner` session only.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct DraftNotice {
    /// The note that changed.
    pub source_ref: SourceRef,

    /// The lowercase hex SHA-256 of the new bytes of the note.
    pub revision: DigestHex,

    /// The slug of the note in the current release. Absent when the note is not published.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<Slug>,
}

/// The payload of `edit.completed`, for a `site_owner` session only.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct EditCompletedNotice {
    /// The operation that the edit opened.
    pub operation_id: OperationId,

    /// The note that the edit changed.
    pub source_ref: SourceRef,

    /// The lowercase hex SHA-256 of the bytes that the vault now holds.
    pub revision: DigestHex,
}

/// The payload of `edit.conflicted`, for a `site_owner` session only.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct EditConflictedNotice {
    /// The operation that the edit opened.
    pub operation_id: OperationId,

    /// The conflict that `ratatoskr-notes` raised for the edit.
    pub conflict_id: NotesConflictId,

    /// The note that the edit tried to change.
    pub source_ref: SourceRef,
}
