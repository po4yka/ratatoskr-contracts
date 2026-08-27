//! Bounded owner operational inspection response types.

use ratatoskr_error_contracts::ErrorCode;
use ratatoskr_identifiers::{EntityRef, OperationId, UserId, WireTimestamp, wire_string_newtype};
use ratatoskr_operation_contracts::{OperationKind, OperationStatus};
use uuid::Uuid;

/// Maximum number of rows in any operational inspection page.
pub const MAX_PAGE_ITEMS: usize = 100;

wire_string_newtype! {
    /// Opaque service-generated pagination cursor.
    pub struct InspectionCursor {
        pattern  = r"^[A-Za-z0-9_-]{1,512}$",
        max_len  = 512,
        examples = ["eyJvYnNlcnZlZF9hdCI6IjIwMjYtMDgtMjdUMDk6MDA6MDBaIn0"],
    }
}

wire_string_newtype! {
    /// Stable product-service label, never a hostname or address.
    pub struct ServiceName {
        pattern  = r"^[a-z][a-z0-9_-]{0,63}$",
        max_len  = 64,
        examples = ["telegram"],
    }
}

wire_string_newtype! {
    /// Stable schedule name within one service.
    pub struct ScheduleName {
        pattern  = r"^[a-z][a-z0-9_-]{0,119}$",
        max_len  = 120,
        examples = ["daily_sync"],
    }
}

wire_string_newtype! {
    /// Stable audited action token.
    pub struct AuditAction {
        pattern  = r"^[a-z][a-z0-9_]{0,29}(\.[a-z][a-z0-9_]{0,29}){1,3}$",
        max_len  = 120,
        examples = ["operation.read"],
    }
}

wire_string_newtype! {
    /// Stable kind of audited target.
    pub struct AuditTargetKind {
        pattern  = r"^[a-z][a-z0-9_-]{0,59}$",
        max_len  = 60,
        examples = ["operation"],
    }
}

/// One redacted operation in an owner inspection page.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct OperationInspectionSummary {
    /// Stable operation identity.
    pub operation_id: OperationId,
    /// User whose data the operation concerns.
    pub owner_user_id: UserId,
    /// Contracted kind of work.
    pub kind: OperationKind,
    /// Exact lifecycle state.
    pub status: OperationStatus,
    /// Platform-observed acceptance instant.
    pub accepted_at: WireTimestamp,
    /// Platform-observed instant at which status last changed.
    pub status_changed_at: WireTimestamp,
    /// Stable user-safe failure code, absent when no failure is recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<ErrorCode>,
}

/// Cursor page of deployment-wide operation summaries.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationInspectionPage {
    /// At most 100 summaries in newest-accepted-first order.
    #[schemars(length(max = 100))]
    pub items: Vec<OperationInspectionSummary>,
    /// Opaque continuation cursor, absent on the final page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<InspectionCursor>,
}

/// One schedule's current Platform-owned status projection.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ScheduleInspectionSummary {
    /// Stable schedule identity.
    pub schedule_id: Uuid,
    /// Stable service label, not an address.
    pub service_name: ServiceName,
    /// Schedule name within that service.
    pub name: ScheduleName,
    /// User whose data scheduled work concerns.
    pub owner_user_id: UserId,
    /// Next Platform-observed due instant.
    pub next_due_at: WireTimestamp,
    /// Whether future occurrences are enabled.
    pub enabled: bool,
    /// Last operation outcome, absent before the first occurrence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_outcome: Option<OperationStatus>,
}

/// Cursor page of deterministic schedule status summaries.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScheduleInspectionPage {
    /// At most 100 schedule status rows.
    #[schemars(length(max = 100))]
    pub items: Vec<ScheduleInspectionSummary>,
    /// Opaque continuation cursor, absent on the final page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<InspectionCursor>,
}

/// Stable outcome of one audited action.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// The action proceeded.
    Allowed,
    /// Authorization refused the action.
    Denied,
    /// The permitted action did not complete.
    Failed,
}

/// One redacted audit event.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AuditEventSummary {
    /// Stable audit record identity.
    pub audit_event_id: Uuid,
    /// Platform-observed occurrence instant.
    pub occurred_at: WireTimestamp,
    /// Acting user, absent for a system or unauthenticated event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_user_id: Option<UserId>,
    /// Acting session, absent when no session acted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_session_id: Option<Uuid>,
    /// Stable bounded audited action token.
    pub action: AuditAction,
    /// Stable bounded target kind.
    pub target_kind: AuditTargetKind,
    /// Target UUID when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<Uuid>,
    /// Stable audited outcome.
    pub outcome: AuditOutcome,
    /// Namespaced correlation reference visible to support workflows.
    pub correlation_id: EntityRef,
}

/// Cursor page of newest-first audit summaries.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuditEventPage {
    /// At most 100 audit summaries.
    #[schemars(length(max = 100))]
    pub items: Vec<AuditEventSummary>,
    /// Opaque continuation cursor, absent on the final page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<InspectionCursor>,
}

macro_rules! impl_bounded_page {
    ($page:ident, $row:ty) => {
        impl $page {
            /// Creates a page after enforcing the collection bound.
            ///
            /// # Errors
            ///
            /// Returns [`InspectionContractError::PageTooLarge`] above 100 rows.
            pub fn new(
                items: Vec<$row>,
                next_cursor: Option<InspectionCursor>,
            ) -> Result<Self, InspectionContractError> {
                if items.len() > MAX_PAGE_ITEMS {
                    return Err(InspectionContractError::PageTooLarge {
                        got: items.len(),
                        max: MAX_PAGE_ITEMS,
                    });
                }
                Ok(Self { items, next_cursor })
            }
        }

        impl<'de> serde::Deserialize<'de> for $page {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(serde::Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Wire {
                    items: Vec<$row>,
                    #[serde(default)]
                    next_cursor: Option<InspectionCursor>,
                }

                let wire = Wire::deserialize(deserializer)?;
                Self::new(wire.items, wire.next_cursor).map_err(serde::de::Error::custom)
            }
        }
    };
}

impl_bounded_page!(OperationInspectionPage, OperationInspectionSummary);
impl_bounded_page!(ScheduleInspectionPage, ScheduleInspectionSummary);
impl_bounded_page!(AuditEventPage, AuditEventSummary);

/// Invalid operational inspection response invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InspectionContractError {
    /// A response attempted to exceed the public page bound.
    #[error("inspection page contains {got} rows; maximum is {max}")]
    PageTooLarge {
        /// Observed row count.
        got: usize,
        /// Contract maximum.
        max: usize,
    },
}
