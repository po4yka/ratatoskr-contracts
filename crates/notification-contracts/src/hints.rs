//! Advisory delivery hints: [`NotificationPriority`] and [`QuietHoursHint`].

use crate::error::NotificationContractError;

/// How urgently a producer judges its notification should reach the user.
///
/// A **hint, never an order**: enforcement stays with `ratatoskr-telegram`, which applies it
/// through the user's own preferences. Absent means the consumer's default ordering applies.
///
/// **Closed on purpose**, mirroring `BackupPriorityHint`: a guessed priority silently reorders
/// delivery, so an unrecognized value must stop processing rather than be read as some default.
/// Adding a level later is an additive, non-breaking expansion governed by the payload major,
/// not by the class registry — priority is not part of the taxonomy.
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
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum NotificationPriority {
    /// Deliver only when the channel is otherwise idle; the news keeps.
    Low,
    /// Ordinary delivery.
    Normal,
    /// Deliver promptly and let it break through quiet hours where the user's own preferences
    /// allow that at all.
    High,
}

/// A recurring daily window during which delivery should be held back, expressed as two offsets
/// from UTC midnight in seconds.
///
/// Advisory only: the producer states a window it knows or infers about the recipient;
/// interpreting it against the user's real timezone and preferences is the consumer's job, which
/// is why the wire carries offsets and never a timezone. Absent means no window is asserted.
///
/// Each offset names a second inside one UTC day: 0..=[`Self::MAX_OFFSET_SECONDS`]. A start
/// greater than the end is the normal overnight shape and wraps past midnight; equal bounds are
/// refused because such a window cannot say whether it means an empty day or a full one, and no
/// consumer should guess between opposite behaviours.
///
/// `Deserialize` is hand-written over a private mirror so both offsets pass through
/// [`Self::new`]'s checks on the way in; the published JSON Schema carries the same numeric
/// bounds, so an out-of-range offset is refused at **both** layers while the inequality stays
/// Rust-only (draft 2020-12 cannot compare two properties).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct QuietHoursHint {
    start_offset_seconds: u32,
    end_offset_seconds: u32,
}

impl QuietHoursHint {
    /// The largest offset one UTC day can name: `86_399` seconds.
    pub const MAX_OFFSET_SECONDS: u32 = 86_399;

    /// Builds a validated window from two offsets in seconds from UTC midnight.
    ///
    /// # Errors
    ///
    /// [`NotificationContractError::QuietHoursOffsetOutOfRange`] when either offset exceeds
    /// [`Self::MAX_OFFSET_SECONDS`], [`NotificationContractError::QuietHoursWindowDegenerate`]
    /// when the bounds are equal.
    pub fn new(
        start_offset_seconds: u32,
        end_offset_seconds: u32,
    ) -> Result<Self, NotificationContractError> {
        for offset in [start_offset_seconds, end_offset_seconds] {
            if offset > Self::MAX_OFFSET_SECONDS {
                return Err(NotificationContractError::QuietHoursOffsetOutOfRange {
                    got: offset,
                    max: Self::MAX_OFFSET_SECONDS,
                });
            }
        }
        if start_offset_seconds == end_offset_seconds {
            return Err(NotificationContractError::QuietHoursWindowDegenerate);
        }
        Ok(Self {
            start_offset_seconds,
            end_offset_seconds,
        })
    }

    /// Seconds after UTC midnight at which the window opens.
    #[must_use]
    pub const fn start_offset_seconds(&self) -> u32 {
        self.start_offset_seconds
    }

    /// Seconds after UTC midnight at which the window closes; smaller than the start when the
    /// window wraps past midnight.
    #[must_use]
    pub const fn end_offset_seconds(&self) -> u32 {
        self.end_offset_seconds
    }
}

/// The wire mirror of [`QuietHoursHint`], checked by [`QuietHoursHint::new`] before the value
/// exists.
#[derive(Debug, serde::Deserialize)]
struct QuietHoursHintWire {
    start_offset_seconds: u32,
    end_offset_seconds: u32,
}

impl<'de> serde::Deserialize<'de> for QuietHoursHint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = QuietHoursHintWire::deserialize(deserializer)?;
        Self::new(wire.start_offset_seconds, wire.end_offset_seconds)
            .map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for QuietHoursHint {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("QuietHoursHint")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::QuietHoursHint"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "title": "QuietHoursHint",
            "description": "A recurring daily window during which notification delivery should be \
                            held back, advisory only. Two offsets from UTC midnight in seconds; a \
                            start greater than the end wraps past midnight. Equal bounds are \
                            refused by the Rust type - they cannot say whether they mean an empty \
                            day or a full one - which this schema cannot express.",
            "properties": {
                "start_offset_seconds": {
                    "type": "integer",
                    "format": "uint32",
                    "minimum": 0,
                    "maximum": Self::MAX_OFFSET_SECONDS,
                    "description": "Seconds after UTC midnight at which the window opens.",
                },
                "end_offset_seconds": {
                    "type": "integer",
                    "format": "uint32",
                    "minimum": 0,
                    "maximum": Self::MAX_OFFSET_SECONDS,
                    "description": "Seconds after UTC midnight at which the window closes; \
                                    smaller than the start when the window wraps past midnight.",
                },
            },
            "required": ["start_offset_seconds", "end_offset_seconds"],
        })
    }
}
