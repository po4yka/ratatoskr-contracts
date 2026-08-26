//! Normalized social-source contracts: what `ratatoskr-x`, `ratatoskr-instagram` and
//! `ratatoskr-threads` publish when a post, permalink or capture enters a user's library, and
//! what `ratatoskr-knowledge` consumes to index it.
//!
//! # What is on the wire
//!
//! - [`SocialSourceSnapshot`] — the normalized record plus the facts of *this* capture of it.
//! - [`SocialSourceCaptured`] / [`SocialSourceUpdated`] — the two event payloads, both carrying
//!   the whole snapshot (state-carried transfer).
//! - [`AcquisitionMethod`] / [`SavedAuthority`] — how a source arrived and what its saved-state
//!   claim is worth. Closed vocabularies: an unknown value stops processing rather than being
//!   guessed at, because misreading authority is exactly how an Instagram capture becomes a
//!   phantom bookmark.
//! - [`Platform`] / [`SocialMediaKind`] / [`SocialRelationKind`] — open validated tokens: new
//!   platforms, media kinds and relation kinds must not break a running consumer.
//!
//! # What is deliberately absent
//!
//! No media bytes — a [`SocialMediaItem`] carries a
//! [`BlobRef`](ratatoskr_identifiers::BlobRef) and nothing else. No Ratatoskr-side collection
//! model — only provider-native folder membership. No bookmark timestamps fabricated from
//! capture time: `published_at` is present only when the provider authored it.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod author;
mod collection;
mod error;
mod events;
mod media;
mod relation;
mod snapshot;
mod tokens;
mod values;
mod vocabulary;

pub use crate::author::SocialAuthor;
pub use crate::collection::SocialFolderMembership;
pub use crate::error::SocialContractError;
pub use crate::events::{SocialSourceCaptured, SocialSourceRemoved, SocialSourceUpdated};
pub use crate::media::{SocialMediaItem, SocialMediaKind};
pub use crate::relation::{SocialRelation, SocialRelationKind};
pub use crate::snapshot::{SocialSourceSnapshot, SyncCheckpointCursor};
pub use crate::tokens::Platform;
pub use crate::values::{AuthorHandle, DisplayName, PostPermalink, PostText};
pub use crate::vocabulary::{
    AcquisitionMethod, CaptureCompleteness, RemovalReason, SavedAuthority, UpstreamAvailability,
};
