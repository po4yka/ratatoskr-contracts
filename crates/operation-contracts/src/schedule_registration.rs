//! The command a domain service sends to Platform to register one of its recurring schedules
//! (XR-021 CONTRACTS.md S08).

use ratatoskr_event_envelope::{CommandPayload, CommandType};
use ratatoskr_identifiers::{Extensions, UserId, wire_string_newtype};

use crate::kind::OperationKind;

wire_string_newtype! {
    /// A stable lowercase label naming a registering service or one of its schedules, such as
    /// `ratatoskr-channel-digests` or `daily-digest`.
    ///
    /// A product identifier, not display text: spaces, uppercase letters and punctuation other
    /// than `_` and `-` are refused so one logical schedule cannot exist under several spellings.
    pub struct ScheduleRegistrationLabel {
        pattern  = r"^[a-z][a-z0-9_-]{0,63}$",
        max_len  = 64,
        examples = ["ratatoskr-channel-digests", "daily-digest"],
    }
}

wire_string_newtype! {
    /// A five-field UTC cron expression, such as `0 6 * * *`.
    ///
    /// This type fixes the shape only: five fields separated by spaces or tabs. Whether every field is
    /// a legal cron term, and when the schedule next fires, is decided by Platform's scheduler,
    /// which refuses an expression it cannot evaluate.
    pub struct ScheduleCronExpression {
        pattern  = r"^[^\x00-\x20\x7f]+([ \t]+[^\x00-\x20\x7f]+){4}$",
        max_len  = 128,
        examples = ["0 6 * * *", "*/15 * * * *"],
    }
}

/// Payload of `platform.schedule.registration_requested.v1`: a domain service asks Platform to
/// create or update one recurring schedule.
///
/// Platform upserts on `(service_name, name)`, so registering the same pair again replaces the
/// earlier registration. The enclosing command envelope's `producer` MUST equal `service_name`,
/// and Platform refuses a registration from a producer it has not allowlisted. When the schedule
/// is due Platform publishes `command_type` with `payload` as its template.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PlatformScheduleRegistrationRequested {
    /// The registering service, such as `ratatoskr-channel-digests`. Equals the envelope
    /// `producer`.
    pub service_name: ScheduleRegistrationLabel,

    /// The schedule's name, unique per service.
    pub name: ScheduleRegistrationLabel,

    /// The Platform user the scheduled operations belong to.
    pub owner_user_id: UserId,

    /// When the schedule fires, as a five-field UTC cron expression.
    pub cron_expression: ScheduleCronExpression,

    /// The command Platform publishes each time the schedule is due.
    pub command_type: CommandType,

    /// The kind of Platform operation minted for each occurrence.
    pub operation_kind: OperationKind,

    /// Template of the published command's payload. A JSON object, empty when the command needs
    /// no template.
    pub payload: serde_json::Map<String, serde_json::Value>,

    /// Whether the schedule fires. A disabled registration keeps the schedule without running it.
    pub enabled: bool,

    /// Unknown-but-preserved additive payload members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl CommandPayload for PlatformScheduleRegistrationRequested {
    const COMMAND_TYPE: &'static str = "platform.schedule.registration_requested.v1";
}
