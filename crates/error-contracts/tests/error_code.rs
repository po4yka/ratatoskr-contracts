//! `ErrorCode` — test X-1. A code is machine-actionable structure, not prose.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_error_contracts::ErrorCode;

/// X-1. Two to four `snake_case` segments; the first segment is the owning bounded context.
#[test]
fn parses_namespaced_code_and_rejects_single_segment_or_uppercase() {
    let code = ErrorCode::parse("platform.operation.not_found").expect("a documented code parses");
    assert_eq!(code.as_str(), "platform.operation.not_found");
    assert_eq!(code.bounded_context(), "platform");
    assert_eq!(code.to_string(), "platform.operation.not_found");

    // Two, three and four segments are all inside the grammar.
    assert_eq!(
        ErrorCode::parse("platform.not_found")
            .expect("two segments")
            .bounded_context(),
        "platform"
    );
    assert!(ErrorCode::parse("content.extraction.media.unsupported").is_ok());

    // A single segment carries no bounded context, so a consumer cannot route on it.
    assert!(ErrorCode::parse("not_found").is_err());
    // Five segments are beyond the published grammar.
    assert!(ErrorCode::parse("a.b.c.d.e").is_err());
    // Uppercase is a second spelling of one code, which defeats machine matching.
    assert!(ErrorCode::parse("Platform.Operation.NotFound").is_err());
    // Hyphens belong to entity kinds, not to codes.
    assert!(ErrorCode::parse("platform.operation.not-found").is_err());
    assert!(ErrorCode::parse("").is_err());

    // The wire form is the validated text, with no second spelling.
    let wire = serde_json::to_string(&code).unwrap();
    assert_eq!(wire, "\"platform.operation.not_found\"");
    assert_eq!(serde_json::from_str::<ErrorCode>(&wire).unwrap(), code);
    assert!(serde_json::from_str::<ErrorCode>("\"NOPE\"").is_err());
}
