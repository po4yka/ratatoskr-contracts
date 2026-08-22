//! [`AiContentPart`]: the one shared grammar for message content, with an unknown-part channel
//! that preserves unrecognized provider records verbatim.

use ratatoskr_identifiers::BlobRef;

use crate::asset::{AiAsset, AiCitation, AiToolCall, AiToolResult};
use crate::values::AiText;

/// The JSON member that discriminates a known part on the wire.
pub(crate) const PART_KIND_KEY: &str = "part_kind";

/// Every discriminator the Rust layer parses strictly, in one place so the schema's unknown
/// branch and the tests cannot drift from the match arms.
pub(crate) const KNOWN_PART_KINDS: &[&str] = &[
    "text",
    "markdown",
    "image",
    "asset",
    "citation",
    "tool_call",
    "tool_result",
];

/// One typed piece of a message's content.
///
/// One grammar serves every provider (`docs/ARCHITECTURE.md` S8.2): `ChatGPT`- and `Claude`-shaped
/// messages both decompose into these parts plus extension points. The wire form is an object
/// whose `part_kind` member selects the shape:
///
/// ```json
/// {"part_kind": "text", "text": "plain answer"}
/// ```
///
/// # Preservation
///
/// A `part_kind` this build does not know — or a non-object record — parses into
/// [`AiContentPart::Unknown`] and re-serializes byte-identically, so a future provider record
/// survives normalization and re-export (`AGENTS.md`: archive imports must not discard
/// unrecognized records). A *recognized* kind with an invalid body is **not** demoted to the
/// unknown channel: it fails loudly, because half-typing a record we do understand would be
/// worse than refusing it. The raw export blob is the preservation channel of last resort for
/// such malformed records; the parser records a gap.
///
/// Both directions are hand-written rather than derived, because serde's internally tagged
/// enums cannot express an unknown-catch-all variant, and schemars' derive cannot describe one.
/// Test `content_parts.rs` pins both behaviours, and test `content_parts_schema.rs` pins the
/// schema to them.
///
/// # Member order inside a part
///
/// The outer structs of this crate serialize members in declaration order. A part's payload is
/// staged through `serde_json::Value` (the unknown channel requires it), so members inside a
/// known part re-emit in deterministic sorted-key order rather than declaration order. The
/// rendering is stable and byte-canonical either way.
#[derive(Debug, Clone, PartialEq)]
pub enum AiContentPart {
    /// Plain prose.
    Text {
        /// The text itself.
        text: AiText,
    },
    /// Markdown-formatted prose (assistant answers are typically this).
    Markdown {
        /// The markdown source.
        markdown: AiText,
    },
    /// An image by reference: generated, pasted or captured.
    Image {
        /// Content-addressed bytes owned by the storing service.
        image: BlobRef,
    },
    /// A stored file, artifact or canvas-like object by reference.
    Asset {
        /// What the asset is and where its bytes live.
        asset: AiAsset,
    },
    /// An attribution of the surrounding statement to a source.
    Citation {
        /// What was cited and where its evidence lives.
        citation: AiCitation,
    },
    /// One side of a tool invocation: the call itself.
    ToolCall {
        /// Which tool was invoked, under what id.
        tool_call: AiToolCall,
    },
    /// The other side of a tool invocation: how it ended.
    ToolResult {
        /// How the invocation ended and what it returned inline.
        tool_result: AiToolResult,
    },
    /// A record this build does not recognize, preserved verbatim.
    Unknown(serde_json::Value),
}

impl serde::Serialize for AiContentPart {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error as _;
        use serde::ser::SerializeMap;

        // The unknown channel bypasses the tagged form entirely: the preserved value may be
        // any JSON, not only an object, and it re-emits verbatim.
        let (member_key, member_value): (&'static str, serde_json::Value) = match self {
            Self::Text { text } => (
                "text",
                serde_json::to_value(text).map_err(S::Error::custom)?,
            ),
            Self::Markdown { markdown } => (
                "markdown",
                serde_json::to_value(markdown).map_err(S::Error::custom)?,
            ),
            Self::Image { image } => (
                "image",
                serde_json::to_value(image).map_err(S::Error::custom)?,
            ),
            Self::Asset { asset } => (
                "asset",
                serde_json::to_value(asset).map_err(S::Error::custom)?,
            ),
            Self::Citation { citation } => (
                "citation",
                serde_json::to_value(citation).map_err(S::Error::custom)?,
            ),
            Self::ToolCall { tool_call } => (
                "tool_call",
                serde_json::to_value(tool_call).map_err(S::Error::custom)?,
            ),
            Self::ToolResult { tool_result } => (
                "tool_result",
                serde_json::to_value(tool_result).map_err(S::Error::custom)?,
            ),
            Self::Unknown(value) => return value.serialize(serializer),
        };

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(PART_KIND_KEY, member_key)?;
        map.serialize_entry(member_key, &member_value)?;
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for AiContentPart {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Self::from_value(value).map_err(serde::de::Error::custom)
    }
}

impl AiContentPart {
    /// Classifies one wire value: a recognized `part_kind` is parsed strictly (and a malformed
    /// body fails loudly); anything else lands in [`AiContentPart::Unknown`] verbatim.
    pub(crate) fn from_value(value: serde_json::Value) -> Result<Self, AiContentPartError> {
        let Some(kind) = value.get(PART_KIND_KEY).and_then(serde_json::Value::as_str) else {
            return Ok(Self::Unknown(value));
        };
        let Some(object) = value.as_object() else {
            return Ok(Self::Unknown(value));
        };
        match kind {
            "text" => Ok(Self::Text {
                text: exactly_one(object, "text")?,
            }),
            "markdown" => Ok(Self::Markdown {
                markdown: exactly_one(object, "markdown")?,
            }),
            "image" => Ok(Self::Image {
                image: exactly_one(object, "image")?,
            }),
            "asset" => Ok(Self::Asset {
                asset: exactly_one(object, "asset")?,
            }),
            "citation" => Ok(Self::Citation {
                citation: exactly_one(object, "citation")?,
            }),
            "tool_call" => Ok(Self::ToolCall {
                tool_call: exactly_one(object, "tool_call")?,
            }),
            "tool_result" => Ok(Self::ToolResult {
                tool_result: exactly_one(object, "tool_result")?,
            }),
            _ => Ok(Self::Unknown(value)),
        }
    }

    /// The unknown value when this part came through the preservation channel.
    #[must_use]
    pub fn as_unknown(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Unknown(value) => Some(value),
            _ => None,
        }
    }

    /// `true` when this part came through the preservation channel.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

/// Extracts exactly one payload member named `key` from a tagged part object.
///
/// Any member besides `part_kind` and `key` is a malformed body: the kind was recognized, so
/// the record claims grammar this build owns, and an extra or missing member is a contract
/// violation — not an unknown-record case.
fn exactly_one<T: serde::de::DeserializeOwned>(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &'static str,
) -> Result<T, AiContentPartError> {
    for name in object.keys() {
        if name.as_str() != PART_KIND_KEY && name.as_str() != key {
            return Err(AiContentPartError::UnexpectedMember {
                expected: key,
                found: name.clone(),
            });
        }
    }
    let Some(value) = object.get(key) else {
        return Err(AiContentPartError::MissingMember { expected: key });
    };
    serde_json::from_value::<T>(value.clone()).map_err(|error| AiContentPartError::MemberBody {
        expected: key,
        source: error,
    })
}

/// Every way a recognized part can fail to parse.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum AiContentPartError {
    /// A recognized kind carried a member this build does not associate with it.
    #[error("malformed AiContentPart: expected only `{expected}`, found `{found}`")]
    UnexpectedMember {
        /// The one member the recognized kind requires.
        expected: &'static str,
        /// The member that was actually present.
        found: String,
    },
    /// A recognized kind lacked its required member.
    #[error("malformed AiContentPart: missing required member `{expected}`")]
    MissingMember {
        /// The member the recognized kind requires.
        expected: &'static str,
    },
    /// A recognized kind's member did not match its type.
    #[error("malformed AiContentPart: member `{expected}` does not match its type: {source}")]
    MemberBody {
        /// The member that failed to parse.
        expected: &'static str,
        /// The underlying serde error.
        source: serde_json::Error,
    },
}

impl schemars::JsonSchema for AiContentPart {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("AiContentPart")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::AiContentPart"))
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // One branch per known kind, then the always-valid unknown branch. The published schema
        // is a lower bound (ADR-0001): it accepts any object, because deciding whether an
        // object is a malformed known part needs the Rust layer.
        let branch = |kind: &'static str, member: &'static str, member_schema: schemars::Schema| {
            schemars::json_schema!({
                "type": "object",
                "properties": {
                    PART_KIND_KEY: { "const": kind },
                    (member): member_schema,
                },
                "required": [PART_KIND_KEY, member],
                "additionalProperties": false,
            })
        };
        schemars::json_schema!({
            "title": "AiContentPart",
            "description": "One typed piece of a message's content, in one grammar shared by \
                            every provider. A `part_kind` selects the shape; an unrecognized \
                            record is preserved verbatim through normalization and re-export.",
            "oneOf": [
                branch("text", "text", generator.subschema_for::<AiText>()),
                branch("markdown", "markdown", generator.subschema_for::<AiText>()),
                branch("image", "image", generator.subschema_for::<BlobRef>()),
                branch("asset", "asset", generator.subschema_for::<AiAsset>()),
                branch("citation", "citation", generator.subschema_for::<AiCitation>()),
                branch("tool_call", "tool_call", generator.subschema_for::<AiToolCall>()),
                branch("tool_result", "tool_result", generator.subschema_for::<AiToolResult>()),
                schemars::json_schema!({
                    "description": "A record this build does not recognize, preserved verbatim. \
                                    Exclusive of the known branches so `oneOf` stays exact.",
                    "x-ratatoskr-unknown-policy": "preserve",
                    "anyOf": [
                        { "not": { "type": "object" } },
                        {
                            "type": "object",
                            "properties": {
                                PART_KIND_KEY: { "not": { "enum": KNOWN_PART_KINDS } },
                            },
                        },
                    ],
                }),
            ],
        })
    }
}
