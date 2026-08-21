//! `OperationReported` producer-to-Platform payload tests.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_operation_contracts::OperationReported;

#[test]
fn a_report_without_a_status_is_refused() {
    let error = serde_json::from_value::<OperationReported>(serde_json::json!({
        "operation_id": "018f0000-0000-7000-8000-000000000010"
    }))
    .expect_err("status is required");

    assert!(
        error.to_string().contains("missing field `status`"),
        "{error}"
    );
}

#[test]
fn a_report_carries_no_snapshot_only_fields() {
    let schema = serde_json::to_value(schemars::schema_for!(OperationReported))
        .expect("the derived schema serializes");
    let properties = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .expect("OperationReported is an object schema");
    let description = schema
        .get("description")
        .and_then(serde_json::Value::as_str)
        .expect("OperationReported documents its boundary");

    for absent in ["kind", "accepted_at", "correlation_id", "tenant_id"] {
        assert!(
            !properties.contains_key(absent),
            "{absent} must stay Platform-owned"
        );
        assert!(
            description.contains(absent),
            "the rustdoc must explain why {absent} is absent"
        );
    }
}
