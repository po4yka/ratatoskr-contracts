//! Knowledge recap request and terminal fact payloads.

use ratatoskr_event_envelope::{CommandPayload, EventPayload};
use ratatoskr_identifiers::{ContentDigest, Extensions, OperationId, TenantRef, WireTimestamp};

use crate::{
    ChannelDigestContractError, ChannelDigestManifestRef, ChannelDigestResultId,
    ChannelDigestResultRef, ChannelDigestRunId, DigestChannelCount, DigestSourceCount,
    DigestWindow, KnowledgeAnalysisRef, OutputLanguage,
};

/// Fixed analysis family accepted by the first-version recap pipeline.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum ChannelDigestAnalysisFamily {
    /// Public-channel digest recap.
    #[serde(rename = "channel_digest_recap")]
    ChannelDigestRecap,
}

/// Fixed result contract accepted by the first-version recap pipeline.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum ChannelDigestAnalysisContract {
    /// First and only development-version recap contract.
    #[serde(rename = "channel_digest_recap.v1")]
    ChannelDigestRecapV1,
}

/// Payload of `knowledge.channel_digest_recap.requested.v1`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct KnowledgeChannelDigestRecapRequested {
    /// Platform operation owning this recap.
    pub operation_id: OperationId,
    /// Authenticated internal user owner.
    pub owner: TenantRef,
    /// Digest run whose immutable source manifest is analyzed.
    pub digest_run_id: ChannelDigestRunId,
    /// Closed-open source publication window.
    pub window: DigestWindow,
    /// Requested recap language.
    pub output_language: OutputLanguage,
    /// Number of immutable source revisions in the manifest.
    pub source_count: DigestSourceCount,
    /// Number of public channels represented by those sources.
    pub channel_count: DigestChannelCount,
    /// Owner-authorized source interface reference.
    pub manifest_ref: ChannelDigestManifestRef,
    /// SHA-256 identity of the canonical manifest bytes.
    pub manifest_digest: ContentDigest,
    /// Fixed analysis family.
    pub analysis_family: ChannelDigestAnalysisFamily,
    /// Fixed structured-result contract.
    pub analysis_contract: ChannelDigestAnalysisContract,
    /// Unknown additive fields preserved by consumers; producers leave this empty.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl CommandPayload for KnowledgeChannelDigestRecapRequested {
    const COMMAND_TYPE: &'static str = "knowledge.channel_digest_recap.requested.v1";
}

impl KnowledgeChannelDigestRecapRequested {
    /// Validates request count linkage while preserving additive consumer fields.
    ///
    /// # Errors
    ///
    /// Returns [`ChannelDigestContractError::ChannelCountExceedsSourceCount`] when more channels
    /// than sources are declared.
    pub fn validate(&self) -> Result<(), ChannelDigestContractError> {
        if self.channel_count.get() > self.source_count.get() {
            return Err(ChannelDigestContractError::ChannelCountExceedsSourceCount);
        }
        Ok(())
    }

    /// Validates the request and refuses producer-authored extensions.
    ///
    /// # Errors
    ///
    /// Returns any semantic error or
    /// [`ChannelDigestContractError::ProducerExtensionsNotEmpty`].
    pub fn validate_for_publish(&self) -> Result<(), ChannelDigestContractError> {
        self.validate()?;
        require_empty_extensions(&self.extensions)
    }
}

#[derive(serde::Deserialize)]
struct KnowledgeChannelDigestRecapRequestedWire {
    operation_id: OperationId,
    owner: TenantRef,
    digest_run_id: ChannelDigestRunId,
    window: DigestWindow,
    output_language: OutputLanguage,
    source_count: DigestSourceCount,
    channel_count: DigestChannelCount,
    manifest_ref: ChannelDigestManifestRef,
    manifest_digest: ContentDigest,
    analysis_family: ChannelDigestAnalysisFamily,
    analysis_contract: ChannelDigestAnalysisContract,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for KnowledgeChannelDigestRecapRequested {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = KnowledgeChannelDigestRecapRequestedWire::deserialize(deserializer)?;
        let payload = Self {
            operation_id: wire.operation_id,
            owner: wire.owner,
            digest_run_id: wire.digest_run_id,
            window: wire.window,
            output_language: wire.output_language,
            source_count: wire.source_count,
            channel_count: wire.channel_count,
            manifest_ref: wire.manifest_ref,
            manifest_digest: wire.manifest_digest,
            analysis_family: wire.analysis_family,
            analysis_contract: wire.analysis_contract,
            extensions: wire.extensions,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}

/// Exact source coverage reported by one successful recap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct ChannelDigestRecapCoverage {
    /// Verified source revisions selected from the manifest.
    pub selected_count: DigestSourceCount,
    /// Complete source revisions included in provider context.
    pub included_count: u16,
    /// Complete source revisions omitted under the context budget.
    pub omitted_count: u16,
    /// Public channels represented by the included sources.
    pub channel_count: DigestChannelCount,
}

impl ChannelDigestRecapCoverage {
    /// Validates exact arithmetic and the non-empty completion rule.
    ///
    /// # Errors
    ///
    /// Returns [`ChannelDigestContractError::EmptyIncludedCoverage`] for zero included sources or
    /// [`ChannelDigestContractError::InconsistentCoverage`] when counts do not add up.
    pub fn validate(&self) -> Result<(), ChannelDigestContractError> {
        if self.included_count == 0 {
            return Err(ChannelDigestContractError::EmptyIncludedCoverage);
        }
        if self.included_count.checked_add(self.omitted_count) != Some(self.selected_count.get()) {
            return Err(ChannelDigestContractError::InconsistentCoverage);
        }
        if self.channel_count.get() > self.included_count {
            return Err(ChannelDigestContractError::InconsistentCoverage);
        }
        Ok(())
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::struct_field_names,
    reason = "the private wire type mirrors the published count field names"
)]
struct ChannelDigestRecapCoverageWire {
    selected_count: DigestSourceCount,
    included_count: u16,
    omitted_count: u16,
    channel_count: DigestChannelCount,
}

impl<'de> serde::Deserialize<'de> for ChannelDigestRecapCoverage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ChannelDigestRecapCoverageWire::deserialize(deserializer)?;
        let coverage = Self {
            selected_count: wire.selected_count,
            included_count: wire.included_count,
            omitted_count: wire.omitted_count,
            channel_count: wire.channel_count,
        };
        coverage.validate().map_err(serde::de::Error::custom)?;
        Ok(coverage)
    }
}

/// Payload of `knowledge.channel_digest_recap.completed.v1`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct KnowledgeChannelDigestRecapCompleted {
    /// Authenticated internal user owner.
    pub owner: TenantRef,
    /// Platform operation owning this recap.
    pub operation_id: OperationId,
    /// Stable digest run identity.
    pub digest_run_id: ChannelDigestRunId,
    /// Digest of the exact source manifest analyzed.
    pub manifest_digest: ContentDigest,
    /// Stable Knowledge analysis reference.
    pub analysis_ref: KnowledgeAnalysisRef,
    /// Stable identity of the durable result projection.
    pub digest_result_id: ChannelDigestResultId,
    /// Owner-authorized pointer to that result projection.
    pub result_ref: ChannelDigestResultRef,
    /// SHA-256 identity of the canonical result projection bytes.
    pub result_digest: ContentDigest,
    /// Knowledge-asserted durable completion instant.
    pub completed_at: WireTimestamp,
    /// Exact selected/included/omitted source coverage.
    pub coverage: ChannelDigestRecapCoverage,
    /// Unknown additive fields preserved by consumers; producers leave this empty.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for KnowledgeChannelDigestRecapCompleted {
    const EVENT_TYPE: &'static str = "knowledge.channel_digest_recap.completed.v1";
}

impl KnowledgeChannelDigestRecapCompleted {
    /// Validates coverage and result identity linkage.
    ///
    /// # Errors
    ///
    /// Returns the coverage error or [`ChannelDigestContractError::ResultReferenceMismatch`].
    pub fn validate(&self) -> Result<(), ChannelDigestContractError> {
        self.coverage.validate()?;
        if !self
            .result_ref
            .as_str()
            .ends_with(&self.digest_result_id.to_string())
        {
            return Err(ChannelDigestContractError::ResultReferenceMismatch);
        }
        Ok(())
    }

    /// Validates the completion and refuses producer-authored extensions.
    ///
    /// # Errors
    ///
    /// Returns any semantic error or
    /// [`ChannelDigestContractError::ProducerExtensionsNotEmpty`].
    pub fn validate_for_publish(&self) -> Result<(), ChannelDigestContractError> {
        self.validate()?;
        require_empty_extensions(&self.extensions)
    }
}

#[derive(serde::Deserialize)]
struct KnowledgeChannelDigestRecapCompletedWire {
    owner: TenantRef,
    operation_id: OperationId,
    digest_run_id: ChannelDigestRunId,
    manifest_digest: ContentDigest,
    analysis_ref: KnowledgeAnalysisRef,
    digest_result_id: ChannelDigestResultId,
    result_ref: ChannelDigestResultRef,
    result_digest: ContentDigest,
    completed_at: WireTimestamp,
    coverage: ChannelDigestRecapCoverage,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for KnowledgeChannelDigestRecapCompleted {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = KnowledgeChannelDigestRecapCompletedWire::deserialize(deserializer)?;
        let payload = Self {
            owner: wire.owner,
            operation_id: wire.operation_id,
            digest_run_id: wire.digest_run_id,
            manifest_digest: wire.manifest_digest,
            analysis_ref: wire.analysis_ref,
            digest_result_id: wire.digest_result_id,
            result_ref: wire.result_ref,
            result_digest: wire.result_digest,
            completed_at: wire.completed_at,
            coverage: wire.coverage,
            extensions: wire.extensions,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}

/// Closed content-free terminal failure code for a recap request.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ChannelDigestRecapFailureCode {
    /// Manifest could not be retrieved through the authenticated source interface.
    ManifestUnavailable,
    /// Manifest or referenced revision failed integrity validation.
    ManifestIntegrity,
    /// Requested output language is not supported by the active recap contract.
    UnsupportedLanguage,
    /// No complete source fits the configured context budget.
    ContextBudget,
    /// Provider was unavailable after bounded attempts.
    ProviderUnavailable,
    /// Provider did not complete within the bounded deadline.
    ProviderTimeout,
    /// Provider output remained invalid after the bounded repair attempt.
    InvalidOutput,
    /// Configured finite cost budget refused another attempt.
    CostBudget,
    /// The owning operation was cancelled.
    Cancelled,
}

/// Payload of `knowledge.channel_digest_recap.failed.v1`.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct KnowledgeChannelDigestRecapFailed {
    /// Authenticated internal user owner.
    pub owner: TenantRef,
    /// Platform operation owning this recap.
    pub operation_id: OperationId,
    /// Stable digest run identity.
    pub digest_run_id: ChannelDigestRunId,
    /// Digest of the exact source manifest requested.
    pub manifest_digest: ContentDigest,
    /// Closed content-free terminal reason.
    pub failure_code: ChannelDigestRecapFailureCode,
    /// Knowledge-asserted terminal failure instant.
    pub failed_at: WireTimestamp,
    /// Unknown additive fields preserved by consumers; producers leave this empty.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl EventPayload for KnowledgeChannelDigestRecapFailed {
    const EVENT_TYPE: &'static str = "knowledge.channel_digest_recap.failed.v1";
}

impl KnowledgeChannelDigestRecapFailed {
    /// Refuses producer-authored provider diagnostics or success linkage in extensions.
    ///
    /// # Errors
    ///
    /// Returns [`ChannelDigestContractError::ProducerExtensionsNotEmpty`] when an authored
    /// extension is present.
    pub fn validate_for_publish(&self) -> Result<(), ChannelDigestContractError> {
        require_empty_extensions(&self.extensions)
    }
}

fn require_empty_extensions(extensions: &Extensions) -> Result<(), ChannelDigestContractError> {
    if extensions.is_empty() {
        Ok(())
    } else {
        Err(ChannelDigestContractError::ProducerExtensionsNotEmpty)
    }
}
