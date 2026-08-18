//! Shared wire identifiers.
//!
//! # Wire-form rule (ADR-0002)
//!
//! A field naming the entity a record **is** has a statically known kind, so its wire form is the
//! bare canonical lowercase-hyphenated UUID and its type is a typed newtype ([`EventId`],
//! [`OperationId`], [`UserId`], [`CorrelationId`]).
//!
//! A field **pointing at** another entity whose kind is not fixed by the schema is self-describing
//! on the wire as `<kind>:<local_id>` and its type is [`EntityRef`].
//!
//! This is the only reading under which every identifier in the `ARCHITECTURE.md` S5.2 example is
//! legal — `event_id` is bare there, the other four are prefixed — and under which `README.md`'s
//! `"aggregate_id": "x-post:123"` is legal, whose local part is not a UUID at all.
//!
//! # Repository-wide serialization rules
//!
//! - Every validated string type serializes as a JSON string and validates on the way in.
//! - `None` and an absent member are the same thing, and `None` always serializes as absent.
//! - Timestamps have exactly one spelling per instant; see [`WireTimestamp`].
//! - Unknown additive members are preserved in an [`Extensions`] map and re-emitted sorted.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod macros;

mod blob;
mod entity;
mod error;
mod text;
mod timestamp;
mod uuid_ids;
mod wire;

pub use crate::blob::BlobRef;
pub use crate::entity::{EntityKind, EntityLocalId, EntityRef, TenantRef};
pub use crate::error::IdentifierError;
pub use crate::macros::doc_description;
pub use crate::text::SafeMessage;
pub use crate::timestamp::WireTimestamp;
pub use crate::uuid_ids::{CorrelationId, EventId, OperationId, UserId};
pub use crate::wire::{Extensions, canonical_json, dropped_field_pointers};
