//! The bounded, integer completion estimate.

use crate::error::OperationContractError;

/// Completion estimate, whole percent, 0..=100 inclusive
/// (`ARCHITECTURE.md` S5.4 "optional bounded percentage").
///
/// Integer, never floating point: a float on the wire makes canonical serialization and content
/// hashing platform-dependent. The unit is in the name of the field that holds it
/// (`progress_percent`), per `AGENTS.md` principle 6.
///
/// Absent means "the producer cannot estimate", never "zero". Not monotonic — only `status` is
/// (S5.4).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "u8", into = "u8")]
pub struct ProgressPercent(u8);

impl ProgressPercent {
    /// The inclusive upper bound, published as JSON Schema `maximum`.
    pub const MAX: u8 = 100;

    /// Validates and wraps a whole-percent estimate.
    ///
    /// # Errors
    ///
    /// [`OperationContractError::ProgressPercentOutOfRange`] above 100.
    pub fn new(value: u8) -> Result<Self, OperationContractError> {
        if value > Self::MAX {
            return Err(OperationContractError::ProgressPercentOutOfRange { got: value });
        }
        Ok(Self(value))
    }

    /// The validated whole-percent value.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for ProgressPercent {
    type Error = OperationContractError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ProgressPercent> for u8 {
    fn from(value: ProgressPercent) -> Self {
        value.0
    }
}

impl core::fmt::Display for ProgressPercent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

impl schemars::JsonSchema for ProgressPercent {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ProgressPercent")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::ProgressPercent"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "integer",
            "title": "ProgressPercent",
            "description": "Completion estimate in whole percent, 0..=100 inclusive. Integer, \
                            never floating point, so canonical serialization is \
                            platform-independent. Absence means the producer cannot estimate, \
                            never zero.",
            "minimum": 0,
            "maximum": Self::MAX,
            "examples": [42],
        })
    }
}
