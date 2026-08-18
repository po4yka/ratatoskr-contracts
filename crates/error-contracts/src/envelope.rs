//! The terminal error envelope and its non-terminal sibling.

use ratatoskr_identifiers::{EntityRef, Extensions, SafeMessage};

use crate::code::ErrorCode;
use crate::field::{FieldPath, FieldViolation};
use crate::trace::TraceId;

/// A terminal, machine-actionable failure crossing a process boundary
/// (`ARCHITECTURE.md` S5.5).
///
/// Contains no stack trace, no raw provider response, no credential and no storage path
/// (`ARCHITECTURE.md` S14, `SECURITY.md`). See spec D8 for the removal of S5.5's
/// `details: Option<serde_json::Value>`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ErrorEnvelope {
    /// Stable machine-actionable code. The only field a consumer may branch on.
    pub code: ErrorCode,

    /// Human-readable explanation for an operator or an end user. Not machine-parsed and not
    /// stable across releases; changing it is not a contract change.
    pub message: SafeMessage,

    /// Whether repeating the identical request may succeed later without operator action.
    /// Explicit, never inferred from `code` by the consumer (`ARCHITECTURE.md` S5.5:
    /// "retryability is explicit").
    pub retryable: bool,

    /// Field-level violations. Non-empty only for validation failures.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_violations: Vec<FieldViolation>,

    /// The unit of user-visible work this failure belongs to, e.g. `operation:018f…`.
    /// Required by `INTERFACES.md` ("Error envelopes expose stable code, safe message, retry
    /// class, correlation ID, and partial effects"); additive to S5.5.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<EntityRef>,

    /// W3C trace-id of the failing request, for log correlation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl ErrorEnvelope {
    /// The three members every failure must carry. Everything else is additive and starts empty.
    #[must_use]
    pub fn new(code: ErrorCode, message: SafeMessage, retryable: bool) -> Self {
        Self {
            code,
            message,
            retryable,
            field_violations: Vec::new(),
            correlation_id: None,
            trace_id: None,
            extensions: Extensions::new(),
        }
    }
}

/// A non-terminal problem that did not prevent the recorded outcome.
///
/// A distinct type, not a flag on [`ErrorEnvelope`]: `ARCHITECTURE.md` S5.5 requires
/// "partial-success warnings are distinct from terminal errors", and a shared type with a
/// severity field would let a producer emit a "warning" that consumers treat as fatal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WarningEnvelope {
    /// Stable machine-actionable code, same grammar as an error code.
    pub code: ErrorCode,

    /// User-safe explanation.
    pub message: SafeMessage,

    /// Where in the input the problem was observed, when a path applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_path: Option<FieldPath>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}
