//! Desired-backup-policy contracts: what `ratatoskr-github` publishes to state which
//! repositories must be preserved and at what depth, what `ratatoskr-vault` answers through the
//! canonical event envelope, and the drift rules that decide what a policy version covers when
//! the repository catalog changes between versions.
//!
//! # What is on the wire
//!
//! - [`DesiredBackupPolicy`] — the versioned policy document: a monotonic `policy_version`,
//!   per-repository entries carrying a repository reference, a mirror cadence class, priority
//!   and size hints, and explicit exclusions.
//! - [`PolicyAcknowledged`] — payload of `vault.backup_policy.acknowledged.v1`: an accepted or
//!   rejected outcome with stable, machine-actionable reason codes.
//! - [`MirrorCadence`] / [`BackupPriorityHint`] / [`BackupExclusionScope`] /
//!   [`PolicyRejectionCode`] — closed vocabularies: an unrecognized value stops processing
//!   rather than being guessed at, because misreading one silently changes backup frequency,
//!   ordering or scope.
//! - [`validate_policy_succession`], [`uncovered_catalog_repositories`],
//!   [`entries_absent_from_catalog`], [`apply_exclusions`] — drift semantics as pure functions
//!   over borrowed slices, testable without I/O.
//!
//! # What is deliberately absent
//!
//! No retention or tombstone vocabulary, no scheduler mechanics, no concrete cadence intervals,
//! no ownership or tenant fields: a policy names repositories, not people. The publication
//! transport is the services' choice and is not modeled here.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod events;
mod policy;

pub use crate::error::BackupContractError;
pub use crate::events::{
    PolicyAcknowledged, PolicyOutcome, PolicyRejectionCode, PolicyRejectionReason,
};
pub use crate::policy::{
    BackupExclusion, BackupExclusionScope, BackupPriorityHint, DesiredBackupPolicy,
    ExclusionExpression, MirrorCadence, RepositoryBackupEntry, apply_exclusions,
    entries_absent_from_catalog, uncovered_catalog_repositories, validate_policy_succession,
};
