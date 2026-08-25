//! The notification-class taxonomy: recognition, preservation, grammar.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_notification_contracts::{NOTIFICATION_CLASS_REGISTRY_VERSION, NotificationClass};

/// Registry version 1 speaks exactly these six classes, sorted for stable review diffs.
#[test]
fn registry_version_one_speaks_six_known_classes() {
    assert_eq!(NOTIFICATION_CLASS_REGISTRY_VERSION, 1);
    assert_eq!(
        NotificationClass::KNOWN,
        &[
            "analysis_ready",
            "archive_imported",
            "backup_outcome",
            "operation_completed",
            "operation_failed",
            "watch_triggered",
        ]
    );
}

/// Every known class token maps to its named variant, reports itself known, and re-renders the
/// same token.
#[test]
fn known_classes_parse_to_named_variants() {
    let cases: [(&str, NotificationClass); 6] = [
        ("analysis_ready", NotificationClass::AnalysisReady),
        ("archive_imported", NotificationClass::ArchiveImported),
        ("backup_outcome", NotificationClass::BackupOutcome),
        ("operation_completed", NotificationClass::OperationCompleted),
        ("operation_failed", NotificationClass::OperationFailed),
        ("watch_triggered", NotificationClass::WatchTriggered),
    ];

    for (token, expected) in cases {
        let parsed = NotificationClass::parse(token).expect("a known token parses");
        assert!(
            matches!(parsed, ref got if *got == expected),
            "token `{token}` must map to its named variant, got {parsed}"
        );
        assert!(parsed.is_known(), "token `{token}` must report as known");
        assert_eq!(parsed.as_str(), token);
    }
}

/// A well-formed token this build does not know parses, reports unrecognized, and round trips
/// through serde byte-identically - preserved, never rejected.
///
/// Green on arrival by design: preservation is the placeholder parse's own behaviour made
/// precise. The test pins the property regressively, exactly like a golden-output pin.
#[test]
fn unknown_class_token_is_preserved() {
    let parsed = NotificationClass::parse("carrier_pigeon").expect("a well-formed token parses");
    assert!(
        !parsed.is_known(),
        "an unknown token must report unrecognized"
    );
    assert_eq!(parsed.as_str(), "carrier_pigeon");

    let wire = serde_json::to_string(&parsed).expect("serializes");
    assert_eq!(wire, r#""carrier_pigeon""#);
    let back: NotificationClass = serde_json::from_str(&wire).expect("deserializes");
    assert_eq!(back, parsed);
}

/// A token violating the grammar is neither recognizable nor safely preservable: both layers
/// reject it. Pinned on the schema side by `class-token-uppercase.json`.
#[test]
fn malformed_class_tokens_are_refused() {
    for token in ["Carrier_Pigeon", "", "a".repeat(33).as_str(), "has space"] {
        let error = NotificationClass::parse(token)
            .err()
            .unwrap_or_else(|| panic!("`{token}` must be refused"));
        assert!(
            error.to_string().contains("does not match"),
            "case `{token}` failed for the wrong reason: {error}"
        );
    }
}
