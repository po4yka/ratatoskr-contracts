//! Notes contracts: the events `ratatoskr-notes` publishes for connected Obsidian vaults
//! (changeset OBS-023). The cross-repository behaviour is defined in the workspace store, change
//! `add-notes-vault-connections`, specs `notes-vault-connections` and `notes-file-changes`.
//!
//! # What is on the wire
//!
//! - [`VaultConnected`] — payload of `notes.vault.connected.v1`: one vault connection became
//!   active.
//! - [`VaultDisconnected`] — payload of `notes.vault.disconnected.v1`: one vault connection was
//!   removed.
//! - [`FileChanged`] — payload of `notes.file.changed.v1`: one file of a connected vault was
//!   created, updated, renamed or deleted.
//! - [`VaultChannel`] / [`FileChangeKind`] — closed vocabularies.
//! - [`VaultDisplayName`] / [`VaultPath`] — validated strings.
//!
//! # The envelope carries the owner
//!
//! The owner is the envelope's `tenant_id`, `user:<id>`, and no payload repeats it. The aggregate
//! is `notes_connection:<uuid>` for the two vault events and `notes_file:<uuid>` for
//! `notes.file.changed.v1`.
//!
//! # What is deliberately absent
//!
//! No file bytes, no file text, no vault secret, no device credential and no host path. Every
//! payload refuses a member it does not declare (`unknown_policy = "reject"`), so none of these can
//! ride along in an unknown field. That is a privacy rule, and it is why these payloads carry no
//! `extensions` map.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod file;
mod vault;

pub use crate::error::NotesContractError;
pub use crate::file::{FileChangeKind, FileChanged, VaultPath};
pub use crate::vault::{VaultChannel, VaultConnected, VaultDisconnected, VaultDisplayName};
