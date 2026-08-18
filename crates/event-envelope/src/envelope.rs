//! The envelope itself: [`EventEnvelope`], [`EnvelopeSchemaVersion`], [`EventPayload`] and
//! [`EnvelopeError`].

use core::convert::TryFrom;

use ratatoskr_identifiers::{
    EntityRef, EventId, Extensions, TenantRef, WireTimestamp, canonical_json,
};

use crate::event_type::EventType;
use crate::producer::ProducerName;

/// Major version of the **envelope itself** — not of the payload. The payload major lives in
/// `event_type`'s `.v<major>` suffix (`ARCHITECTURE.md` S5.2, ADR-0002).
///
/// Deserialization rejects any value other than [`Self::CURRENT`], so an envelope produced by a
/// future envelope major is refused **at parse time** with a named error rather than being
/// half-interpreted. `DOMAIN.md` invariant 6, "rejected explicitly" branch.
///
/// The check is field-local: there is no hand-written envelope `Deserialize` for a future field
/// to bypass.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "u32", into = "u32")]
pub struct EnvelopeSchemaVersion(u32);

impl EnvelopeSchemaVersion {
    /// The only envelope major this build can interpret.
    pub const CURRENT: Self = Self(1);

    /// The wire integer.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for EnvelopeSchemaVersion {
    type Error = EnvelopeError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == Self::CURRENT.0 {
            Ok(Self::CURRENT)
        } else {
            Err(EnvelopeError::UnsupportedSchemaVersion {
                found: value,
                supported: Self::CURRENT.0,
            })
        }
    }
}

impl From<EnvelopeSchemaVersion> for u32 {
    fn from(value: EnvelopeSchemaVersion) -> Self {
        value.0
    }
}

impl schemars::JsonSchema for EnvelopeSchemaVersion {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("EnvelopeSchemaVersion")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::EnvelopeSchemaVersion"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "integer",
            "title": "EnvelopeSchemaVersion",
            "description": "Major version of the envelope contract. Always 1 for this schema.",
            "const": 1,
        })
    }
}

/// The common envelope for every asynchronous domain event (`ARCHITECTURE.md` S5.2).
///
/// **Tolerant reader.** Unknown top-level fields land in [`Self::extensions`] and are re-emitted
/// unchanged, so adding an optional envelope field stays backward compatible exactly as
/// `AGENTS.md` classifies it. A future *envelope major* is a different matter and is refused by
/// [`EnvelopeSchemaVersion`].
///
/// Field order below is the order in `ARCHITECTURE.md` S5.2, and derived `Serialize` preserves
/// it, so the documented example is a byte-exact fixture.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EventEnvelope {
    /// Globally unique identity of this event occurrence, and the at-least-once deduplication
    /// key. `UUIDv7` for newly minted events. Bare UUID on the wire, not namespaced. Required.
    pub event_id: EventId,

    /// `<bounded_context>.<aggregate>.<action>.v<major>`. The suffix is the **payload** major.
    /// Required.
    pub event_type: EventType,

    /// Instant at which the fact became true in the producing bounded context. Producer-asserted:
    /// the clock is the producer's. Neither the publish time nor the receive time; the envelope
    /// carries no such field. Required.
    pub occurred_at: WireTimestamp,

    /// The deployable that asserted the fact. Required.
    pub producer: ProducerName,

    /// Namespaced reference to the aggregate the fact is about, e.g. `document:018f…` or
    /// `x-post:123`. Polymorphic — the referent kind varies by event type — therefore an
    /// [`EntityRef`] and not a typed newtype. Required.
    pub aggregate_id: EntityRef,

    /// Namespaced reference tying this event to the unit of user-visible work it belongs to, e.g.
    /// `operation:018f…`. Required: `AGENTS.md` lists a correlation ID unconditionally, and an
    /// event with no correlation cannot be traced through an at-least-once replay. A root event
    /// with no owning operation correlates to itself as `event:<event_id>`.
    pub correlation_id: EntityRef,

    /// Namespaced reference to the record that directly caused this event — an event today, a
    /// command once command contracts land. Optional: absent when the event has no in-system
    /// cause, such as an external webhook, a scheduled sweep, or a user's first request.
    /// `AGENTS.md`: "causation ID when applicable". `null` and absent both parse to `None`, and
    /// `None` always serializes as absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<EntityRef>,

    /// The owner of the data this fact concerns. Optional: absent on system-scoped events that
    /// belong to no single user. `AGENTS.md`: "owner/tenant identity where required". `null` and
    /// absent both parse to `None`, and `None` always serializes as absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<TenantRef>,

    /// Major version of the envelope contract. Always 1 in this build. Required.
    pub schema_version: EnvelopeSchemaVersion,

    /// Event-type-specific body. Always a JSON object, never a scalar or an array, so payload
    /// contracts can evolve additively. Typed as a map, so "payload is an object" is a type-level
    /// fact and a schema fact with no runtime check. Required; an event with nothing to say
    /// carries `{}`.
    pub payload: serde_json::Map<String, serde_json::Value>,

    /// Unknown-but-preserved additive envelope fields. Never interpreted, re-emitted verbatim in
    /// sorted key order.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventEnvelope {
    /// Deserialize from JSON bytes.
    ///
    /// Every validation this contract performs is in `Deserialize` and all of it is field-local,
    /// so this is a convenience wrapper and `serde_json::from_slice::<EventEnvelope>` is equally
    /// safe.
    ///
    /// # Errors
    ///
    /// [`EnvelopeError::Json`] wrapping the underlying failure, including an unsupported
    /// `schema_version` and any identifier or timestamp rejection.
    pub fn from_json(bytes: &[u8]) -> Result<Self, EnvelopeError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Canonical JSON: two-space pretty, one trailing `\n`. The byte form of
    /// `fixtures/**/valid/*.json`.
    ///
    /// # Errors
    ///
    /// Propagates a `serde_json` serialization failure.
    pub fn to_canonical_json(&self) -> Result<String, EnvelopeError> {
        Ok(canonical_json(self)?)
    }

    /// Typed access to `payload` for a consumer that knows the event type.
    ///
    /// # Errors
    ///
    /// [`EnvelopeError::PayloadType`] when `event_type` is not `P::EVENT_TYPE`,
    /// [`EnvelopeError::Json`] when the body does not match `P`.
    pub fn payload_as<P: EventPayload>(&self) -> Result<P, EnvelopeError> {
        let found = self.event_type.to_wire();
        if found != P::EVENT_TYPE {
            return Err(EnvelopeError::PayloadType {
                expected: P::EVENT_TYPE,
                found,
            });
        }
        Ok(serde_json::from_value(serde_json::Value::Object(
            self.payload.clone(),
        ))?)
    }

    /// Replaces the payload and sets `event_type` from `P`, so an envelope whose `event_type`
    /// disagrees with its body cannot be produced through this API.
    ///
    /// # Errors
    ///
    /// [`EnvelopeError::PayloadNotAnObject`] when `P` does not serialize to a JSON object,
    /// [`EnvelopeError::Json`] when `P` cannot be serialized at all.
    pub fn set_payload<P: EventPayload>(&mut self, payload: &P) -> Result<(), EnvelopeError> {
        let serde_json::Value::Object(body) = serde_json::to_value(payload)? else {
            return Err(EnvelopeError::PayloadNotAnObject);
        };
        self.payload = body;
        self.event_type = P::event_type();
        Ok(())
    }
}

/// A typed event payload bound to exactly one event type.
pub trait EventPayload:
    serde::Serialize + serde::de::DeserializeOwned + schemars::JsonSchema
{
    /// The event type this payload is the body of, e.g. `platform.operation.progressed.v1`.
    /// Validity is proved by the owning crate's own test suite, so [`Self::event_type`] cannot
    /// panic in a shipped build.
    const EVENT_TYPE: &'static str;

    /// [`Self::EVENT_TYPE`], parsed.
    ///
    /// # Panics
    ///
    /// Only if `EVENT_TYPE` is malformed, which every implementing crate's test suite forbids.
    #[must_use]
    #[allow(
        clippy::expect_used,
        reason = "EVENT_TYPE is a compile-time contract constant proved parseable by the \
                  implementing crate's test suite; a build whose constant is malformed is \
                  broken before it reaches a wire"
    )]
    fn event_type() -> EventType {
        EventType::parse(Self::EVENT_TYPE).expect("EVENT_TYPE is a valid event type")
    }
}

/// Every way an envelope operation can fail.
///
/// A *parse* error type: it is never serialized onto the wire — that is
/// `ratatoskr_error_contracts::ErrorEnvelope`'s job.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EnvelopeError {
    /// The bytes are not a well-formed envelope. Wraps every field-local rejection, including a
    /// bad identifier, a non-canonical instant and an unsupported `schema_version`.
    #[error("malformed envelope JSON")]
    Json(#[from] serde_json::Error),
    /// The envelope declares an envelope major this build cannot interpret.
    #[error("unsupported envelope schema_version {found}; this build supports {supported}")]
    UnsupportedSchemaVersion {
        /// The envelope major that was on the wire.
        found: u32,
        /// The only envelope major this build understands.
        supported: u32,
    },
    /// A typed payload was requested for an envelope carrying a different event type.
    #[error("envelope carries {found} but {expected} was requested")]
    PayloadType {
        /// The `EventPayload::EVENT_TYPE` that was requested.
        expected: &'static str,
        /// The `event_type` the envelope actually carries.
        found: String,
    },
    /// A payload type serialized to something other than a JSON object.
    #[error("payload must be a JSON object")]
    PayloadNotAnObject,
}
