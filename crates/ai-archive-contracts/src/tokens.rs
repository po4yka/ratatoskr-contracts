//! The open tokens an archive carries: which provider, which parser, which parser build.

use ratatoskr_identifiers::wire_string_newtype;

wire_string_newtype! {
    /// The AI provider an archive came from, e.g. `chatgpt`, `claude`.
    ///
    /// **Open on purpose.** A validated token, not an enum: a provider added by a later milestone
    /// must not break a running consumer, and no consumer may assume the vocabulary is
    /// exhaustive. Branch on equality with known tokens; treat everything else generically. The
    /// grammar is the event-type segment grammar (`EventType::SEGMENT_PATTERN`), so one
    /// snake_case alphabet covers both.
    pub struct AiProvider {
        pattern  = r"^[a-z][a-z0-9_]{0,31}$",
        max_len  = 32,
        examples = ["chatgpt", "claude"],
    }
}

wire_string_newtype! {
    /// Which parser normalized a node, e.g. `chatgpt_export`, `claude_export`.
    ///
    /// **Open on purpose**, like [`AiProvider`]: a parser rename or addition must not break a
    /// running consumer. Opaque to consumers beyond display and staleness comparison; no
    /// consumer may branch on its internals.
    ///
    /// [`AiProvider`]: crate::AiProvider
    pub struct ParserName {
        pattern  = r"^[a-z][a-z0-9_]{0,31}$",
        max_len  = 32,
        examples = ["chatgpt_export", "claude_export"],
    }
}

wire_string_newtype! {
    /// The build of the [`ParserName`] that normalized a node.
    ///
    /// Bounded printable ASCII without whitespace: semver (`1.4.2`), date-based (`2026.08.1`)
    /// and commit-sha spellings all fit. Deliberately **not** semver-typed — parsers version in
    /// more than one scheme, and any tighter grammar would reject an honest stamp. Consumers MAY
    /// compare stamps for staleness; none may parse them for semantics.
    ///
    /// [`ParserName`]: crate::ParserName
    pub struct ParserVersion {
        pattern  = r"^[!-~]{1,64}$",
        max_len  = 64,
        examples = ["1.4.2", "2026.08.1"],
    }
}
