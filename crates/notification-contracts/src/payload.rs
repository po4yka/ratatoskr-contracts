//! The raised-notification payload: [`NotificationRaised`].

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{EntityRef, Extensions, NotificationId, SafeMessage, TenantRef};

use crate::error::NotificationContractError;
use crate::hints::{NotificationPriority, QuietHoursHint};
use crate::taxonomy::NotificationClass;

/// Payload of `platform.notification.raised.v1`: a producer judged that one of its users should
/// be told something.
///
/// A fact, not an order (`AGENTS.md` principle 9): by the time this payload exists the judgment
/// is complete, and whether a message is actually sent — preference filtering, dedupe, quiet
/// hours, channel choice — is `ratatoskr-telegram`'s decision. The aggregate identifier of the
/// carrying envelope names the raised notification itself as `notification:<uuid>`, because the
/// notification is what consumers acknowledge, suppress and dedupe; its identity is never
/// borrowed from the causing operation or analysis.
///
/// The only cross-field invariant today is the taxonomy registry floor, but `Deserialize` is
/// hand-written anyway, following the crate family's checked-intermediate pattern: a private
/// wire mirror parses field by field, then [`Self::validate`] runs. A field added to the public
/// struct and not to the mirror would be silently dropped; the full-payload fixture round trips
/// through `cargo contracts check`, which fails the moment that happens. Do not add a field to
/// one of these structs alone.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct NotificationRaised {
    /// Identity of this raised notification. Required. Bare canonical lowercase hyphenated UUID;
    /// also the logical key a consumer uses to recognize the same notification re-raised under a
    /// new transport event id.
    pub notification_id: NotificationId,

    /// Registry version of the class taxonomy the producer speaks. Required; greater than zero.
    /// A consumer that knows only registry version 1 reads a higher version as "may carry classes
    /// I will hold as preserved".
    pub class_registry_version: u32,

    /// Which kind of thing happened. Required; see [`NotificationClass`]. Unknown tokens are
    /// preserved verbatim, never rejected.
    pub class: NotificationClass,

    /// The one user this notification concerns. Required; closed tenancy grammar `user:<uuid>`.
    /// When the carrying envelope names a tenant at all, it names the same user.
    pub recipient: TenantRef,

    /// Short carrier-safe headline. Required; see `SafeMessage`.
    pub title: SafeMessage,

    /// Longer carrier-safe detail. Optional and omitted when absent; producers keep it
    /// summary-level — secrets and raw provider content travel elsewhere, never here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<SafeMessage>,

    /// The operation this notification belongs to, when one does. Optional; opaque to this crate:
    /// the open `<kind>:<local_id>` pointer grammar applies and no referent kind is interpreted.
    /// Should equal the carrying envelope's `correlation_id` when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_ref: Option<EntityRef>,

    /// The knowledge analysis this notification belongs to, when one does. Optional; opaque to
    /// this crate in the same way as `operation_ref`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis_ref: Option<EntityRef>,

    /// Producer-asserted urgency judgment. Optional; advisory only, enforced by nobody on this
    /// side of the wire. Absent means the consumer's default ordering applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority_hint: Option<NotificationPriority>,

    /// A daily window during which delivery should be held back. Optional; advisory only, and
    /// interpreted by the consumer against timezone and preference data it owns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quiet_hours: Option<QuietHoursHint>,

    /// Unknown-but-preserved additive fields. A producer constructing a payload leaves this
    /// empty (ADR-0008): everything a consumer should read is a typed field above.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for NotificationRaised {
    const EVENT_TYPE: &'static str = "platform.notification.raised.v1";
}

impl NotificationRaised {
    /// Re-checks every self-consistency invariant. `Deserialize` calls this; a producer that
    /// mutates a payload in place calls it again before emitting.
    ///
    /// # Errors
    ///
    /// [`NotificationContractError::ZeroRegistryVersion`] when `class_registry_version` is zero.
    pub fn validate(&self) -> Result<(), NotificationContractError> {
        if self.class_registry_version == 0 {
            return Err(NotificationContractError::ZeroRegistryVersion);
        }
        Ok(())
    }
}

/// The wire mirror of [`NotificationRaised`], parsed before the cross-field invariants run.
///
/// Serde offers no post-deserialization validation hook, so the public type's `Deserialize` is
/// hand-written and delegates the field-by-field work here.
///
/// **Drift guard:** a field added to [`NotificationRaised`] and not to this mirror is silently
/// dropped on parse; the full-payload fixture round trip fails the moment that happens. Do not
/// add a field to one of these structs alone.
#[derive(Debug, serde::Deserialize)]
struct NotificationRaisedWire {
    notification_id: NotificationId,
    class_registry_version: u32,
    class: NotificationClass,
    recipient: TenantRef,
    title: SafeMessage,
    #[serde(default)]
    message: Option<SafeMessage>,
    #[serde(default)]
    operation_ref: Option<EntityRef>,
    #[serde(default)]
    analysis_ref: Option<EntityRef>,
    #[serde(default)]
    priority_hint: Option<NotificationPriority>,
    #[serde(default)]
    quiet_hours: Option<QuietHoursHint>,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for NotificationRaised {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = NotificationRaisedWire::deserialize(deserializer)?;
        let payload = Self {
            notification_id: wire.notification_id,
            class_registry_version: wire.class_registry_version,
            class: wire.class,
            recipient: wire.recipient,
            title: wire.title,
            message: wire.message,
            operation_ref: wire.operation_ref,
            analysis_ref: wire.analysis_ref,
            priority_hint: wire.priority_hint,
            quiet_hours: wire.quiet_hours,
            extensions: wire.extensions,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}
