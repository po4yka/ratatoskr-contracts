//! Validated channel-digest wire values.

use core::fmt;
use core::str::FromStr;

use crate::ChannelDigestContractError;
use ratatoskr_identifiers::{WireTimestamp, wire_string_newtype};

const UUID_PATTERN: &str = r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

macro_rules! channel_digest_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
        pub struct $name(uuid::Uuid);

        impl $name {
            /// The published canonical UUID schema pattern.
            pub const PATTERN: &'static str = UUID_PATTERN;

            /// Parses a bare canonical lowercase UUID.
            ///
            /// # Errors
            ///
            /// Returns [`ChannelDigestContractError::InvalidIdentifier`] for every non-canonical
            /// spelling or malformed UUID.
            pub fn parse(raw: &str) -> Result<Self, ChannelDigestContractError> {
                if !compiled_uuid_pattern().is_match(raw) {
                    return Err(ChannelDigestContractError::InvalidIdentifier);
                }
                uuid::Uuid::parse_str(raw)
                    .map(Self)
                    .map_err(|_| ChannelDigestContractError::InvalidIdentifier)
            }

            /// Returns the wrapped UUID.
            #[must_use]
            pub const fn as_uuid(self) -> uuid::Uuid {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, formatter)
            }
        }

        impl FromStr for $name {
            type Err = ChannelDigestContractError;

            fn from_str(raw: &str) -> Result<Self, Self::Err> {
                Self::parse(raw)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let raw = <String as serde::Deserialize>::deserialize(deserializer)?;
                Self::parse(&raw).map_err(serde::de::Error::custom)
            }
        }

        impl schemars::JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($name))
            }

            fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({
                    "type": "string",
                    "format": "uuid",
                    "title": stringify!($name),
                    "description": "Canonical channel-digest UUID identity.",
                    "pattern": Self::PATTERN
                })
            }
        }
    };
}

#[allow(
    clippy::expect_used,
    reason = "the published static UUID pattern is compiled in the test suite"
)]
fn compiled_uuid_pattern() -> &'static regex::Regex {
    static COMPILED: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(UUID_PATTERN).expect("UUID_PATTERN must compile")
    });
    &COMPILED
}

channel_digest_id! {
    /// Identity of one owner/public-channel subscription.
    ChannelDigestSubscriptionId
}

wire_string_newtype! {
    /// Bounded opaque identity of one logical channel-digest mutation or run request.
    pub struct ChannelDigestIdempotencyKey {
        pattern  = r"^[A-Za-z0-9][A-Za-z0-9._~:@+-]{0,127}$",
        max_len  = 128,
        examples = ["digest.018f0000-0000-7000-8000-000000000001"],
    }
}

wire_string_newtype! {
    /// Owner-authorized immutable source-manifest reference; never a storage URL.
    pub struct ChannelDigestManifestRef {
        pattern  = r"^channel-digest-manifest:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        max_len  = 60,
        examples = ["channel-digest-manifest:018f0000-0000-7000-8000-000000000002"],
    }
}

wire_string_newtype! {
    /// Owner-authorized digest-result projection reference; never recap content.
    pub struct ChannelDigestResultRef {
        pattern  = r"^channel-digest-result:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        max_len  = 58,
        examples = ["channel-digest-result:018f0000-0000-7000-8000-000000000003"],
    }
}

wire_string_newtype! {
    /// Stable Knowledge analysis reference linked to a digest recap.
    pub struct KnowledgeAnalysisRef {
        pattern  = r"^analysis:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        max_len  = 45,
        examples = ["analysis:018f0000-0000-7000-8000-000000000004"],
    }
}

wire_string_newtype! {
    /// Stable Platform schedule reference for a scheduled digest run.
    pub struct DigestScheduleRef {
        pattern  = r"^schedule:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        max_len  = 45,
        examples = ["schedule:018f0000-0000-7000-8000-000000000005"],
    }
}

wire_string_newtype! {
    /// Stable Platform occurrence reference for one scheduled digest run.
    pub struct DigestOccurrenceRef {
        pattern  = r"^schedule-occurrence:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        max_len  = 56,
        examples = ["schedule-occurrence:018f0000-0000-7000-8000-000000000006"],
    }
}

/// Language of the requested first-version recap projection.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum OutputLanguage {
    /// Russian recap.
    Ru,
    /// English recap.
    En,
}

/// Desired state of one owner/public-channel subscription.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionDesiredState {
    /// Acquire the public channel for future digest runs.
    Active,
    /// Stop acquiring the public channel for future digest runs.
    Inactive,
}

/// Authority which selected one digest run window.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChannelDigestRunTrigger {
    /// A caller-authenticated on-demand request.
    OnDemand {
        /// Platform's accepted instant; must equal the requested window end.
        accepted_at: WireTimestamp,
    },
    /// One stable Platform schedule occurrence.
    Scheduled {
        /// Platform schedule authority.
        schedule_ref: DigestScheduleRef,
        /// Stable occurrence identity for replay-safe scheduling.
        occurrence_ref: DigestOccurrenceRef,
        /// Platform due instant; must equal the requested window end.
        due_at: WireTimestamp,
    },
}

/// Closed-open UTC source window for one digest run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DigestWindow {
    /// Inclusive lower bound.
    pub start_at: WireTimestamp,
    /// Exclusive upper bound.
    pub end_at: WireTimestamp,
}

macro_rules! bounded_count {
    ($(#[$meta:meta])* $name:ident, $max:literal, $error:ident) => {
        $(#[$meta])*
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            serde::Serialize,
            serde::Deserialize,
        )]
        #[serde(try_from = "u16", into = "u16")]
        pub struct $name(u16);

        impl $name {
            /// Inclusive upper bound of this count.
            pub const MAX: u16 = $max;

            /// Validates and wraps a positive bounded count.
            ///
            /// # Errors
            ///
            /// Returns the count-specific contract error when `value` is zero or above
            /// [`Self::MAX`].
            pub fn new(value: u16) -> Result<Self, ChannelDigestContractError> {
                if value == 0 || value > Self::MAX {
                    Err(ChannelDigestContractError::$error)
                } else {
                    Ok(Self(value))
                }
            }

            /// Returns the validated count.
            #[must_use]
            pub const fn get(self) -> u16 {
                self.0
            }
        }

        impl TryFrom<u16> for $name {
            type Error = ChannelDigestContractError;

            fn try_from(value: u16) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for u16 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl schemars::JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($name))
            }

            fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({
                    "type": "integer",
                    "title": stringify!($name),
                    "description": "Positive bounded channel-digest count.",
                    "minimum": 1,
                    "maximum": Self::MAX,
                })
            }
        }
    };
}

bounded_count! {
    /// Positive number of immutable source records selected for one recap request.
    DigestSourceCount, 100, InvalidSourceCount
}

bounded_count! {
    /// Positive number of public channels represented by one recap request.
    DigestChannelCount, 20, InvalidChannelCount
}

impl DigestWindow {
    /// Creates one bounded closed-open digest window.
    ///
    /// # Errors
    ///
    /// Returns [`ChannelDigestContractError::InvalidWindowOrder`] when the exclusive end is not
    /// later than the inclusive start, or [`ChannelDigestContractError::WindowTooLong`] when the
    /// window is longer than seven days.
    pub fn new(
        start_at: WireTimestamp,
        end_at: WireTimestamp,
    ) -> Result<Self, ChannelDigestContractError> {
        let duration = end_at.as_jiff().duration_since(start_at.as_jiff());
        if duration <= jiff::SignedDuration::ZERO {
            return Err(ChannelDigestContractError::InvalidWindowOrder);
        }
        if duration > jiff::SignedDuration::from_hours(7 * 24) {
            return Err(ChannelDigestContractError::WindowTooLong);
        }
        Ok(Self { start_at, end_at })
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DigestWindowWire {
    start_at: WireTimestamp,
    end_at: WireTimestamp,
}

impl<'de> serde::Deserialize<'de> for DigestWindow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = DigestWindowWire::deserialize(deserializer)?;
        Self::new(wire.start_at, wire.end_at).map_err(serde::de::Error::custom)
    }
}

channel_digest_id! {
    /// Identity of one bounded channel-digest run.
    ChannelDigestRunId
}

channel_digest_id! {
    /// Identity of one resolved channel-digest result projection.
    ChannelDigestResultId
}

/// Canonical lowercase public Telegram channel username.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct ChannelUsername(String);

impl ChannelUsername {
    /// The published JSON Schema pattern.
    pub const PATTERN: &'static str = r"^[a-z][a-z0-9_]{4,31}$";

    /// Parses one canonical public-channel username.
    ///
    /// # Errors
    ///
    /// Returns [`ChannelDigestContractError::InvalidChannelUsername`] when the value is not the
    /// canonical lowercase public-channel grammar.
    pub fn parse(raw: &str) -> Result<Self, ChannelDigestContractError> {
        if compiled_channel_username_pattern().is_match(raw) {
            Ok(Self(raw.to_owned()))
        } else {
            Err(ChannelDigestContractError::InvalidChannelUsername)
        }
    }

    /// Returns the canonical wire spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[allow(
    clippy::expect_used,
    reason = "the published static pattern is compiled in the test suite"
)]
fn compiled_channel_username_pattern() -> &'static regex::Regex {
    static COMPILED: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(ChannelUsername::PATTERN).expect("ChannelUsername::PATTERN must compile")
    });
    &COMPILED
}

impl fmt::Display for ChannelUsername {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ChannelUsername {
    type Err = ChannelDigestContractError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

impl<'de> serde::Deserialize<'de> for ChannelUsername {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = <String as serde::Deserialize>::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for ChannelUsername {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ChannelUsername")
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "title": "ChannelUsername",
            "description": "Canonical lowercase public Telegram channel username.",
            "pattern": Self::PATTERN,
            "minLength": 5,
            "maxLength": 32,
            "examples": ["example_channel"]
        })
    }
}
