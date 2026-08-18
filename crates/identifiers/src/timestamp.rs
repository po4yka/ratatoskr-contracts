//! The one instant type this repository puts on a wire.

use core::convert::TryFrom;
use core::fmt;
use core::str::FromStr;

use crate::error::IdentifierError;

/// An instant on the wire.
///
/// Wire form: RFC 3339 in UTC with the literal `Z` designator, e.g. `2026-08-17T10:00:00Z`
/// (`ARCHITECTURE.md` S9.2: "timestamps use RFC 3339 UTC on the wire").
///
/// **Exactly one spelling per instant.** Parsing accepts a string only when re-rendering the
/// parsed value reproduces the input byte for byte. That single rule rejects `+02:00`, `+00:00`,
/// `2026-08-17T10:00:00.120Z` (renders `.12Z`), `2026-08-17T10:00:00.000Z` (renders without a
/// fraction), a space separator, and lowercase `z`; it accepts `2026-08-17T10:00:00Z` and
/// `2026-08-17T10:00:00.123456789Z`. No precision floor or ceiling is invented.
///
/// This type carries no authority of its own. The **field** that holds it declares whether the
/// instant is producer-asserted, provider-authored, observed, received or processed; see the
/// `[[contract.field]]` entries in `contracts.toml`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct WireTimestamp(jiff::Timestamp);

impl WireTimestamp {
    /// Published as the JSON Schema `pattern` alongside `"format": "date-time"`. A lower bound:
    /// canonicality is enforced in Rust, so an invalid fixture that is merely non-canonical
    /// declares `rejected_by = ["serde"]`.
    pub const PATTERN: &'static str = r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,9})?Z$";

    /// Reads the system clock. Only producers call this.
    #[must_use]
    pub fn now() -> Self {
        Self(jiff::Timestamp::now())
    }

    /// Wraps an instant a producer already holds.
    #[must_use]
    pub const fn from_jiff(instant: jiff::Timestamp) -> Self {
        Self(instant)
    }

    /// The wrapped instant.
    #[must_use]
    pub const fn as_jiff(self) -> jiff::Timestamp {
        self.0
    }

    /// Parses the canonical wire form.
    ///
    /// # Errors
    ///
    /// [`IdentifierError::MalformedInstant`] when jiff cannot parse it (naive input included),
    /// [`IdentifierError::NonCanonicalInstant`] when it parses but is not its own canonical form.
    pub fn parse(raw: &str) -> Result<Self, IdentifierError> {
        let instant: jiff::Timestamp =
            raw.parse()
                .map_err(|reason: jiff::Error| IdentifierError::MalformedInstant {
                    input: raw.to_owned(),
                    reason: reason.to_string(),
                })?;
        let canonical = instant.to_string();
        if canonical != raw {
            return Err(IdentifierError::NonCanonicalInstant {
                input: raw.to_owned(),
                canonical,
            });
        }
        Ok(Self(instant))
    }

    /// The canonical wire rendering.
    #[must_use]
    pub fn to_wire(self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for WireTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for WireTimestamp {
    type Err = IdentifierError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

impl TryFrom<String> for WireTimestamp {
    type Error = IdentifierError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<WireTimestamp> for String {
    fn from(value: WireTimestamp) -> Self {
        value.to_wire()
    }
}

impl schemars::JsonSchema for WireTimestamp {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("WireTimestamp")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::WireTimestamp"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "format": "date-time",
            "title": "WireTimestamp",
            "description": "An instant on the wire: RFC 3339 in UTC with the literal `Z` \
                            designator. Exactly one spelling is accepted per instant — an input \
                            that does not re-render to itself is rejected. The field holding it \
                            declares the clock authority.",
            "pattern": Self::PATTERN,
            "examples": ["2026-08-17T10:00:00Z"],
        })
    }
}
