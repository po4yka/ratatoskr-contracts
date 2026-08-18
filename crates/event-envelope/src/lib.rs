//! The common envelope for every asynchronous Ratatoskr domain event.
//!
//! # What this crate owns
//!
//! - [`EventType`] — the validated `<bounded_context>.<aggregate>.<action>.v<major>` name
//!   (`ARCHITECTURE.md` S9.1, ADR-0002). The `.v<major>` suffix is the **payload** major.
//! - [`ProducerName`] — the deployable that asserted a fact.
//! - [`EventEnvelope`] — the envelope of `ARCHITECTURE.md` S5.2, field for field and in order.
//! - [`EnvelopeSchemaVersion`] — the major of the **envelope contract itself**, which is a
//!   different axis from the payload major and is refused at parse time when it is not
//!   [`EnvelopeSchemaVersion::CURRENT`].
//!
//! # The two compatibility axes
//!
//! An unknown *field* inside the current envelope major is **preserved**: it lands in
//! [`EventEnvelope::extensions`] and is re-emitted verbatim, because `README.md`'s compatibility
//! policy says consumers ignore unknown additive fields and `AGENTS.md` classifies adding an
//! optional field as backward compatible.
//!
//! An unknown *envelope major* is **rejected**: a build that cannot know which members changed
//! meaning must not half-interpret the record. Both branches together are `DOMAIN.md`
//! invariant 6, "preserved or rejected explicitly".
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod envelope;
mod event_type;
mod producer;

pub use crate::envelope::{EnvelopeError, EnvelopeSchemaVersion, EventEnvelope, EventPayload};
pub use crate::event_type::{EventType, EventTypeError};
pub use crate::producer::ProducerName;
