//! The public view of a long-running Ratatoskr operation and its lifecycle event.
//!
//! # What is on the wire
//!
//! - [`OperationSnapshot`] — a point-in-time public view (`ARCHITECTURE.md` S5.4).
//! - [`OperationReported`] — producer progress sent to Platform.
//! - [`OperationProgressed`] — Platform's client-facing lifecycle snapshot event.
//! - [`OperationStatus`] — the closed lifecycle enum a client branches on.
//!
//! # Two vocabularies, on purpose
//!
//! [`OperationStatus`] is closed and an unknown value is rejected at parse: a client that guesses
//! at an unknown lifecycle state reports unfinished work as finished. [`OperationStage`] is an
//! open validated string and is display-only: it exists so a producer can say `downloading`
//! without a contract change, and no consumer may branch on it.
//!
//! # What is deliberately absent
//!
//! No transition table and no `can_transition_to`: `AGENTS.md` forbids business workflows here,
//! and S5.4 publishes none. No `schema_version`: the envelope already carries the envelope major.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod account_erasure;
mod error;
mod events;
mod kind;
mod percent;
mod result_ref;
mod snapshot;
mod status;

pub use crate::account_erasure::{
    AccountErasureAcknowledged, AccountErasureOutcome, AccountErasureRequested,
};
pub use crate::error::OperationContractError;
pub use crate::events::{OperationProgressed, OperationReported};
pub use crate::kind::{OperationKind, OperationResultKind, OperationStage};
pub use crate::percent::ProgressPercent;
pub use crate::result_ref::OperationResultRef;
pub use crate::snapshot::OperationSnapshot;
pub use crate::status::OperationStatus;
