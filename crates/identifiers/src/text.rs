//! Human-readable wire text that cannot carry a stack trace.

use crate::wire_string_newtype;

wire_string_newtype! {
    /// Human-readable text safe to show to an operator or an end user.
    ///
    /// 1..=1024 characters with no C0 or DEL control characters. The newline ban is the point: it
    /// makes it structurally impossible to smuggle a stack trace or a forged log line into a wire
    /// message (`ARCHITECTURE.md` S5.5 "no stack traces or secrets in wire errors", S14).
    ///
    /// Not machine-parsed and not stable across releases; changing a message is not a contract
    /// change. Consumers branch on codes, never on this.
    pub struct SafeMessage {
        pattern  = r"^[^\x00-\x1f\x7f]{1,1024}$",
        max_len  = 1024,
        examples = ["The requested document does not exist."],
    }
}
