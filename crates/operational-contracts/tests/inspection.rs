//! Operational inspection contract boundary tests.

use ratatoskr_operational_contracts::{
    AuditEventPage, OperationInspectionPage, ScheduleInspectionPage,
};
use serde_json::{Value, json};

fn operation() -> Value {
    json!({
        "operation_id": "018f0000-0000-7000-8000-000000000001",
        "owner_user_id": "018f0000-0000-7000-8000-000000000002",
        "kind": "content.document.extract",
        "status": "failed",
        "accepted_at": "2026-08-27T09:00:00Z",
        "status_changed_at": "2026-08-27T09:01:00Z",
        "failure_code": "platform.operation.failed"
    })
}

fn schedule() -> Value {
    json!({
        "schedule_id": "018f0000-0000-7000-8000-000000000003",
        "service_name": "telegram",
        "name": "daily_sync",
        "owner_user_id": "018f0000-0000-7000-8000-000000000002",
        "next_due_at": "2026-08-28T09:00:00Z",
        "enabled": false,
        "last_outcome": "failed"
    })
}

fn audit_event() -> Value {
    json!({
        "audit_event_id": "018f0000-0000-7000-8000-000000000004",
        "occurred_at": "2026-08-27T09:02:00Z",
        "action": "operation.read",
        "target_kind": "operation",
        "target_id": "018f0000-0000-7000-8000-000000000001",
        "outcome": "allowed",
        "correlation_id": "operation:018f0000-0000-7000-8000-000000000001"
    })
}

#[test]
fn operational_pages_are_bounded_and_content_free() -> Result<(), Box<dyn std::error::Error>> {
    let operation_page = json!({"items": [operation()], "next_cursor": "next-page"});
    let schedule_page = json!({"items": [schedule()]});
    let audit_page = json!({"items": [audit_event()], "next_cursor": "audit-page"});

    serde_json::from_value::<OperationInspectionPage>(operation_page)?;
    serde_json::from_value::<ScheduleInspectionPage>(schedule_page)?;
    serde_json::from_value::<AuditEventPage>(audit_page)?;

    let oversized = json!({"items": vec![operation(); 101]});
    assert!(
        serde_json::from_value::<OperationInspectionPage>(oversized).is_err(),
        "an inspection page must reject more than 100 rows"
    );

    for private in [
        json!({"items": [{
            "operation_id": "018f0000-0000-7000-8000-000000000001",
            "owner_user_id": "018f0000-0000-7000-8000-000000000002",
            "kind": "content.document.extract", "status": "failed",
            "accepted_at": "2026-08-27T09:00:00Z", "status_changed_at": "2026-08-27T09:01:00Z",
            "request_payload": {"private_url": "https://secret.invalid"}
        }]}),
        json!({"items": [{
            "schedule_id": "018f0000-0000-7000-8000-000000000003",
            "service_name": "telegram", "name": "daily_sync",
            "owner_user_id": "018f0000-0000-7000-8000-000000000002",
            "next_due_at": "2026-08-28T09:00:00Z", "enabled": true,
            "configuration": {"token": "secret"}
        }]}),
        json!({"items": [{
            "audit_event_id": "018f0000-0000-7000-8000-000000000004",
            "occurred_at": "2026-08-27T09:02:00Z", "action": "operation.read",
            "target_kind": "operation", "outcome": "allowed",
            "correlation_id": "operation:018f0000-0000-7000-8000-000000000001",
            "diagnostic": "bearer secret"
        }]}),
    ] {
        let rejected = serde_json::from_value::<OperationInspectionPage>(private.clone()).is_err()
            && serde_json::from_value::<ScheduleInspectionPage>(private.clone()).is_err()
            && serde_json::from_value::<AuditEventPage>(private).is_err();
        assert!(
            rejected,
            "private fields must not match any inspection page"
        );
    }

    Ok(())
}
