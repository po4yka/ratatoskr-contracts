//! The canonical command envelope: [`CommandEnvelope`] and [`CommandPayload`].

use ratatoskr_identifiers::{
    CommandId, EntityRef, Extensions, TenantRef, WireTimestamp, canonical_json,
};

use crate::{CommandType, EnvelopeSchemaVersion, ProducerName};

/// The common envelope for every asynchronous Ratatoskr command.
///
/// Commands request work; they are deliberately distinct from completed-fact
/// [`crate::EventEnvelope`]s. Within an envelope major, unknown additive members are preserved
/// and re-emitted. An unsupported envelope major is rejected by [`EnvelopeSchemaVersion`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CommandEnvelope {
    /// Globally unique identity of this command delivery. A consumer uses it as the at-least-once
    /// deduplication key; domain intent idempotency remains in the typed payload.
    pub command_id: CommandId,
    /// `<bounded_context>.<aggregate>.<action>.v<major>`. The suffix versions the payload.
    pub command_type: CommandType,
    /// Instant at which the producer issued the command.
    pub issued_at: WireTimestamp,
    /// The deployable that requested the work.
    pub producer: ProducerName,
    /// The aggregate the requested work concerns.
    pub aggregate_id: EntityRef,
    /// The user-visible work this command belongs to.
    pub correlation_id: EntityRef,
    /// The in-system record that directly caused this command, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<EntityRef>,
    /// The owner of the requested work, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<TenantRef>,
    /// The major version of the common envelope contract.
    pub schema_version: EnvelopeSchemaVersion,
    /// Command-type-specific body. It is always an object, never a scalar or array.
    pub payload: serde_json::Map<String, serde_json::Value>,
    /// Unknown-but-preserved additive envelope members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl CommandEnvelope {
    /// Deserializes a command envelope from JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::Json`] when any envelope member is invalid.
    pub fn from_json(bytes: &[u8]) -> Result<Self, CommandError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Serializes the command as canonical JSON: two-space pretty with one trailing newline.
    ///
    /// # Errors
    ///
    /// Propagates a JSON serialization failure.
    pub fn to_canonical_json(&self) -> Result<String, CommandError> {
        Ok(canonical_json(self)?)
    }

    /// Deserializes the payload as its one declared command type.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::PayloadType`] when this envelope carries a different type, or
    /// [`CommandError::Json`] when its object does not match `P`.
    pub fn payload_as<P: CommandPayload>(&self) -> Result<P, CommandError> {
        let found = self.command_type.to_wire();
        if found != P::COMMAND_TYPE {
            return Err(CommandError::PayloadType {
                expected: P::COMMAND_TYPE,
                found,
            });
        }
        Ok(serde_json::from_value(serde_json::Value::Object(
            self.payload.clone(),
        ))?)
    }

    /// Replaces the payload and sets the matching command type together.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::PayloadNotAnObject`] when `P` serializes to a non-object, or
    /// [`CommandError::Json`] when it cannot be serialized.
    pub fn set_payload<P: CommandPayload>(&mut self, payload: &P) -> Result<(), CommandError> {
        let serde_json::Value::Object(body) = serde_json::to_value(payload)? else {
            return Err(CommandError::PayloadNotAnObject);
        };
        self.payload = body;
        self.command_type = P::command_type();
        Ok(())
    }
}

/// A typed command payload bound to exactly one command type.
pub trait CommandPayload:
    serde::Serialize + serde::de::DeserializeOwned + schemars::JsonSchema
{
    /// The command type this payload is the body of.
    const COMMAND_TYPE: &'static str;

    /// [`Self::COMMAND_TYPE`], parsed.
    ///
    /// # Panics
    ///
    /// Only when the implementing contract declared a malformed static command type.
    #[must_use]
    #[allow(
        clippy::expect_used,
        reason = "COMMAND_TYPE is a compile-time contract constant proved by the owning crate's tests"
    )]
    fn command_type() -> CommandType {
        CommandType::parse(Self::COMMAND_TYPE).expect("COMMAND_TYPE is a valid command type")
    }
}

/// Every way command-envelope processing can fail.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CommandError {
    /// The bytes are not a valid command envelope or typed payload.
    #[error("malformed command JSON")]
    Json(#[from] serde_json::Error),
    /// A consumer requested a typed payload from a command of another type.
    #[error("command carries {found} but {expected} was requested")]
    PayloadType {
        /// The requested payload's command type.
        expected: &'static str,
        /// The command type actually carried.
        found: String,
    },
    /// A command payload serialized to something other than a JSON object.
    #[error("command payload must be a JSON object")]
    PayloadNotAnObject,
}
