//! Platform subscription and run command payloads.

use ratatoskr_event_envelope::CommandPayload;
use ratatoskr_identifiers::{Extensions, OperationId, TenantRef};

use crate::{
    ChannelDigestContractError, ChannelDigestIdempotencyKey, ChannelDigestRunId,
    ChannelDigestRunTrigger, ChannelUsername, DigestWindow, OutputLanguage,
    SubscriptionDesiredState,
};

/// Payload of `channel_digest.subscription.set_requested.v1`.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ChannelDigestSubscriptionSetRequested {
    /// Platform operation owning this visible mutation.
    pub operation_id: OperationId,
    /// Authenticated internal user owner.
    pub owner: TenantRef,
    /// Stable logical mutation identity.
    pub idempotency_key: ChannelDigestIdempotencyKey,
    /// Canonical public Telegram channel locator.
    pub channel_username: ChannelUsername,
    /// Exact desired subscription state.
    pub desired_state: SubscriptionDesiredState,
    /// Unknown additive fields preserved by consumers; producers leave this empty.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl CommandPayload for ChannelDigestSubscriptionSetRequested {
    const COMMAND_TYPE: &'static str = "channel_digest.subscription.set_requested.v1";
}

impl ChannelDigestSubscriptionSetRequested {
    /// Refuses untyped state before a producer publishes the command.
    ///
    /// # Errors
    ///
    /// Returns [`ChannelDigestContractError::ProducerExtensionsNotEmpty`] when an authored
    /// extension is present.
    pub fn validate_for_publish(&self) -> Result<(), ChannelDigestContractError> {
        require_empty_extensions(&self.extensions)
    }
}

/// Payload of `channel_digest.run.requested.v1`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct ChannelDigestRunRequested {
    /// Platform operation owning this visible run.
    pub operation_id: OperationId,
    /// Authenticated internal user owner.
    pub owner: TenantRef,
    /// Stable digest-run identity selected before dispatch.
    pub digest_run_id: ChannelDigestRunId,
    /// Stable logical request identity.
    pub idempotency_key: ChannelDigestIdempotencyKey,
    /// Bounded closed-open acquisition window.
    pub window: DigestWindow,
    /// Requested recap output language.
    pub output_language: OutputLanguage,
    /// Authority which selected the run and its end instant.
    pub trigger: ChannelDigestRunTrigger,
    /// Unknown additive fields preserved by consumers; producers leave this empty.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl CommandPayload for ChannelDigestRunRequested {
    const COMMAND_TYPE: &'static str = "channel_digest.run.requested.v1";
}

impl ChannelDigestRunRequested {
    /// Validates the authority/window linkage while preserving additive consumer fields.
    ///
    /// # Errors
    ///
    /// Returns [`ChannelDigestContractError::TriggerWindowMismatch`] when the trigger instant
    /// differs from the exclusive window end.
    pub fn validate(&self) -> Result<(), ChannelDigestContractError> {
        let authority_instant = match &self.trigger {
            ChannelDigestRunTrigger::OnDemand { accepted_at } => *accepted_at,
            ChannelDigestRunTrigger::Scheduled { due_at, .. } => *due_at,
        };
        if authority_instant != self.window.end_at {
            return Err(ChannelDigestContractError::TriggerWindowMismatch);
        }
        Ok(())
    }

    /// Validates semantic linkage and refuses producer-authored extensions.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::validate`] or
    /// [`ChannelDigestContractError::ProducerExtensionsNotEmpty`].
    pub fn validate_for_publish(&self) -> Result<(), ChannelDigestContractError> {
        self.validate()?;
        require_empty_extensions(&self.extensions)
    }
}

#[derive(serde::Deserialize)]
struct ChannelDigestRunRequestedWire {
    operation_id: OperationId,
    owner: TenantRef,
    digest_run_id: ChannelDigestRunId,
    idempotency_key: ChannelDigestIdempotencyKey,
    window: DigestWindow,
    output_language: OutputLanguage,
    trigger: ChannelDigestRunTrigger,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for ChannelDigestRunRequested {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ChannelDigestRunRequestedWire::deserialize(deserializer)?;
        let payload = Self {
            operation_id: wire.operation_id,
            owner: wire.owner,
            digest_run_id: wire.digest_run_id,
            idempotency_key: wire.idempotency_key,
            window: wire.window,
            output_language: wire.output_language,
            trigger: wire.trigger,
            extensions: wire.extensions,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}

fn require_empty_extensions(extensions: &Extensions) -> Result<(), ChannelDigestContractError> {
    if extensions.is_empty() {
        Ok(())
    } else {
        Err(ChannelDigestContractError::ProducerExtensionsNotEmpty)
    }
}
