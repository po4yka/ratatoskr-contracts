//! Public status contract boundary tests.

use ratatoskr_operational_contracts::{
    AUDIT_INSPECT_CAPABILITY, OPERATIONS_INSPECT_CAPABILITY, PLATFORM_OWNER_GRANT,
    PublicStatusDocument, SCHEDULES_INSPECT_CAPABILITY,
};
use serde_json::json;

#[test]
fn public_status_rejects_unknown_or_unsanitized_components()
-> Result<(), Box<dyn std::error::Error>> {
    let valid = json!({
        "generated_at": "2026-08-27T10:00:00Z",
        "state": "degraded",
        "components": [
            {"id": "api", "state": "operational", "observed_at": "2026-08-27T10:00:00Z", "stale": false},
            {"id": "storage", "state": "operational", "observed_at": "2026-08-27T09:59:59Z", "stale": false},
            {"id": "command_delivery", "state": "degraded", "observed_at": "2026-08-27T09:58:00Z", "stale": true},
            {"id": "connected_services", "state": "unknown", "stale": false}
        ]
    });

    let parsed: PublicStatusDocument = serde_json::from_value(valid.clone())?;
    assert_eq!(serde_json::to_value(parsed)?, valid);

    for invalid in [
        json!({
            "generated_at": "2026-08-27T10:00:00Z",
            "state": "degraded",
            "components": [
                {"id": "postgres-primary", "state": "degraded", "stale": true}
            ]
        }),
        json!({
            "generated_at": "2026-08-27T10:00:00Z",
            "state": "yellow",
            "components": []
        }),
        json!({
            "generated_at": "2026-08-27T10:00:00Z",
            "state": "degraded",
            "components": [
                {"id": "api", "state": "degraded", "stale": true, "diagnostic": "nats://internal:4222"}
            ]
        }),
    ] {
        assert!(
            serde_json::from_value::<PublicStatusDocument>(invalid).is_err(),
            "unknown and private status members must be rejected"
        );
    }

    assert_eq!(PLATFORM_OWNER_GRANT, "platform.owner");
    assert_eq!(OPERATIONS_INSPECT_CAPABILITY, "platform.operations.inspect");
    assert_eq!(SCHEDULES_INSPECT_CAPABILITY, "platform.schedules.inspect");
    assert_eq!(AUDIT_INSPECT_CAPABILITY, "platform.audit.inspect");
    Ok(())
}
