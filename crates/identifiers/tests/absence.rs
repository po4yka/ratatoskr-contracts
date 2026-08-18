//! Absence semantics — test I-10. `DOMAIN.md` invariant 4, spec D12: `null` == absent == `None`,
//! and `None` always serializes as absent.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::{EventId, SafeMessage, TenantRef, canonical_json};

/// A stand-in for any wire struct in this repository: the optionality attribute is the
/// repository-wide rule, applied here to the identifier types that carry it.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct OptionalityProbe {
    event_id: EventId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tenant_id: Option<TenantRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    detail_message: Option<SafeMessage>,
}

/// I-10.
#[test]
fn null_and_absent_are_both_none_and_serialize_absent() {
    let absent: OptionalityProbe =
        serde_json::from_str(r#"{"event_id":"018f0000-0000-7000-8000-000000000001"}"#)
            .expect("an absent optional member is None");
    let explicit_null: OptionalityProbe = serde_json::from_str(
        r#"{"event_id":"018f0000-0000-7000-8000-000000000001","tenant_id":null,"detail_message":null}"#,
    )
    .expect("an explicit null is the same thing");

    assert_eq!(absent, explicit_null);
    assert_eq!(absent.tenant_id, None);
    assert_eq!(absent.detail_message, None);

    // `None` always serializes as absent, so there is exactly one wire form.
    let rendered = serde_json::to_string(&explicit_null).unwrap();
    assert_eq!(
        rendered,
        r#"{"event_id":"018f0000-0000-7000-8000-000000000001"}"#
    );
    assert!(!rendered.contains("null"));
}

/// A present optional member survives the round trip and stays in declaration order, which is the
/// documented wire order.
#[test]
fn present_optional_members_keep_declaration_order() {
    let probe = OptionalityProbe {
        event_id: EventId::parse("018f0000-0000-7000-8000-000000000001").unwrap(),
        tenant_id: Some(TenantRef::parse("user:018f0000-0000-7000-8000-000000000005").unwrap()),
        detail_message: Some(SafeMessage::parse("The requested document does not exist.").unwrap()),
    };
    let rendered = canonical_json(&probe).expect("canonical rendering");
    assert!(
        rendered.ends_with("}\n"),
        "canonical JSON ends with exactly one newline"
    );

    let event_at = rendered.find("event_id").expect("present");
    let tenant_at = rendered.find("tenant_id").expect("present");
    let message_at = rendered.find("detail_message").expect("present");
    assert!(event_at < tenant_at && tenant_at < message_at);

    let decoded: OptionalityProbe = serde_json::from_str(&rendered).expect("round trip");
    assert_eq!(decoded, probe);
}
