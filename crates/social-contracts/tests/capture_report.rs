//! The social capture report builders and their golden fixtures (XR-021 CONTRACTS.md S10 CD1).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_event_envelope::EnvelopeSchemaVersion;
use ratatoskr_identifiers::{
    CommandId, EntityRef, EventId, OperationId, SocialSourceId, UserId, canonical_json,
};
use ratatoskr_operation_contracts::{OperationReported, OperationStage, OperationStatus};
use ratatoskr_social_contracts::{
    SOCIAL_POST_RESULT_KIND, STAGE_PRESERVED, STAGE_QUEUED, STAGE_UNAVAILABLE,
    SocialCaptureOutcomeCode, SocialContractError, SourceUnavailability, preserved_report,
    queued_report, report_envelope, unavailable_report,
};

const OPERATION: &str = "018f0000-0000-7000-8000-000000000a21";
const SOURCE: &str = "018f0000-0000-7000-8000-000000000a31";
const OWNER: &str = "018f0000-0000-7000-8000-000000000005";
const CAPTURE: &str = "capture:018f0000-0000-7000-8000-000000000a41";
const COMMAND: &str = "018f0000-0000-7000-8000-000000000b01";
const EVENT: &str = "018f0000-0000-7000-8000-000000000c01";

fn operation() -> OperationId {
    OperationId::parse(OPERATION).expect("a canonical UUID")
}

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../fixtures/events/platform.operation.reported.v1/valid/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn assert_golden(report: &OperationReported, name: &str) {
    assert_eq!(
        canonical_json(report).expect("the report serializes"),
        fixture(name),
        "{name} is the byte-exact rendering of the builder's report"
    );
    report
        .validate()
        .expect("a built report satisfies its invariants");
}

#[test]
fn builders_reproduce_the_four_golden_fixtures_byte_for_byte() {
    assert_golden(
        &queued_report(operation()).expect("builds"),
        "social-capture-queued.json",
    );
    assert_golden(
        &preserved_report(
            operation(),
            SocialSourceId::parse(SOURCE).expect("a canonical UUID"),
        )
        .expect("builds"),
        "social-capture-preserved.json",
    );
    assert_golden(
        &unavailable_report(operation(), SourceUnavailability::Deleted).expect("builds"),
        "social-capture-deleted.json",
    );
    assert_golden(
        &unavailable_report(operation(), SourceUnavailability::Transient).expect("builds"),
        "social-capture-unavailable-retryable.json",
    );
}

#[test]
fn stages_statuses_codes_and_retryability_follow_the_taxonomy() {
    assert_eq!(SOCIAL_POST_RESULT_KIND, "social.post");
    assert_eq!(STAGE_QUEUED, "capture_queued");
    assert_eq!(STAGE_PRESERVED, "capture_preserved");
    assert_eq!(STAGE_UNAVAILABLE, "capture_unavailable");

    let queued = queued_report(operation()).expect("builds");
    assert_eq!(queued.status, OperationStatus::Queued);
    assert_eq!(
        queued.stage.as_ref().map(OperationStage::as_str),
        Some(STAGE_QUEUED)
    );
    assert!(queued.results.is_empty() && queued.error.is_none() && queued.warnings.is_empty());

    let source = SocialSourceId::parse(SOURCE).expect("a canonical UUID");
    let preserved = preserved_report(operation(), source).expect("builds");
    assert_eq!(preserved.status, OperationStatus::Succeeded);
    assert_eq!(preserved.results.len(), 1);
    let result = preserved.results.first().expect("one result");
    assert_eq!(result.result_kind.as_str(), SOCIAL_POST_RESULT_KIND);
    let target = EntityRef::parse(&result.target.to_wire()).expect("the target is a reference");
    assert_eq!(target.kind().as_str(), "social_source");
    assert_eq!(target.to_wire(), format!("social_source:{SOURCE}"));
    assert!(preserved.error.is_none() && preserved.warnings.is_empty());

    for (reason, code, retryable) in [
        (
            SourceUnavailability::Deleted,
            SocialCaptureOutcomeCode::SourceDeleted,
            false,
        ),
        (
            SourceUnavailability::Inaccessible,
            SocialCaptureOutcomeCode::SourceUnavailable,
            false,
        ),
        (
            SourceUnavailability::Transient,
            SocialCaptureOutcomeCode::SourceUnavailable,
            true,
        ),
    ] {
        let report = unavailable_report(operation(), reason).expect("builds");
        assert_eq!(report.status, OperationStatus::Failed);
        assert_eq!(
            report.stage.as_ref().map(OperationStage::as_str),
            Some(STAGE_UNAVAILABLE)
        );
        assert!(report.results.is_empty() && report.warnings.is_empty());
        let error = report.error.as_ref().expect("exactly one error");
        let parsed = SocialCaptureOutcomeCode::parse(error.code.as_str())
            .expect("the code is in the closed social taxonomy");
        assert_eq!(parsed, code, "{reason:?}");
        assert_eq!(error.retryable, retryable, "{reason:?}");
    }
    let inaccessible =
        unavailable_report(operation(), SourceUnavailability::Inaccessible).expect("builds");
    assert_eq!(
        inaccessible
            .error
            .as_ref()
            .map(|error| error.message.as_str()),
        Some("The post cannot be retrieved publicly.")
    );
}

#[test]
fn report_envelope_carries_the_documented_linkage() {
    let report = unavailable_report(operation(), SourceUnavailability::Deleted).expect("builds");
    let owner = UserId::parse(OWNER).expect("a canonical UUID");
    let aggregate = EntityRef::parse(CAPTURE).expect("a legal reference");
    let causation = CommandId::parse(COMMAND).expect("a canonical UUID");
    let event_id = EventId::parse(EVENT).expect("a canonical UUID");

    let envelope = report_envelope(
        "ratatoskr-x",
        owner,
        operation(),
        aggregate.clone(),
        causation,
        event_id,
        &report,
    )
    .expect("builds");
    assert_eq!(envelope.event_id, event_id);
    assert_eq!(
        envelope.event_type.to_wire(),
        "platform.operation.reported.v1"
    );
    assert_eq!(envelope.producer.as_str(), "ratatoskr-x");
    assert_eq!(envelope.aggregate_id, aggregate);
    assert_eq!(
        envelope.correlation_id.to_wire(),
        format!("operation:{OPERATION}")
    );
    assert_eq!(
        envelope.causation_id.as_ref().map(EntityRef::to_wire),
        Some(format!("command:{COMMAND}"))
    );
    assert_eq!(
        envelope.tenant_id.map(|tenant| tenant.to_string()),
        Some(format!("user:{OWNER}"))
    );
    assert_eq!(envelope.schema_version, EnvelopeSchemaVersion::CURRENT);
    assert!(envelope.extensions.is_empty());
    assert_eq!(
        envelope
            .payload_as::<OperationReported>()
            .expect("typed read"),
        report
    );
    let wire = envelope.to_canonical_json().expect("encodes");
    assert_eq!(
        ratatoskr_event_envelope::EventEnvelope::from_json(wire.as_bytes()).expect("round trips"),
        envelope
    );

    // A producer name outside the grammar is refused, and so is a report that breaks the status
    // invariants (here a failure with no error).
    assert!(matches!(
        report_envelope(
            "Not A Producer",
            owner,
            operation(),
            aggregate.clone(),
            causation,
            event_id,
            &report
        ),
        Err(SocialContractError::ReportConstruction(_))
    ));
    let mut broken = report;
    broken.error = None;
    assert!(matches!(
        report_envelope(
            "ratatoskr-x",
            owner,
            operation(),
            aggregate,
            causation,
            event_id,
            &broken
        ),
        Err(SocialContractError::ReportConstruction(_))
    ));
}
