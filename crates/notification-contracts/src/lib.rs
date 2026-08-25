//! User-facing notification contracts: the typed payload a producer service raises when one of
//! its users should be told something, and the vocabulary that payload travels with.
//!
//! # What is on the wire
//!
//! - [`NotificationRaised`] — payload of `platform.notification.raised.v1`, carried inside the
//!   canonical event envelope: the notification's own identity, the taxonomy registry version,
//!   its class, the recipient, carrier-safe text, opaque correlation references and advisory
//!   delivery hints.
//! - [`NotificationClass`] — the versioned, extensible class taxonomy. Known classes at registry
//!   version 1 are `operation_completed`, `operation_failed`, `analysis_ready`,
//!   `backup_outcome`, `watch_triggered` and `archive_imported`; an unknown token is preserved
//!   verbatim, never rejected, so a class added by a later producer still reaches its audience.
//! - [`NotificationPriority`] / [`QuietHoursHint`] — advisory delivery hints. The contract fixes
//!   their wire shape only; enforcement stays with `ratatoskr-telegram`.
//!
//! # What is deliberately absent
//!
//! No delivery guarantees beyond the bus's own at-least-once semantics, no dedupe or preference
//! filtering (Telegram owns both), no email/webhook channels, no preference storage, no UI
//! rendering, and no command envelope: raising a notification is a completed fact about what a
//! producer judged worth telling, not an order to send a message.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod hints;
mod payload;
mod taxonomy;

pub use crate::error::NotificationContractError;
pub use crate::hints::{NotificationPriority, QuietHoursHint};
pub use crate::payload::NotificationRaised;
pub use crate::taxonomy::{NOTIFICATION_CLASS_REGISTRY_VERSION, NotificationClass};
