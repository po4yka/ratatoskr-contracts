//! The point-in-time public view of a long-running operation, and its cross-field invariants.

use ratatoskr_error_contracts::{ErrorEnvelope, WarningEnvelope};
use ratatoskr_identifiers::{EntityRef, Extensions, OperationId, TenantRef, WireTimestamp};

use crate::error::OperationContractError;
use crate::kind::{OperationKind, OperationStage};
use crate::percent::ProgressPercent;
use crate::result_ref::OperationResultRef;
use crate::status::OperationStatus;

/// A point-in-time public view of a long-running operation (`ARCHITECTURE.md` S5.4).
///
/// Named `OperationSnapshot`, not `Operation`, because this repository owns wire representations,
/// not entities (`ARCHITECTURE.md` S15.1, S15.12). Two snapshots of the same `operation_id` at
/// different `status_changed_at` are both valid and are not expected to be equal.
///
/// No `schema_version` field: S5.4's field list does not include one, and the envelope already
/// carries the envelope major. A third version axis is a compatibility hazard, not a feature.
///
/// `Deserialize` is hand-written (the **only** hand-written `Deserialize` in this repository)
/// because the invariants below are cross-field and serde has no validation hook. It parses a
/// private mirror struct and then checks. A field added to the public struct and not the mirror
/// would be silently dropped; test `O-2` (byte round-trip of a fixture carrying every field)
/// fails immediately if that happens, and a source comment points at it.
///
/// # Cross-field invariants
///
/// Every one is re-checkable through [`OperationSnapshot::validate`]:
///
/// - **I1** `terminated_at` is present exactly when `status.is_terminal()`.
/// - **I2** `failed` requires at least one error.
/// - **I3** `succeeded` forbids errors.
/// - **I4** `partially_succeeded` requires at least one warning or error.
/// - **I5** `status_changed_at` and `terminated_at` are never earlier than `accepted_at`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct OperationSnapshot {
    /// Stable identity of the operation, assigned when it was accepted and never reused.
    /// `UUIDv7` when newly minted. Bare UUID on the wire.
    pub operation_id: OperationId,

    /// What work this operation performs.
    pub kind: OperationKind,

    /// Current lifecycle state. Closed enum; see [`OperationStatus`].
    pub status: OperationStatus,

    /// Producer-defined phase inside the current status. Absent when the producer publishes none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<OperationStage>,

    /// Whole-percent completion estimate. Absent means unknown, never zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<ProgressPercent>,

    /// References to what the operation produced. Empty until effects exist. Ordering is
    /// producer-defined and is not contract-significant.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<OperationResultRef>,

    /// Terminal failures. Non-empty exactly when `status` is `failed`; permitted on
    /// `partially_succeeded` and `cancelled`; forbidden on `succeeded`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ErrorEnvelope>,

    /// Non-terminal problems that did not prevent the recorded outcome. Distinct from `errors`
    /// (`ARCHITECTURE.md` S5.5).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<WarningEnvelope>,

    /// Whether resubmitting the identical request may succeed. Meaningful only for a terminal
    /// unsuccessful status; `false` otherwise. Deliberately unconstrained by `status`: no
    /// document states the rule, and a harmless nonsensical value beats an invented one.
    pub retryable: bool,

    /// The unit of user-visible work this operation belongs to, e.g. `operation:018f…`.
    /// Every event and error emitted while serving the operation carries the same value.
    pub correlation_id: EntityRef,

    /// The owner of the data this operation concerns. Absent for system-initiated operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<TenantRef>,

    /// Instant the accepting component durably recorded the request. Platform-observed, not
    /// client-asserted.
    pub accepted_at: WireTimestamp,

    /// Instant `status` last changed. Platform-observed. Equal to `accepted_at` for the first
    /// snapshot; never earlier.
    pub status_changed_at: WireTimestamp,

    /// Instant the operation first reached a terminal status. Present exactly when
    /// `status.is_terminal()`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminated_at: Option<WireTimestamp>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl OperationSnapshot {
    /// Re-checks every cross-field invariant. `Deserialize` calls this; a producer that mutates a
    /// snapshot in place calls it again before emitting.
    ///
    /// # Errors
    ///
    /// [`OperationContractError::TerminationMismatch`] for I1,
    /// [`OperationContractError::FailedWithoutError`] for I2,
    /// [`OperationContractError::SucceededWithError`] for I3,
    /// [`OperationContractError::PartialWithoutDiagnostic`] for I4,
    /// [`OperationContractError::TimestampsOutOfOrder`] for I5.
    pub fn validate(&self) -> Result<(), OperationContractError> {
        // I1
        if self.terminated_at.is_some() != self.status.is_terminal() {
            return Err(OperationContractError::TerminationMismatch {
                status: self.status.as_str(),
                present: self.terminated_at.is_some(),
            });
        }
        // I2
        if self.status == OperationStatus::Failed && self.errors.is_empty() {
            return Err(OperationContractError::FailedWithoutError);
        }
        // I3
        if self.status == OperationStatus::Succeeded && !self.errors.is_empty() {
            return Err(OperationContractError::SucceededWithError {
                count: self.errors.len(),
            });
        }
        // I4
        if self.status == OperationStatus::PartiallySucceeded
            && self.warnings.is_empty()
            && self.errors.is_empty()
        {
            return Err(OperationContractError::PartialWithoutDiagnostic);
        }
        // I5
        if self.status_changed_at < self.accepted_at {
            return Err(OperationContractError::TimestampsOutOfOrder {
                later: "status_changed_at",
                later_value: self.status_changed_at.to_wire(),
                accepted_at: self.accepted_at.to_wire(),
            });
        }
        if let Some(terminated_at) = self.terminated_at
            && terminated_at < self.accepted_at
        {
            return Err(OperationContractError::TimestampsOutOfOrder {
                later: "terminated_at",
                later_value: terminated_at.to_wire(),
                accepted_at: self.accepted_at.to_wire(),
            });
        }
        Ok(())
    }
}

/// The wire mirror of [`OperationSnapshot`], parsed before the cross-field invariants run.
///
/// Serde offers no post-deserialization validation hook, so the public type's `Deserialize` is
/// hand-written and delegates the field-by-field work here.
///
/// **Drift guard:** a field added to [`OperationSnapshot`] and not to this mirror is silently
/// dropped on parse. Test `O-2`
/// (`crates/operation-contracts/tests/snapshot_roundtrip.rs::roundtrips_a_snapshot_carrying_every_field`)
/// fails the moment that happens. Do not add a field to one of these structs alone.
#[derive(Debug, serde::Deserialize)]
struct OperationSnapshotWire {
    operation_id: OperationId,
    kind: OperationKind,
    status: OperationStatus,
    #[serde(default)]
    stage: Option<OperationStage>,
    #[serde(default)]
    progress_percent: Option<ProgressPercent>,
    #[serde(default)]
    results: Vec<OperationResultRef>,
    #[serde(default)]
    errors: Vec<ErrorEnvelope>,
    #[serde(default)]
    warnings: Vec<WarningEnvelope>,
    retryable: bool,
    correlation_id: EntityRef,
    #[serde(default)]
    tenant_id: Option<TenantRef>,
    accepted_at: WireTimestamp,
    status_changed_at: WireTimestamp,
    #[serde(default)]
    terminated_at: Option<WireTimestamp>,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for OperationSnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = OperationSnapshotWire::deserialize(deserializer)?;
        let snapshot = Self {
            operation_id: wire.operation_id,
            kind: wire.kind,
            status: wire.status,
            stage: wire.stage,
            progress_percent: wire.progress_percent,
            results: wire.results,
            errors: wire.errors,
            warnings: wire.warnings,
            retryable: wire.retryable,
            correlation_id: wire.correlation_id,
            tenant_id: wire.tenant_id,
            accepted_at: wire.accepted_at,
            status_changed_at: wire.status_changed_at,
            terminated_at: wire.terminated_at,
            extensions: wire.extensions,
        };
        snapshot.validate().map_err(serde::de::Error::custom)?;
        Ok(snapshot)
    }
}
