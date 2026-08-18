//! Field-level validation failures and the safe paths that name them.

use ratatoskr_identifiers::{Extensions, SafeMessage, wire_string_newtype};

use crate::code::ErrorCode;

wire_string_newtype! {
    /// RFC 6901 JSON Pointer to the offending member of the rejected payload, restricted to
    /// identifier-shaped tokens, e.g. `/blocks/3/text`.
    ///
    /// Structure only. The restricted alphabet makes it structurally impossible to smuggle a
    /// rejected **value** (which may be user content or a credential) into an error payload —
    /// `/tenant_id=alice@example.com` does not parse. `ARCHITECTURE.md` S5.5: "validation errors
    /// identify safe field paths."
    pub struct FieldPath {
        pattern  = r"^(/[A-Za-z0-9_-]+)+$",
        max_len  = 256,
        examples = ["/blocks/3/text", "/tenant_id"],
    }
}

/// One field-level validation failure.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct FieldViolation {
    /// Safe path to the offending member. Never contains the rejected value.
    pub field_path: FieldPath,
    /// Stable code for this specific violation.
    pub code: ErrorCode,
    /// User-safe explanation. Must not echo the rejected value.
    pub message: SafeMessage,
    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}
