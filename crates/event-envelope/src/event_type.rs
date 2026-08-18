//! The validated event name: [`EventType`] and [`EventTypeError`].

use core::convert::TryFrom;
use core::fmt;
use core::str::FromStr;

/// A parsed `<bounded_context>.<aggregate>.<action>.v<major>` event type
/// (`ARCHITECTURE.md` S9.1, `README.md`, `AGENTS.md`).
///
/// Never a bare `String`: the grammar and the major version are part of the contract (ADR-0002).
/// Example: `content.document.extracted.v1`.
///
/// The `action` segment is past tense because events are completed facts
/// (`AGENTS.md` principle 9). Tense is **not** enforced by this parser — a consumer must not fail
/// to read a producer's event over English grammar — it is enforced by `cargo contracts check`
/// over *registered* event types through [`EventType::action_looks_past_tense`].
///
/// Segments are `snake_case` with no hyphens: every example in `README.md` and S9.1 is snake, and
/// the one-character context `x` (from `x.bookmark.observed.v1`) must parse. Entity kinds use a
/// different, hyphen-permitting alphabet (`EntityKind::PATTERN`) because `README.md` shows
/// `x-post`. Keeping the two alphabets distinct is honest to the documents rather than inventing
/// a union.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct EventType {
    bounded_context: String,
    aggregate: String,
    action: String,
    major: u32,
}

/// The highest payload major the wire grammar can spell, from `PATTERN`'s `[1-9][0-9]{0,3}`.
const MAX_MAJOR: u32 = 9999;

/// The lowest payload major the wire grammar can spell. There is no `v0`.
const MIN_MAJOR: u32 = 1;

impl EventType {
    /// Grammar of one name segment, published as the JSON Schema `pattern` of a bare segment and
    /// compiled as the runtime validator, so the two cannot drift.
    pub const SEGMENT_PATTERN: &'static str = r"^[a-z][a-z0-9_]{0,31}$";

    /// Grammar of the whole event type, published as this type's JSON Schema `pattern`.
    pub const PATTERN: &'static str = concat!(
        r"^[a-z][a-z0-9_]{0,31}\.[a-z][a-z0-9_]{0,31}\.[a-z][a-z0-9_]{0,31}",
        r"\.v[1-9][0-9]{0,3}$",
    );

    /// Contract maximum length in UTF-8 bytes, published as JSON Schema `maxLength`. `PATTERN`
    /// also bounds it; this exists so an over-long input reports [`EventTypeError::TooLong`]
    /// instead of an unhelpful pattern mismatch.
    pub const MAX_LEN: usize = 128;

    /// [`Self::SEGMENT_PATTERN`], compiled once.
    #[allow(
        clippy::expect_used,
        reason = "SEGMENT_PATTERN is a compile-time contract constant; a build whose pattern \
                  does not compile is broken before it reaches a wire"
    )]
    fn compiled_segment_pattern() -> &'static regex::Regex {
        static COMPILED: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(EventType::SEGMENT_PATTERN)
                .expect("contract SEGMENT_PATTERN must be a valid regular expression")
        });
        &COMPILED
    }

    /// Parses the canonical wire form.
    ///
    /// # Errors
    ///
    /// [`EventTypeError::TooLong`] beyond 128 UTF-8 bytes,
    /// [`EventTypeError::WrongShape`] when there are not exactly four dot-separated segments,
    /// [`EventTypeError::InvalidSegment`] for a malformed name segment,
    /// [`EventTypeError::InvalidMajor`] for `v0`, `v01` or a missing `v`.
    pub fn parse(raw: &str) -> Result<Self, EventTypeError> {
        if raw.len() > Self::MAX_LEN {
            return Err(EventTypeError::TooLong {
                input: raw.to_owned(),
                got: raw.len(),
            });
        }
        let mut segments = raw.split('.');
        let (Some(bounded_context), Some(aggregate), Some(action), Some(version), None) = (
            segments.next(),
            segments.next(),
            segments.next(),
            segments.next(),
            segments.next(),
        ) else {
            return Err(EventTypeError::WrongShape {
                input: raw.to_owned(),
            });
        };
        Ok(Self {
            bounded_context: Self::checked_segment(bounded_context)?,
            aggregate: Self::checked_segment(aggregate)?,
            action: Self::checked_segment(action)?,
            major: Self::checked_major(raw, version)?,
        })
    }

    /// Builds a name from its parts, validating exactly as [`Self::parse`] does.
    ///
    /// # Errors
    ///
    /// The same errors as [`Self::parse`], because the parts are joined and re-parsed.
    pub fn new(
        bounded_context: &str,
        aggregate: &str,
        action: &str,
        major: u32,
    ) -> Result<Self, EventTypeError> {
        Self::parse(&format!("{bounded_context}.{aggregate}.{action}.v{major}"))
    }

    /// The bounded context that owns the fact, e.g. `content`.
    #[must_use]
    pub fn bounded_context(&self) -> &str {
        &self.bounded_context
    }

    /// The aggregate the fact is about, e.g. `document`.
    #[must_use]
    pub fn aggregate(&self) -> &str {
        &self.aggregate
    }

    /// The past-tense action, e.g. `extracted`.
    #[must_use]
    pub fn action(&self) -> &str {
        &self.action
    }

    /// The payload contract major version (`ARCHITECTURE.md` S5.2: "Payloads are versioned
    /// independently through `event_type` major versions"). Never zero: `v0` is unparseable.
    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }

    /// Version-free family key, e.g. `content.document.extracted`. Two names with the same family
    /// describe the same fact at different payload majors.
    #[must_use]
    pub fn family(&self) -> String {
        format!(
            "{}.{}.{}",
            self.bounded_context, self.aggregate, self.action
        )
    }

    /// The only sanctioned mutation: bump the payload major, keep the fact.
    /// `AGENTS.md`: "Do not rename existing event types for style."
    ///
    /// The wire grammar can spell majors `1..=9999` only. An argument outside that range is a
    /// **caller bug**, not a supported input: a major computed arithmetically (`current - 1` on a
    /// rollback path) that lands on `0` does not mean `v1`. It fails the debug assertion in every
    /// test and debug build; in a release build it is clamped into range rather than producing a
    /// name that cannot be serialized.
    #[must_use]
    pub fn with_major(&self, major: u32) -> Self {
        debug_assert!(
            (MIN_MAJOR..=MAX_MAJOR).contains(&major),
            "event-type major {major} is outside the wire grammar's {MIN_MAJOR}..={MAX_MAJOR}"
        );
        Self {
            major: major.clamp(MIN_MAJOR, MAX_MAJOR),
            ..self.clone()
        }
    }

    /// The canonical wire rendering, byte-identical to the input [`Self::parse`] accepted.
    #[must_use]
    pub fn to_wire(&self) -> String {
        format!(
            "{}.{}.{}.v{}",
            self.bounded_context, self.aggregate, self.action, self.major
        )
    }

    /// Heuristic used by `cargo contracts check`; never consulted while parsing.
    ///
    /// Deliberately crude: it recognises the regular `-ed` ending only. Every event type
    /// registered in `contracts.toml` today ends that way, and an irregular past tense
    /// (`sent`, `built`) is a review conversation rather than a vocabulary this repository
    /// invents.
    #[must_use]
    pub fn action_looks_past_tense(&self) -> bool {
        self.action.ends_with("ed")
    }

    /// Validates one name segment against [`Self::SEGMENT_PATTERN`].
    fn checked_segment(segment: &str) -> Result<String, EventTypeError> {
        if Self::compiled_segment_pattern().is_match(segment) {
            Ok(segment.to_owned())
        } else {
            Err(EventTypeError::InvalidSegment {
                segment: segment.to_owned(),
                pattern: Self::SEGMENT_PATTERN,
            })
        }
    }

    /// Validates the `v<major>` segment: a literal `v`, then `[1-9][0-9]{0,3}`.
    fn checked_major(input: &str, version: &str) -> Result<u32, EventTypeError> {
        let invalid = || EventTypeError::InvalidMajor {
            input: input.to_owned(),
        };
        let digits = version.strip_prefix('v').ok_or_else(invalid)?;
        let legal = !digits.is_empty()
            && digits.len() <= 4
            && !digits.starts_with('0')
            && digits.bytes().all(|byte| byte.is_ascii_digit());
        if !legal {
            return Err(invalid());
        }
        digits.parse::<u32>().map_err(|_| invalid())
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.v{}",
            self.bounded_context, self.aggregate, self.action, self.major
        )
    }
}

impl FromStr for EventType {
    type Err = EventTypeError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

impl TryFrom<String> for EventType {
    type Error = EventTypeError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<EventType> for String {
    fn from(value: EventType) -> Self {
        value.to_wire()
    }
}

impl schemars::JsonSchema for EventType {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("EventType")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::EventType"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "title": "EventType",
            "description": "An event name, `<bounded_context>.<aggregate>.<action>.v<major>`. \
                            The `.v<major>` suffix versions the payload contract, not the \
                            envelope. Segments are snake_case; the action is past tense because \
                            an event is a completed fact.",
            "pattern": Self::PATTERN,
            "maxLength": Self::MAX_LEN,
            "examples": ["content.document.extracted.v1", "platform.operation.progressed.v1"],
        })
    }
}

/// Every way an event name can fail to parse.
///
/// A *parse* error type: it is never serialized onto the wire, so echoing the offending input
/// here cannot leak into a broadcast payload.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum EventTypeError {
    /// The name is longer than [`EventType::MAX_LEN`] UTF-8 bytes.
    #[error("event type {input:?} is {got} bytes; the maximum is 128")]
    TooLong {
        /// The offending input, echoed for diagnosis only.
        input: String,
        /// Length of the offending input in UTF-8 bytes.
        got: usize,
    },
    /// The name does not have exactly four dot-separated segments.
    #[error("event type {input:?} must be `<bounded_context>.<aggregate>.<action>.v<major>`")]
    WrongShape {
        /// The offending input, echoed for diagnosis only.
        input: String,
    },
    /// One of the three name segments violates [`EventType::SEGMENT_PATTERN`].
    #[error("event type segment {segment:?} does not match {pattern}")]
    InvalidSegment {
        /// The offending segment, echoed for diagnosis only.
        segment: String,
        /// The published segment pattern, which is also the compiled runtime validator.
        pattern: &'static str,
    },
    /// The version segment is not `v<major>` with a major that starts at 1 and carries no
    /// leading zeros.
    #[error(
        "event type {input:?} has an invalid major version; majors start at 1 and carry no \
         leading zeros"
    )]
    InvalidMajor {
        /// The offending input, echoed for diagnosis only.
        input: String,
    },
}
