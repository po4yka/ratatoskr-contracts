//! The reports a social-capture owner sends to Platform, built once (XR-021 CONTRACTS.md S10
//! CD1).
//!
//! `ratatoskr-x`, `ratatoskr-instagram` and `ratatoskr-threads` report an explicit capture to
//! Platform as `platform.operation.reported.v1`. Three facts are reported, and every service
//! builds them here so the wording, codes and retryability cannot drift apart:
//!
//! - [`queued_report`]: the capture was accepted and is waiting for resolution.
//! - [`preserved_report`]: the post was resolved and preserved as a social source.
//! - [`unavailable_report`]: the post could not be preserved, and why ([`SourceUnavailability`]).
//!
//! Exactly one terminal report (preserved or unavailable) is sent per operation.
//! [`report_envelope`] wraps any of them in the canonical event envelope.

use ratatoskr_error_contracts::ErrorEnvelope;
use ratatoskr_event_envelope::{EnvelopeSchemaVersion, EventEnvelope, EventPayload, ProducerName};
use ratatoskr_identifiers::{
    CommandId, EntityRef, EventId, Extensions, OperationId, SafeMessage, SocialSourceId, TenantRef,
    UserId, WireTimestamp,
};
use ratatoskr_operation_contracts::{
    OperationReported, OperationResultKind, OperationResultRef, OperationStage, OperationStatus,
};

use crate::{SocialCaptureOutcomeCode, SocialContractError};

/// Result kind of the social source a preserved capture produced.
pub const SOCIAL_POST_RESULT_KIND: &str = "social.post";

/// Stage of a capture that was accepted and awaits resolution.
pub const STAGE_QUEUED: &str = "capture_queued";

/// Stage of a capture that was resolved and preserved.
pub const STAGE_PRESERVED: &str = "capture_preserved";

/// Stage of a capture that could not be preserved.
pub const STAGE_UNAVAILABLE: &str = "capture_unavailable";

/// Why a post could not be preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceUnavailability {
    /// The provider says the post was deleted. Permanent.
    Deleted,
    /// The post is private, protected, unsupported or its permalink cannot be mapped. Permanent.
    Inaccessible,
    /// The provider could not be reached or throttled the request, and every retry is spent.
    /// The user may try again later.
    Transient,
}

impl SourceUnavailability {
    const fn code(self) -> SocialCaptureOutcomeCode {
        match self {
            Self::Deleted => SocialCaptureOutcomeCode::SourceDeleted,
            Self::Inaccessible | Self::Transient => SocialCaptureOutcomeCode::SourceUnavailable,
        }
    }

    const fn message(self) -> &'static str {
        match self {
            Self::Deleted => "The post was deleted by the provider.",
            Self::Inaccessible => "The post cannot be retrieved publicly.",
            Self::Transient => "The post could not be retrieved. Try again later.",
        }
    }

    const fn retryable(self) -> bool {
        matches!(self, Self::Transient)
    }
}

fn construction(reason: impl std::fmt::Display) -> SocialContractError {
    SocialContractError::ReportConstruction(reason.to_string())
}

fn report(
    operation: OperationId,
    status: OperationStatus,
    stage: &str,
) -> Result<OperationReported, SocialContractError> {
    Ok(OperationReported {
        operation_id: operation,
        status,
        stage: Some(OperationStage::parse(stage).map_err(construction)?),
        progress_percent: None,
        results: Vec::new(),
        error: None,
        warnings: Vec::new(),
        extensions: Extensions::new(),
    })
}

/// The report of a capture that was accepted and waits for resolution: status `queued`, stage
/// `capture_queued`, no results and no error.
///
/// # Errors
///
/// [`SocialContractError::ReportConstruction`] when a static member fails its grammar.
pub fn queued_report(operation: OperationId) -> Result<OperationReported, SocialContractError> {
    report(operation, OperationStatus::Queued, STAGE_QUEUED)
}

/// The report of a capture that preserved a post: status `succeeded`, stage `capture_preserved`
/// and one `social.post` result pointing at `social_source:<uuid>`. Carries no error and no
/// warning.
///
/// # Errors
///
/// [`SocialContractError::ReportConstruction`] when a static member fails its grammar.
pub fn preserved_report(
    operation: OperationId,
    source: SocialSourceId,
) -> Result<OperationReported, SocialContractError> {
    let mut preserved = report(operation, OperationStatus::Succeeded, STAGE_PRESERVED)?;
    preserved.results.push(OperationResultRef {
        result_kind: OperationResultKind::parse(SOCIAL_POST_RESULT_KIND).map_err(construction)?,
        target: EntityRef::from(source),
        blob: None,
        ai_archive_import_summary: None,
        extensions: Extensions::new(),
    });
    preserved.validate().map_err(construction)?;
    Ok(preserved)
}

/// The report of a capture that could not preserve its post: status `failed`, stage
/// `capture_unavailable`, no results and exactly one error.
///
/// | reason | code | retryable | message |
/// | --- | --- | --- | --- |
/// | `Deleted` | `social.source.deleted` | no | The post was deleted by the provider. |
/// | `Inaccessible` | `social.source.unavailable` | no | The post cannot be retrieved publicly. |
/// | `Transient` | `social.source.unavailable` | yes | The post could not be retrieved. Try again later. |
///
/// # Errors
///
/// [`SocialContractError::ReportConstruction`] when a static member fails its grammar.
pub fn unavailable_report(
    operation: OperationId,
    reason: SourceUnavailability,
) -> Result<OperationReported, SocialContractError> {
    let mut unavailable = report(operation, OperationStatus::Failed, STAGE_UNAVAILABLE)?;
    unavailable.error = Some(ErrorEnvelope::new(
        reason.code().error_code(),
        SafeMessage::parse(reason.message()).map_err(construction)?,
        reason.retryable(),
    ));
    unavailable.validate().map_err(construction)?;
    Ok(unavailable)
}

/// Wraps `report` in the canonical `platform.operation.reported.v1` event envelope.
///
/// `producer` is the reporting service (`ratatoskr-x`, `ratatoskr-instagram` or
/// `ratatoskr-threads`), `aggregate` names the capture (services pass `capture:<capture uuid>`),
/// and `causation` is the command that started the work. The envelope's correlation is
/// `operation:<operation>`, its tenant is `user:<owner>`, its schema version is the current one
/// and it is stamped with the current instant.
///
/// # Errors
///
/// [`SocialContractError::ReportConstruction`] when `producer` is not a legal producer name, when
/// the report breaks an operation-status invariant, or when it cannot be encoded as an object.
pub fn report_envelope(
    producer: &str,
    owner: UserId,
    operation: OperationId,
    aggregate: EntityRef,
    causation: CommandId,
    event_id: EventId,
    report: &OperationReported,
) -> Result<EventEnvelope, SocialContractError> {
    report.validate().map_err(construction)?;
    let serde_json::Value::Object(payload) = serde_json::to_value(report).map_err(construction)?
    else {
        return Err(construction("the report did not encode as an object"));
    };
    Ok(EventEnvelope {
        event_id,
        event_type: OperationReported::event_type(),
        occurred_at: WireTimestamp::now(),
        producer: ProducerName::parse(producer).map_err(construction)?,
        aggregate_id: aggregate,
        correlation_id: operation.as_entity_ref(),
        causation_id: Some(causation.as_entity_ref()),
        tenant_id: Some(TenantRef::of_user(owner)),
        schema_version: EnvelopeSchemaVersion::CURRENT,
        payload,
        extensions: Extensions::new(),
    })
}
