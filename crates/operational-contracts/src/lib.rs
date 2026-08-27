//! Sanitized public status and bounded owner operational inspection wire contracts.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod inspection;
mod status;

pub use crate::inspection::{
    AuditAction, AuditEventPage, AuditEventSummary, AuditOutcome, AuditTargetKind,
    InspectionContractError, InspectionCursor, MAX_PAGE_ITEMS, OperationInspectionPage,
    OperationInspectionSummary, ScheduleInspectionPage, ScheduleInspectionSummary, ScheduleName,
    ServiceName,
};
pub use crate::status::{
    PublicComponentId, PublicComponentState, PublicStatusComponent, PublicStatusDocument,
    PublicStatusState, StatusContractError,
};

/// The live grant required for deployment-wide operational inspection.
pub const PLATFORM_OWNER_GRANT: &str = "platform.owner";
/// Capability for deployment-wide operation inspection.
pub const OPERATIONS_INSPECT_CAPABILITY: &str = "platform.operations.inspect";
/// Capability for deployment schedule inspection.
pub const SCHEDULES_INSPECT_CAPABILITY: &str = "platform.schedules.inspect";
/// Capability for deployment audit inspection.
pub const AUDIT_INSPECT_CAPABILITY: &str = "platform.audit.inspect";
