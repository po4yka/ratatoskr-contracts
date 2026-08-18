//! The log-correlation trace identity.

use ratatoskr_identifiers::wire_string_newtype;

wire_string_newtype! {
    /// W3C Trace Context `trace-id`: 32 lowercase hex characters.
    /// For log correlation only; never a business key and never an authorization input.
    pub struct TraceId {
        pattern  = r"^[0-9a-f]{32}$",
        max_len  = 32,
        examples = ["4bf92f3577b34da6a3ce929d0e0e4736"],
    }
}
