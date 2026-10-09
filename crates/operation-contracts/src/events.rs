//! The operation lifecycle event payload.

use ratatoskr_error_contracts::{ErrorEnvelope, WarningEnvelope};
use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{Extensions, OperationId};

use crate::error::OperationContractError;
use crate::kind::OperationStage;
use crate::percent::ProgressPercent;
use crate::result_ref::OperationResultRef;
use crate::snapshot::OperationSnapshot;
use crate::status::OperationStatus;

/// Payload of `platform.operation.reported.v1`.
///
/// A service reports only the operation progress facts it produced. Platform combines the report
/// with the request facts it owns before clients observe an [`OperationSnapshot`].
///
/// A producer MUST NOT emit a report that breaks the status invariants, which mirror
/// [`OperationSnapshot`] invariants I2 to I4: `failed` requires `error`, `succeeded` forbids
/// `error`, and `partially_succeeded` requires at least one entry in `warnings` or an `error`.
/// [`OperationReported::validate`] checks them and Platform rejects a report that fails.
///
/// Snapshot-only request facts are absent on purpose: `kind` is chosen by the component that
/// accepted the request, `accepted_at` is that component's clock, and `correlation_id` and
/// `tenant_id` belong to the accepted request. Requiring them here would make every producer copy
/// Platform's operation record.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct OperationReported {
    /// Operation whose work reached `status`.
    pub operation_id: OperationId,

    /// Lifecycle status reached by the producer.
    pub status: OperationStatus,

    /// Producer-defined display phase inside the status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<OperationStage>,

    /// Whole-percent completion estimate. Absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<ProgressPercent>,

    /// References to effects the producer completed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<OperationResultRef>,

    /// Terminal failure, when one prevented a usable result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorEnvelope>,

    /// Non-terminal problems that did not prevent the reported outcome.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<WarningEnvelope>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl OperationReported {
    /// Re-checks the status invariants a producer owes before it emits a report.
    ///
    /// # Errors
    ///
    /// [`OperationContractError::FailedWithoutError`] for I2,
    /// [`OperationContractError::SucceededWithError`] for I3,
    /// [`OperationContractError::PartialWithoutDiagnostic`] for I4.
    pub fn validate(&self) -> Result<(), OperationContractError> {
        // I2
        if self.status == OperationStatus::Failed && self.error.is_none() {
            return Err(OperationContractError::FailedWithoutError);
        }
        // I3
        if self.status == OperationStatus::Succeeded && self.error.is_some() {
            return Err(OperationContractError::SucceededWithError { count: 1 });
        }
        // I4
        if self.status == OperationStatus::PartiallySucceeded
            && self.warnings.is_empty()
            && self.error.is_none()
        {
            return Err(OperationContractError::PartialWithoutDiagnostic);
        }
        Ok(())
    }
}

impl EventPayload for OperationReported {
    const EVENT_TYPE: &'static str = "platform.operation.reported.v1";
}

/// Payload of `platform.operation.progressed.v1` (`README.md`, `ARCHITECTURE.md` S9.1).
///
/// A fact: the operation **has** reached `operation.status`. It is not a request to change it
/// (`AGENTS.md` principle 9). State-carried transfer — a consumer needs no prior event to
/// interpret this one, which is what makes at-least-once redelivery idempotent.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct OperationProgressed {
    /// Full public snapshot of the operation at the instant the event was produced.
    pub operation: OperationSnapshot,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for OperationProgressed {
    const EVENT_TYPE: &'static str = "platform.operation.progressed.v1";
}
