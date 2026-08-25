//! The versioned notification-class taxonomy: [`NotificationClass`] and
//! [`NOTIFICATION_CLASS_REGISTRY_VERSION`].

use crate::error::NotificationContractError;

/// Registry version of the class taxonomy this build speaks.
///
/// Starts at 1. It MUST be bumped whenever the known set below grows, so a consumer that knows
/// only version 1 can tell a recognized class from one it is merely preserving. The payload
/// carries the value as `class_registry_version`; this constant is what a producer stamps into
/// it.
pub const NOTIFICATION_CLASS_REGISTRY_VERSION: u32 = 1;

/// Which kind of thing happened, drawn from the notification-class taxonomy.
///
/// **Open on purpose**, unlike every other vocabulary in this crate. A class drives presentation
/// routing, not state transitions, and a dropped or mis-filed user signal is worse than an
/// unlabeled one: an unrecognized token is preserved verbatim in [`NotificationClass::Other`] so
/// a class added by a later producer still reaches its audience through a consumer built today.
/// Typos are caught out of band by `contracts.toml [entity_kinds].known`, which every fixture is
/// checked against.
///
/// The known set at registry version 1 — [`Self::KNOWN`], sorted for stable review diffs — is:
/// `operation_completed`, `operation_failed`, `analysis_ready`, `backup_outcome`,
/// `watch_triggered`, `archive_imported`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NotificationClass {
    /// A long-running operation reached its terminal state successfully.
    OperationCompleted,
    /// A long-running operation reached its terminal state unsuccessfully.
    OperationFailed,
    /// A knowledge analysis finished and its findings are ready to read.
    AnalysisReady,
    /// A backup cycle reported how a preservation run went.
    BackupOutcome,
    /// A user-defined watch rule matched new upstream activity.
    WatchTriggered,
    /// A provider AI-archive export finished importing as normalized evidence.
    ArchiveImported,
    /// A well-formed token this build does not know, preserved exactly as received.
    Other(String),
}

impl NotificationClass {
    /// Token grammar: `snake_case` segments, hyphens permitted, at most 32 UTF-8 bytes — the same
    /// alphabet `EntityKind::PATTERN` publishes for reference kinds.
    pub const PATTERN: &'static str = r"^[a-z][a-z0-9_-]{0,31}$";

    /// Contract maximum token length in UTF-8 bytes. `PATTERN` also bounds it.
    pub const MAX_LEN: usize = 32;

    /// The tokens this build maps to a named variant at registry version 1, sorted for stable
    /// review diffs.
    pub const KNOWN: &'static [&'static str] = &[
        "analysis_ready",
        "archive_imported",
        "backup_outcome",
        "operation_completed",
        "operation_failed",
        "watch_triggered",
    ];

    /// Parses a class token, validating the grammar and mapping known tokens onto their
    /// variants.
    ///
    /// The wire boundary is [`Self::parse`] — every deserialization routes through it — so an
    /// unrecognized but well-formed token survives as [`NotificationClass::Other`] while a
    /// malformed one stops processing.
    ///
    /// # Errors
    ///
    /// [`NotificationContractError::MalformedClassToken`] when the token violates
    /// [`Self::PATTERN`]: uppercase variants, over-length tokens and empty strings are typos or
    /// hostile input, not future vocabulary.
    pub fn parse(raw: &str) -> Result<Self, NotificationContractError> {
        if !Self::compiled_pattern().is_match(raw) {
            return Err(NotificationContractError::MalformedClassToken {
                pattern: Self::PATTERN,
                input: raw.to_owned(),
            });
        }
        Ok(Self::from_token(raw))
    }

    /// The published `PATTERN`, compiled once.
    #[allow(
        clippy::expect_used,
        reason = "PATTERN is a compile-time contract constant; a build whose pattern does not \
                  compile is broken before it reaches a wire"
    )]
    fn compiled_pattern() -> &'static regex::Regex {
        static COMPILED: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(NotificationClass::PATTERN)
                .expect("contract PATTERN must be a valid regular expression")
        });
        &COMPILED
    }

    /// Maps a token already known to satisfy `PATTERN` onto a variant.
    fn from_token(token: &str) -> Self {
        match token {
            "analysis_ready" => Self::AnalysisReady,
            "archive_imported" => Self::ArchiveImported,
            "backup_outcome" => Self::BackupOutcome,
            "operation_completed" => Self::OperationCompleted,
            "operation_failed" => Self::OperationFailed,
            "watch_triggered" => Self::WatchTriggered,
            other => Self::Other(other.to_owned()),
        }
    }

    /// The wire token for this class.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::OperationCompleted => "operation_completed",
            Self::OperationFailed => "operation_failed",
            Self::AnalysisReady => "analysis_ready",
            Self::BackupOutcome => "backup_outcome",
            Self::WatchTriggered => "watch_triggered",
            Self::ArchiveImported => "archive_imported",
            Self::Other(token) => token,
        }
    }

    /// `true` when this build understands the class rather than preserving it.
    #[must_use]
    pub const fn is_known(&self) -> bool {
        !matches!(self, Self::Other(_))
    }
}

impl core::fmt::Display for NotificationClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::str::FromStr for NotificationClass {
    type Err = NotificationContractError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

impl core::convert::TryFrom<String> for NotificationClass {
    type Error = NotificationContractError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<NotificationClass> for String {
    fn from(value: NotificationClass) -> Self {
        value.as_str().to_owned()
    }
}

impl serde::Serialize for NotificationClass {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for NotificationClass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for NotificationClass {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("NotificationClass")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::NotificationClass"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "title": "NotificationClass",
            "description": "A notification-class taxonomy token. Snake_case segments, at most 32 \
                            UTF-8 bytes. Unknown tokens are preserved verbatim, never rejected; \
                            an input that violates this pattern is a typo or hostile rather than \
                            future vocabulary.",
            "pattern": Self::PATTERN,
            "maxLength": Self::MAX_LEN,
            "examples": ["operation_completed", "analysis_ready"],
        })
    }
}
