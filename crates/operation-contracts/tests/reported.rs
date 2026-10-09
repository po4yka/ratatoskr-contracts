//! `OperationReported` producer-to-Platform payload tests.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_error_contracts::{ErrorCode, ErrorEnvelope, WarningEnvelope};
use ratatoskr_identifiers::{Extensions, OperationId, SafeMessage};
use ratatoskr_operation_contracts::{OperationContractError, OperationReported, OperationStatus};

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

fn report(status: OperationStatus) -> OperationReported {
    OperationReported {
        operation_id: OperationId::parse("018f0000-0000-7000-8000-000000000010")
            .expect("a canonical UUID"),
        status,
        stage: None,
        progress_percent: None,
        results: Vec::new(),
        error: None,
        warnings: Vec::new(),
        extensions: Extensions::new(),
    }
}

fn failure() -> ErrorEnvelope {
    ErrorEnvelope::new(
        ErrorCode::parse("ai_archive.import.failed").expect("a legal code"),
        SafeMessage::parse("The archive could not be imported.").expect("a legal message"),
        false,
    )
}

fn warning() -> WarningEnvelope {
    WarningEnvelope {
        code: ErrorCode::parse("ai_archive.import.incomplete").expect("a legal code"),
        message: SafeMessage::parse("The archive was imported, but it is not complete.")
            .expect("a legal message"),
        field_path: None,
        extensions: Extensions::new(),
    }
}

#[test]
fn a_partially_succeeded_report_without_warning_or_error_fails_validation() {
    let error = report(OperationStatus::PartiallySucceeded)
        .validate()
        .expect_err("a partial report must say what is missing");

    assert_eq!(error, OperationContractError::PartialWithoutDiagnostic);
}

#[test]
fn a_failed_report_without_error_fails_validation() {
    let error = report(OperationStatus::Failed)
        .validate()
        .expect_err("a failed report must carry its error");

    assert_eq!(error, OperationContractError::FailedWithoutError);
}

#[test]
fn a_succeeded_report_with_error_fails_validation() {
    let mut succeeded = report(OperationStatus::Succeeded);
    succeeded.error = Some(failure());

    let error = succeeded
        .validate()
        .expect_err("a succeeded report forbids an error");

    assert_eq!(
        error,
        OperationContractError::SucceededWithError { count: 1 }
    );
}

#[test]
fn a_partial_report_with_one_warning_validates() {
    let mut partial = report(OperationStatus::PartiallySucceeded);
    partial.warnings.push(warning());

    partial.validate().expect("one warning is a diagnostic");

    let mut failed = report(OperationStatus::Failed);
    failed.error = Some(failure());
    failed.validate().expect("a failed report with its error");
    report(OperationStatus::Succeeded)
        .validate()
        .expect("a clean success");
}
