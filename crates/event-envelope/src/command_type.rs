//! The validated command name: [`CommandType`].

use core::convert::TryFrom;
use core::fmt;
use core::str::FromStr;

use crate::{EventType, EventTypeError};

/// A parsed `<bounded_context>.<aggregate>.<action>.v<major>` command type.
///
/// The grammar is shared with [`EventType`], but this wrapper keeps a command schema and public API
/// from claiming that a request is an event. The action is not interpreted by this type.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct CommandType(EventType);

impl CommandType {
    /// Parses the canonical wire form.
    ///
    /// # Errors
    ///
    /// Propagates [`EventTypeError`] for an invalid four-segment versioned name.
    pub fn parse(raw: &str) -> Result<Self, EventTypeError> {
        Ok(Self(EventType::parse(raw)?))
    }

    /// The canonical wire rendering.
    #[must_use]
    pub fn to_wire(&self) -> String {
        self.0.to_wire()
    }

    /// The payload-contract major carried by the `.v<major>` suffix.
    #[must_use]
    pub const fn major(&self) -> u32 {
        self.0.major()
    }
}

impl fmt::Display for CommandType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for CommandType {
    type Err = EventTypeError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

impl TryFrom<String> for CommandType {
    type Error = EventTypeError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<CommandType> for String {
    fn from(value: CommandType) -> Self {
        value.to_wire()
    }
}

impl schemars::JsonSchema for CommandType {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("CommandType")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::CommandType"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "title": "CommandType",
            "description": "A command name, `<bounded_context>.<aggregate>.<action>.v<major>`. \
                            The `.v<major>` suffix versions the payload contract, not the \
                            envelope. Segments are snake_case.",
            "pattern": EventType::PATTERN,
            "maxLength": EventType::MAX_LEN,
            "examples": ["social.capture.requested.v1"],
        })
    }
}
