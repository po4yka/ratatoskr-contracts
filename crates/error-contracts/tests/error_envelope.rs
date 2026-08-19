//! `ErrorEnvelope` — tests X-2, X-3 and X-4.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_error_contracts::{ErrorCode, ErrorEnvelope, FieldPath, FieldViolation, TraceId};
use ratatoskr_identifiers::{
    EntityRef, Extensions, SafeMessage, canonical_json, dropped_field_pointers,
};

fn code(raw: &str) -> ErrorCode {
    ErrorCode::parse(raw).expect("a legal error code")
}

fn message(raw: &str) -> SafeMessage {
    SafeMessage::parse(raw).expect("a legal safe message")
}

/// X-2. A field path names structure only, so a rejected **value** — user content or a
/// credential — cannot ride out of the service inside an error payload.
#[test]
fn field_path_rejects_embedded_values() {
    assert!(FieldPath::parse("/blocks/3/text").is_ok());
    assert!(FieldPath::parse("/tenant_id").is_ok());

    // The rejected value is exactly what must never appear.
    assert!(FieldPath::parse("/tenant_id=alice@example.com").is_err());
    assert!(FieldPath::parse("/message: hello world").is_err());
    // Not a pointer at all.
    assert!(FieldPath::parse("tenant_id").is_err());
    assert!(FieldPath::parse("/").is_err());
    assert!(FieldPath::parse("").is_err());

    assert!(serde_json::from_str::<FieldPath>("\"/tenant_id=alice@example.com\"").is_err());
}

/// X-3. A stack trace is multi-line, so banning control characters makes one unrepresentable
/// (`ARCHITECTURE.md` S5.5, S14).
#[test]
fn message_rejects_control_characters_and_over_length() {
    assert!(SafeMessage::parse("The requested document does not exist.").is_ok());

    assert!(SafeMessage::parse("boom\n  at Service.run(Service.java:42)").is_err());
    assert!(SafeMessage::parse("carriage\rreturn").is_err());
    assert!(SafeMessage::parse("tab\tseparated").is_err());
    assert!(SafeMessage::parse("nul\u{0}byte").is_err());
    assert!(SafeMessage::parse("delete\u{7f}char").is_err());
    assert!(SafeMessage::parse("").is_err());

    assert!(SafeMessage::parse(&"a".repeat(1024)).is_ok());
    assert!(SafeMessage::parse(&"a".repeat(1025)).is_err());

    // The same ban applies through the wire, not only through the constructor.
    assert!(serde_json::from_str::<SafeMessage>("\"first\\nsecond\"").is_err());
}

/// X-4. `ARCHITECTURE.md` S5.5: validation errors carry safe field paths. The whole envelope
/// survives a canonical round trip with nothing dropped.
#[test]
fn roundtrips_with_field_violations() {
    let mut extensions = Extensions::new();
    extensions.insert("remediation_hint", serde_json::json!("Retry with a PDF."));

    let envelope = ErrorEnvelope {
        code: code("content.extraction.invalid_request"),
        message: message("The request could not be validated."),
        retryable: false,
        field_violations: vec![
            FieldViolation {
                field_path: FieldPath::parse("/blocks/3/text").expect("a legal path"),
                code: code("content.extraction.empty_block"),
                message: message("This block carries no text."),
                extensions: Extensions::new(),
            },
            FieldViolation {
                field_path: FieldPath::parse("/tenant_id").expect("a legal path"),
                code: code("platform.tenant.unknown_kind"),
                message: message("The owner kind is not recognised."),
                extensions: Extensions::new(),
            },
        ],
        correlation_id: Some(
            EntityRef::parse("operation:018f0000-0000-7000-8000-000000000010")
                .expect("a legal reference"),
        ),
        trace_id: Some(TraceId::parse("4bf92f3577b34da6a3ce929d0e0e4736").expect("a legal id")),
        extensions,
    };

    let rendered = canonical_json(&envelope).expect("canonical rendering");
    assert!(rendered.ends_with("}\n"));
    let decoded: ErrorEnvelope = serde_json::from_str(&rendered).expect("round trip");
    assert_eq!(decoded, envelope);
    assert_eq!(
        canonical_json(&decoded).expect("canonical rendering"),
        rendered
    );

    let input: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    let roundtripped: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&decoded).unwrap()).unwrap();
    assert_eq!(
        dropped_field_pointers(&input, &roundtripped),
        Vec::<String>::new()
    );

    // Field order is the documented wire order, and the preserved key follows the known ones.
    let order: Vec<usize> = [
        "code",
        "message",
        "retryable",
        "field_violations",
        "correlation_id",
        "trace_id",
        "remediation_hint",
    ]
    .iter()
    .map(|name| rendered.find(name).expect("every member is rendered"))
    .collect();
    // `is_sorted_by` rather than `windows(2)` and two indexes: it says "strictly increasing"
    // directly, and it needs no `#[allow(clippy::indexing_slicing)]` to say it.
    assert!(order.is_sorted_by(|a, b| a < b));
}

/// The three members every failure must carry; everything else starts empty and is omitted.
#[test]
fn new_builds_the_minimal_envelope() {
    let envelope = ErrorEnvelope::new(
        code("platform.operation.not_found"),
        message("The requested operation does not exist."),
        false,
    );
    assert_eq!(
        serde_json::to_string(&envelope).unwrap(),
        concat!(
            r#"{"code":"platform.operation.not_found","#,
            r#""message":"The requested operation does not exist.","retryable":false}"#
        )
    );
    assert!(envelope.field_violations.is_empty());
    assert_eq!(envelope.correlation_id, None);
    assert_eq!(envelope.trace_id, None);
    assert!(envelope.extensions.is_empty());
}
