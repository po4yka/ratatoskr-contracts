//! Machine-actionable error and warning envelopes crossing a process boundary.
//!
//! # What is on the wire
//!
//! - [`ErrorEnvelope`] — a terminal failure (`ARCHITECTURE.md` S5.5).
//! - [`WarningEnvelope`] — a non-terminal problem that did not prevent the recorded outcome.
//! - [`FieldViolation`] — one field-level validation failure inside an [`ErrorEnvelope`].
//!
//! # What is deliberately absent
//!
//! The typed surface has no untyped `details` member, no stack trace, no raw provider response,
//! no credential and no storage path (`ARCHITECTURE.md` S14, `SECURITY.md`, `THREAT_MODEL.md`).
//! Structure that a consumer must read is typed.
//!
//! # The extension channel is a producer obligation, not a guarantee
//!
//! `extensions` is an unbounded `BTreeMap<String, serde_json::Value>` that every consumer
//! re-emits verbatim and never interprets. Nothing in the wire shape stops a producer putting a
//! stack trace or a raw provider response there — `SafeMessage`'s control-character ban covers
//! `message` and nothing else. Keeping provider payloads out of `extensions` is therefore a
//! reviewed obligation on the producing service (`AGENTS.md`, "Ensure error contracts do not leak
//! provider responses containing secrets"), recorded as a residual risk in `docs/THREAT_MODEL.md`.
//! A producer that wants the strict check asserts `extensions.is_empty()` in its own test.
//!
//! # Codes versus messages
//!
//! [`ErrorCode`] is the contract and the only member a consumer may branch on
//! (`AGENTS.md` principle 7). The message is human-readable, not machine-parsed, and not stable
//! across releases; changing it is not a contract change.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod code;
mod envelope;
mod field;
mod trace;

pub use crate::code::ErrorCode;
pub use crate::envelope::{ErrorEnvelope, WarningEnvelope};
pub use crate::field::{FieldPath, FieldViolation};
pub use crate::trace::TraceId;
