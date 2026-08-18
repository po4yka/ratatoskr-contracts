//! The stable, machine-actionable error code.

use ratatoskr_identifiers::wire_string_newtype;

wire_string_newtype! {
    /// Stable, machine-actionable error code: 2–4 dot-separated snake_case segments, e.g.
    /// `platform.operation.not_found`.
    ///
    /// The code is the contract; the message is not. A consumer branches on this and on nothing
    /// else (`AGENTS.md` principle 7: "Separate stable error codes from human-readable messages
    /// and provider diagnostics"). Service-owned: `ARCHITECTURE.md` S5.5 requires provider
    /// responses to be normalized into service-owned codes, so a provider's own code never
    /// appears here.
    pub struct ErrorCode {
        pattern  = r"^[a-z][a-z0-9_]{0,31}(\.[a-z][a-z0-9_]{0,31}){1,3}$",
        max_len  = 128,
        examples = ["platform.operation.not_found", "content.extraction.unsupported_media"],
    }
}

impl ErrorCode {
    /// First segment — the owning bounded context, e.g. `platform`.
    ///
    /// `PATTERN` guarantees at least one separator, so the head is always a whole segment.
    #[must_use]
    pub fn bounded_context(&self) -> &str {
        self.as_str()
            .split_once('.')
            .map_or_else(|| self.as_str(), |(head, _)| head)
    }
}
