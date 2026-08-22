//! [`Platform`]: where a social source came from.

use ratatoskr_identifiers::wire_string_newtype;

wire_string_newtype! {
    /// The platform a social source originates from, e.g. `x`, `instagram`, `threads`.
    ///
    /// **Open on purpose.** A validated token, not an enum: a platform added by a later
    /// milestone must not break a running consumer, and no consumer may assume the vocabulary
    /// is exhaustive. Branch on equality with known tokens; treat everything else generically.
    /// The grammar is the event-type segment grammar (`EventType::SEGMENT_PATTERN`), so one
    /// snake_case alphabet covers both.
    pub struct Platform {
        pattern  = r"^[a-z][a-z0-9_]{0,31}$",
        max_len  = 32,
        examples = ["x", "instagram", "threads"],
    }
}
