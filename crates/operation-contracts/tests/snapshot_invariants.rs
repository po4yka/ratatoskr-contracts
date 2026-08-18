//! `OperationSnapshot` cross-field invariants I1–I5 and the bounded percent — test O-3.
//!
//! Every invariant is asserted twice: once through [`OperationSnapshot::validate`], which a
//! producer calls after mutating a snapshot, and once through `Deserialize`, which is the layer
//! the matching `invalid/` fixture declares as `rejected_by = ["serde"]`.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_error_contracts::{ErrorCode, ErrorEnvelope, WarningEnvelope};
use ratatoskr_identifiers::{
    EntityRef, Extensions, OperationId, SafeMessage, WireTimestamp, canonical_json,
};
use ratatoskr_operation_contracts::{
    OperationContractError, OperationKind, OperationSnapshot, OperationStatus, ProgressPercent,
};

const ACCEPTED_AT: &str = "2026-08-17T10:00:00Z";
const LATER: &str = "2026-08-17T10:05:00Z";
const EARLIER: &str = "2026-08-17T09:59:00Z";

fn instant(raw: &str) -> WireTimestamp {
    WireTimestamp::parse(raw).expect("a canonical instant")
}

/// A snapshot that satisfies every invariant, for a test to break one member at a time.
fn accepted() -> OperationSnapshot {
    OperationSnapshot {
        operation_id: OperationId::parse("018f0000-0000-7000-8000-000000000010")
            .expect("a canonical UUID"),
        kind: OperationKind::parse("content.document.extract").expect("a legal kind"),
        status: OperationStatus::Accepted,
        stage: None,
        progress_percent: None,
        results: Vec::new(),
        errors: Vec::new(),
        warnings: Vec::new(),
        retryable: false,
        correlation_id: EntityRef::parse("operation:018f0000-0000-7000-8000-000000000010")
            .expect("a legal reference"),
        tenant_id: None,
        accepted_at: instant(ACCEPTED_AT),
        status_changed_at: instant(ACCEPTED_AT),
        terminated_at: None,
        extensions: Extensions::new(),
    }
}

fn failure(code: &str, message: &str) -> ErrorEnvelope {
    ErrorEnvelope::new(
        ErrorCode::parse(code).expect("a legal code"),
        SafeMessage::parse(message).expect("a legal message"),
        false,
    )
}

fn warning(code: &str, message: &str) -> WarningEnvelope {
    WarningEnvelope {
        code: ErrorCode::parse(code).expect("a legal code"),
        message: SafeMessage::parse(message).expect("a legal message"),
        field_path: None,
        extensions: Extensions::new(),
    }
}

/// Serializing never validates — only `Deserialize` does — so this renders an illegal snapshot
/// and asserts the parse layer refuses it with the documented text.
fn rejected_by_serde(snapshot: &OperationSnapshot, expected_text: &str) {
    let rendered = canonical_json(snapshot).expect("serialization never validates");
    let error = serde_json::from_str::<OperationSnapshot>(&rendered)
        .expect_err("the parse layer enforces the cross-field invariants");
    assert!(
        error.to_string().contains(expected_text),
        "{error} must contain {expected_text:?}"
    );
}

/// The baseline is legal, so every failure below is caused by the one member the test changed.
#[test]
fn the_baseline_snapshot_is_valid() {
    let snapshot = accepted();
    assert_eq!(snapshot.validate(), Ok(()));
    let rendered = canonical_json(&snapshot).expect("canonical rendering");
    assert_eq!(
        serde_json::from_str::<OperationSnapshot>(&rendered).expect("round trip"),
        snapshot
    );
}

/// O-3. `ProgressPercent` bounds hold through the constructor and through deserialization.
#[test]
fn progress_percent_rejects_over_100() {
    assert_eq!(ProgressPercent::new(0).expect("zero is legal").get(), 0);
    assert_eq!(
        ProgressPercent::new(ProgressPercent::MAX)
            .expect("the bound is inclusive")
            .get(),
        100
    );
    assert_eq!(
        ProgressPercent::new(101),
        Err(OperationContractError::ProgressPercentOutOfRange { got: 101 })
    );

    assert!(serde_json::from_str::<ProgressPercent>("101").is_err());
    // A fractional estimate is not representable at all: the wire type is an integer.
    assert!(serde_json::from_str::<ProgressPercent>("12.5").is_err());
    assert!(serde_json::from_str::<ProgressPercent>("-1").is_err());
    assert_eq!(
        serde_json::to_string(&ProgressPercent::new(42).expect("a legal estimate")).unwrap(),
        "42"
    );
}

/// I1. A terminal status without a termination instant, and a live status with one, are both
/// facts that cannot be true.
#[test]
fn terminated_at_is_present_exactly_when_status_is_terminal() {
    let mut terminal_without_instant = accepted();
    terminal_without_instant.status = OperationStatus::Cancelled;
    terminal_without_instant.status_changed_at = instant(LATER);
    assert_eq!(
        terminal_without_instant.validate(),
        Err(OperationContractError::TerminationMismatch {
            status: "cancelled",
            present: false,
        })
    );
    rejected_by_serde(
        &terminal_without_instant,
        "terminated_at must be present exactly when status is terminal",
    );

    let mut live_with_instant = accepted();
    live_with_instant.status = OperationStatus::Running;
    live_with_instant.terminated_at = Some(instant(LATER));
    assert_eq!(
        live_with_instant.validate(),
        Err(OperationContractError::TerminationMismatch {
            status: "running",
            present: true,
        })
    );
    rejected_by_serde(
        &live_with_instant,
        "terminated_at must be present exactly when status is terminal",
    );

    let mut terminal = accepted();
    terminal.status = OperationStatus::Cancelled;
    terminal.status_changed_at = instant(LATER);
    terminal.terminated_at = Some(instant(LATER));
    assert_eq!(terminal.validate(), Ok(()));
}

/// I2. `failed` with no structured error tells a consumer nothing it can act on.
#[test]
fn failed_requires_at_least_one_error() {
    let mut snapshot = accepted();
    snapshot.status = OperationStatus::Failed;
    snapshot.status_changed_at = instant(LATER);
    snapshot.terminated_at = Some(instant(LATER));
    assert_eq!(
        snapshot.validate(),
        Err(OperationContractError::FailedWithoutError)
    );
    rejected_by_serde(&snapshot, "status `failed` requires at least one error");

    snapshot
        .errors
        .push(failure("content.extraction.unreadable", "Nothing to read."));
    assert_eq!(snapshot.validate(), Ok(()));
}

/// I3. A terminal error inside a successful outcome is a warning wearing the wrong type
/// (`ARCHITECTURE.md` S5.5).
#[test]
fn succeeded_forbids_errors() {
    let mut snapshot = accepted();
    snapshot.status = OperationStatus::Succeeded;
    snapshot.status_changed_at = instant(LATER);
    snapshot.terminated_at = Some(instant(LATER));
    snapshot
        .errors
        .push(failure("content.extraction.unreadable", "Nothing to read."));
    assert_eq!(
        snapshot.validate(),
        Err(OperationContractError::SucceededWithError { count: 1 })
    );
    rejected_by_serde(&snapshot, "status `succeeded` forbids errors");

    snapshot.errors.clear();
    snapshot
        .warnings
        .push(warning("content.extraction.truncated", "One page was cut."));
    assert_eq!(snapshot.validate(), Ok(()));
}

/// I4. Partial success with no diagnostic does not say which effects were not produced.
#[test]
fn partially_succeeded_requires_a_diagnostic() {
    let mut snapshot = accepted();
    snapshot.status = OperationStatus::PartiallySucceeded;
    snapshot.status_changed_at = instant(LATER);
    snapshot.terminated_at = Some(instant(LATER));
    assert_eq!(
        snapshot.validate(),
        Err(OperationContractError::PartialWithoutDiagnostic)
    );
    rejected_by_serde(
        &snapshot,
        "status `partially_succeeded` requires at least one warning or error",
    );

    let mut with_warning = snapshot.clone();
    with_warning
        .warnings
        .push(warning("content.extraction.truncated", "One page was cut."));
    assert_eq!(with_warning.validate(), Ok(()));

    let mut with_error = snapshot;
    with_error
        .errors
        .push(failure("content.extraction.unreadable", "Page 3 failed."));
    assert_eq!(with_error.validate(), Ok(()));
}

/// I5. A snapshot that changed before it existed is not a fact.
#[test]
fn timestamps_never_precede_accepted_at() {
    let mut changed_early = accepted();
    changed_early.status_changed_at = instant(EARLIER);
    assert_eq!(
        changed_early.validate(),
        Err(OperationContractError::TimestampsOutOfOrder {
            later: "status_changed_at",
            later_value: EARLIER.to_owned(),
            accepted_at: ACCEPTED_AT.to_owned(),
        })
    );
    rejected_by_serde(&changed_early, "is earlier than accepted_at");

    let mut terminated_early = accepted();
    terminated_early.status = OperationStatus::Cancelled;
    terminated_early.terminated_at = Some(instant(EARLIER));
    assert_eq!(
        terminated_early.validate(),
        Err(OperationContractError::TimestampsOutOfOrder {
            later: "terminated_at",
            later_value: EARLIER.to_owned(),
            accepted_at: ACCEPTED_AT.to_owned(),
        })
    );
    rejected_by_serde(&terminated_early, "is earlier than accepted_at");

    // Equality is legal: the first snapshot of an operation changed status when it was accepted.
    let baseline = accepted();
    assert_eq!(baseline.accepted_at, baseline.status_changed_at);
    assert_eq!(baseline.validate(), Ok(()));
}
