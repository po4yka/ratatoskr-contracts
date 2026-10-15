//! Notes contracts: the events `ratatoskr-notes` publishes for connected Obsidian vaults
//! (changeset OBS-023) and the write command it consumes (changeset OBS-024). The
//! cross-repository behaviour is defined in the workspace store: change
//! `add-notes-vault-connections`, specs `notes-vault-connections` and `notes-file-changes`, and
//! change `add-notes-write-back`, spec `notes-write-command`.
//!
//! # What is on the wire
//!
//! - [`VaultConnected`] — payload of `notes.vault.connected.v1`: one vault connection became
//!   active.
//! - [`VaultDisconnected`] — payload of `notes.vault.disconnected.v1`: one vault connection was
//!   removed.
//! - [`FileChanged`] — payload of `notes.file.changed.v1`: one file of a connected vault was
//!   created, updated, renamed or deleted.
//! - [`FileWriteRequested`] — payload of `notes.file.write_requested.v1` (contract
//!   `notes/write-request.v1`): change one file of a connected vault with one
//!   [`WriteOperation`].
//! - [`ConflictRaised`] / [`ConflictResolved`] — payloads of `notes.conflict.raised.v1` and
//!   `notes.conflict.resolved.v1` (contract `notes/conflict.v1`): a write met a revision that its
//!   writer did not see, and the owner resolved it. Hashes only.
//! - [`VaultChannel`] / [`FileChangeKind`] / [`ResolutionChoice`] / [`PropertyChange`] — closed
//!   vocabularies.
//! - [`VaultDisplayName`] / [`VaultPath`] / [`VaultWritePath`] / [`WriteOrigin`] /
//!   [`PropertyKey`] — validated strings.
//!
//! # The envelope carries the owner
//!
//! The owner is the envelope's `tenant_id`, `user:<id>`, and no payload repeats it. The aggregate
//! is `notes_connection:<uuid>` for the two vault events, `notes_file:<uuid>` for
//! `notes.file.changed.v1`, `operation:<uuid>` for `notes.file.write_requested.v1` and
//! `notes_conflict:<uuid>` for the two conflict events.
//!
//! # What is deliberately absent
//!
//! No file bytes, no file text, no vault secret, no device credential and no host path. A write
//! names its content, and the value of a frontmatter property, by a
//! [`ratatoskr_identifiers::BlobRef`] that the caller uploaded first. Every
//! payload refuses a member it does not declare (`unknown_policy = "reject"`), so none of these can
//! ride along in an unknown field. That is a privacy rule, and it is why these payloads carry no
//! `extensions` map.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod conflict;
mod error;
mod file;
mod vault;
mod write;

pub use crate::conflict::{ConflictRaised, ConflictResolved, MergeProposal};
pub use crate::error::NotesContractError;
pub use crate::file::{FileChangeKind, FileChanged, VaultPath};
pub use crate::vault::{VaultChannel, VaultConnected, VaultDisconnected, VaultDisplayName};
pub use crate::write::{
    CreateFile, DeleteFile, FileWriteRequested, PropertyChange, PropertyKey, RelocateFile,
    ResolutionChoice, ResolveConflict, SetFileProperty, UpdateFile, VaultWritePath, WriteOperation,
    WriteOrigin,
};
