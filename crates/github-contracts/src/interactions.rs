//! Repository preview and explicit-action wire contracts.

use ratatoskr_identifiers::wire_string_newtype;

use crate::values::{RepositoryDescription, RepositoryFullName, RepositoryLanguage};

wire_string_newtype! {
    /// Canonical HTTPS GitHub repository URL with exactly an owner and repository path.
    pub struct GitHubRepositoryUrl {
        pattern  = r"^https://github\.com/[A-Za-z0-9][A-Za-z0-9_.-]{0,99}/[A-Za-z0-9][A-Za-z0-9_.-]{0,99}$",
        max_len  = 220,
        examples = ["https://github.com/owner/repository"],
    }
}

wire_string_newtype! {
    /// Opaque reference to one connected GitHub account; never a credential.
    pub struct GitHubAccountRef {
        pattern  = r"^github-account:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        max_len  = 51,
        examples = ["github-account:018f0000-0000-7000-8000-000000000604"],
    }
}

wire_string_newtype! {
    /// Opaque bounded reference to the caller's completed confirmation evidence.
    pub struct ConfirmationEvidenceRef {
        pattern  = r"^[a-z][a-z0-9_-]{0,31}:[A-Za-z0-9][A-Za-z0-9._~:@+-]{0,127}$",
        max_len  = 161,
        examples = ["telegram-confirmation:018f0000-0000-7000-8000-000000000605"],
    }
}

wire_string_newtype! {
    /// Bounded opaque identity of one logical repository action.
    pub struct RepositoryActionIdempotencyKey {
        pattern  = r"^[A-Za-z0-9][A-Za-z0-9._~:@+-]{0,127}$",
        max_len  = 128,
        examples = ["telegram-github-action.018f0000-0000-7000-8000-000000000606"],
    }
}

/// Stable positive GitHub numeric repository identity.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "u64", into = "u64")]
pub struct GitHubRepositoryNumericId(u64);

impl GitHubRepositoryNumericId {
    /// Validates and wraps a GitHub numeric repository identity.
    ///
    /// # Errors
    ///
    /// [`RepositoryInteractionContractError::ZeroRepositoryNumericId`] when `value` is zero.
    pub fn new(value: u64) -> Result<Self, RepositoryInteractionContractError> {
        if value == 0 {
            Err(RepositoryInteractionContractError::ZeroRepositoryNumericId)
        } else {
            Ok(Self(value))
        }
    }

    /// The validated provider identity.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for GitHubRepositoryNumericId {
    type Error = RepositoryInteractionContractError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<GitHubRepositoryNumericId> for u64 {
    fn from(value: GitHubRepositoryNumericId) -> Self {
        value.0
    }
}

impl schemars::JsonSchema for GitHubRepositoryNumericId {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("GitHubRepositoryNumericId")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::GitHubRepositoryNumericId"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "integer",
            "title": "GitHubRepositoryNumericId",
            "description": "Stable positive GitHub numeric repository identity.",
            "minimum": 1,
            "examples": [42],
        })
    }
}

/// Validation failures specific to repository-interaction values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryInteractionContractError {
    /// GitHub repository numeric identities are positive.
    ZeroRepositoryNumericId,
    /// A provider-star action did not name the acting connected account.
    MissingStarAccount,
    /// A non-star action attempted to attach an acting provider account.
    UnexpectedAccount,
    /// A wire aggregate contradicted the component outcomes beside it.
    InconsistentAggregate,
}

impl core::fmt::Display for RepositoryInteractionContractError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroRepositoryNumericId => {
                formatter.write_str("GitHub repository numeric identity must be positive")
            }
            Self::MissingStarAccount => {
                formatter.write_str("star mode requires a connected GitHub account reference")
            }
            Self::UnexpectedAccount => {
                formatter.write_str("only star mode accepts a connected GitHub account reference")
            }
            Self::InconsistentAggregate => {
                formatter.write_str("repository action aggregate contradicts component outcomes")
            }
        }
    }
}

impl std::error::Error for RepositoryInteractionContractError {}

/// A read-only repository preview request.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct RepositoryPreviewRequest {
    /// Canonical GitHub repository URL supplied by the caller.
    pub repository_url: GitHubRepositoryUrl,
}

/// Stable provider identity and current display alias returned by a preview.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct RepositoryPreviewTarget {
    /// Stable positive GitHub numeric repository identity.
    pub github_repository_numeric_id: GitHubRepositoryNumericId,
    /// Current GitHub owner/name alias.
    pub repository_full_name: RepositoryFullName,
    /// Canonical HTTPS GitHub repository URL.
    pub canonical_url: GitHubRepositoryUrl,
}

/// Repository actions currently available to the authenticated caller.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RepositoryActionCapability {
    /// Catalog metadata only.
    Metadata,
    /// Catalog metadata plus desired backup tracking.
    Track,
    /// Provider star plus the catalog and desired backup steps.
    Star,
}

/// A bounded repository preview returned by GitHub Catalog.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct RepositoryPreviewResponse {
    /// Stable repository target.
    pub target: RepositoryPreviewTarget,
    /// Provider description when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<RepositoryDescription>,
    /// Provider stargazer count.
    pub stargazer_count: u64,
    /// Provider primary language when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_language: Option<RepositoryLanguage>,
    /// Acting connected-account reference when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_ref: Option<GitHubAccountRef>,
    /// Actions available to the authenticated caller.
    pub available_actions: Vec<RepositoryActionCapability>,
}

/// A confirmed request to apply one repository mode.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "RepositoryActionRequestWire")]
pub struct RepositoryActionRequest {
    /// Requested repository mode.
    pub mode: RepositoryActionCapability,
    /// Exact stable target returned by preview.
    pub target: RepositoryPreviewTarget,
    /// Acting connected account when required by the mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_ref: Option<GitHubAccountRef>,
    /// Opaque reference proving the caller completed its confirmation flow.
    pub confirmation_evidence_ref: ConfirmationEvidenceRef,
    /// Opaque key that gives one logical action a stable request identity.
    pub idempotency_key: RepositoryActionIdempotencyKey,
}

impl RepositoryActionRequest {
    /// Validates and constructs one confirmed repository action.
    ///
    /// # Errors
    ///
    /// [`RepositoryInteractionContractError::MissingStarAccount`] when `star` has no account;
    /// [`RepositoryInteractionContractError::UnexpectedAccount`] when another mode has one.
    pub fn new(
        mode: RepositoryActionCapability,
        target: RepositoryPreviewTarget,
        account_ref: Option<GitHubAccountRef>,
        confirmation_evidence_ref: ConfirmationEvidenceRef,
        idempotency_key: RepositoryActionIdempotencyKey,
    ) -> Result<Self, RepositoryInteractionContractError> {
        match (mode, account_ref.as_ref()) {
            (RepositoryActionCapability::Star, None) => {
                Err(RepositoryInteractionContractError::MissingStarAccount)
            }
            (RepositoryActionCapability::Metadata | RepositoryActionCapability::Track, Some(_)) => {
                Err(RepositoryInteractionContractError::UnexpectedAccount)
            }
            _ => Ok(Self {
                mode,
                target,
                account_ref,
                confirmation_evidence_ref,
                idempotency_key,
            }),
        }
    }
}

#[derive(schemars::JsonSchema, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "RepositoryActionRequest")]
struct RepositoryActionRequestWire {
    /// Requested repository mode.
    mode: RepositoryActionCapability,
    /// Exact stable target returned by preview.
    target: RepositoryPreviewTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Acting connected account when required by the mode.
    account_ref: Option<GitHubAccountRef>,
    /// Opaque reference proving the caller completed its confirmation flow.
    confirmation_evidence_ref: ConfirmationEvidenceRef,
    /// Opaque key that gives one logical action a stable request identity.
    idempotency_key: RepositoryActionIdempotencyKey,
}

impl schemars::JsonSchema for RepositoryActionRequest {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("RepositoryActionRequest")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::RepositoryActionRequest"))
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        RepositoryActionRequestWire::json_schema(generator)
    }
}

impl TryFrom<RepositoryActionRequestWire> for RepositoryActionRequest {
    type Error = RepositoryInteractionContractError;

    fn try_from(wire: RepositoryActionRequestWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.mode,
            wire.target,
            wire.account_ref,
            wire.confirmation_evidence_ref,
            wire.idempotency_key,
        )
    }
}

/// Safe reasons why a repository-action component was refused.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RepositoryActionRefusalReason {
    /// The authenticated user is not authorized for the target or account.
    NotAuthorized,
    /// No connected account can perform the requested provider action.
    AccountRequired,
    /// More than one account is available and a richer selection flow is required.
    AccountSelectionRequired,
    /// The connected account lacks the required provider scope.
    ScopeMissing,
    /// The stable target no longer matches the provider repository.
    TargetChanged,
}

/// Safe failure classes for an attempted repository-action component.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RepositoryActionFailureReason {
    /// A required internal dependency was unavailable.
    DependencyUnavailable,
    /// GitHub did not complete the provider request.
    ProviderUnavailable,
    /// The provider outcome may exist but no final response was observed.
    OutcomeUnknown,
    /// Catalog could not persist the observed metadata truth.
    CatalogPersistenceFailed,
    /// The desired backup policy could not be accepted for publication.
    PolicyPublicationFailed,
}

/// Safe reasons why a repository-action component was not attempted.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RepositoryActionSkipReason {
    /// The selected mode does not include this component.
    NotApplicable,
    /// A prerequisite component did not establish the state required to continue.
    PrerequisiteFailed,
}

/// Truthful outcome of the metadata component.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
pub enum RepositoryMetadataOutcome {
    /// Metadata was observed and persisted for this action.
    Succeeded,
    /// The same metadata state had already been applied.
    AlreadyApplied,
    /// Policy refused the component before it was attempted.
    Refused {
        /// Safe refusal class.
        reason: RepositoryActionRefusalReason,
    },
    /// The attempted component failed.
    Failed {
        /// Safe failure class.
        reason: RepositoryActionFailureReason,
    },
    /// The component was not attempted.
    Skipped {
        /// Safe skip class.
        reason: RepositoryActionSkipReason,
    },
}

/// Truthful outcome of the provider-star component.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
pub enum RepositoryProviderStarOutcome {
    /// GitHub acknowledged the provider star mutation.
    Succeeded,
    /// GitHub already had the repository starred by the acting account.
    AlreadyApplied,
    /// Policy refused the component before a provider mutation was attempted.
    Refused {
        /// Safe refusal class.
        reason: RepositoryActionRefusalReason,
    },
    /// The attempted provider component failed.
    Failed {
        /// Safe failure class.
        reason: RepositoryActionFailureReason,
    },
    /// The component was not attempted.
    Skipped {
        /// Safe skip class.
        reason: RepositoryActionSkipReason,
    },
}

/// Truthful outcome of desired-backup-policy handling.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
pub enum RepositoryDesiredBackupOutcome {
    /// The desired policy was accepted for publication; backup completion is not implied.
    Accepted,
    /// The same desired policy had already been accepted.
    AlreadyApplied,
    /// Policy refused the component before accepting desired state.
    Refused {
        /// Safe refusal class.
        reason: RepositoryActionRefusalReason,
    },
    /// The attempted desired-policy component failed.
    Failed {
        /// Safe failure class.
        reason: RepositoryActionFailureReason,
    },
    /// The component was not attempted.
    Skipped {
        /// Safe skip class.
        reason: RepositoryActionSkipReason,
    },
}

/// Aggregate truth derived from the three repository-action components.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RepositoryActionAggregate {
    /// Every applicable attempted component reached a positive state.
    Succeeded,
    /// Positive and negative component facts both exist.
    Partial,
    /// No component reached a positive state.
    Failed,
}

/// Aggregate and component-level truth returned for one repository action.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "RepositoryActionResultWire")]
pub struct RepositoryActionResult {
    /// Aggregate outcome derived from the three component facts.
    pub aggregate: RepositoryActionAggregate,
    /// Metadata component fact.
    pub metadata: RepositoryMetadataOutcome,
    /// Provider-star component fact.
    pub provider_star: RepositoryProviderStarOutcome,
    /// Desired-backup component fact.
    pub desired_backup: RepositoryDesiredBackupOutcome,
}

impl RepositoryActionResult {
    /// Constructs a result and derives its aggregate from the supplied component facts.
    #[must_use]
    pub fn new(
        metadata: RepositoryMetadataOutcome,
        provider_star: RepositoryProviderStarOutcome,
        desired_backup: RepositoryDesiredBackupOutcome,
    ) -> Self {
        let aggregate = derive_aggregate(&metadata, &provider_star, &desired_backup);
        Self {
            aggregate,
            metadata,
            provider_star,
            desired_backup,
        }
    }
}

#[derive(schemars::JsonSchema, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "RepositoryActionResult")]
struct RepositoryActionResultWire {
    /// Aggregate outcome derived from the three component facts.
    aggregate: RepositoryActionAggregate,
    /// Metadata component fact.
    metadata: RepositoryMetadataOutcome,
    /// Provider-star component fact.
    provider_star: RepositoryProviderStarOutcome,
    /// Desired-backup component fact.
    desired_backup: RepositoryDesiredBackupOutcome,
}

impl schemars::JsonSchema for RepositoryActionResult {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("RepositoryActionResult")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::RepositoryActionResult"))
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        RepositoryActionResultWire::json_schema(generator)
    }
}

impl TryFrom<RepositoryActionResultWire> for RepositoryActionResult {
    type Error = RepositoryInteractionContractError;

    fn try_from(wire: RepositoryActionResultWire) -> Result<Self, Self::Error> {
        let result = Self::new(wire.metadata, wire.provider_star, wire.desired_backup);
        if result.aggregate == wire.aggregate {
            Ok(result)
        } else {
            Err(RepositoryInteractionContractError::InconsistentAggregate)
        }
    }
}

#[derive(Clone, Copy)]
enum ComponentClass {
    Positive,
    Negative,
    Skipped,
}

fn derive_aggregate(
    metadata: &RepositoryMetadataOutcome,
    provider_star: &RepositoryProviderStarOutcome,
    desired_backup: &RepositoryDesiredBackupOutcome,
) -> RepositoryActionAggregate {
    let classes = [
        metadata_class(metadata),
        provider_star_class(provider_star),
        desired_backup_class(desired_backup),
    ];
    let positive = classes
        .iter()
        .any(|class| matches!(class, ComponentClass::Positive));
    let negative = classes
        .iter()
        .any(|class| matches!(class, ComponentClass::Negative));

    match (positive, negative) {
        (true, true) => RepositoryActionAggregate::Partial,
        (true, false) => RepositoryActionAggregate::Succeeded,
        (false, _) => RepositoryActionAggregate::Failed,
    }
}

fn metadata_class(outcome: &RepositoryMetadataOutcome) -> ComponentClass {
    match outcome {
        RepositoryMetadataOutcome::Succeeded | RepositoryMetadataOutcome::AlreadyApplied => {
            ComponentClass::Positive
        }
        RepositoryMetadataOutcome::Refused { .. } | RepositoryMetadataOutcome::Failed { .. } => {
            ComponentClass::Negative
        }
        RepositoryMetadataOutcome::Skipped { .. } => ComponentClass::Skipped,
    }
}

fn provider_star_class(outcome: &RepositoryProviderStarOutcome) -> ComponentClass {
    match outcome {
        RepositoryProviderStarOutcome::Succeeded
        | RepositoryProviderStarOutcome::AlreadyApplied => ComponentClass::Positive,
        RepositoryProviderStarOutcome::Refused { .. }
        | RepositoryProviderStarOutcome::Failed { .. } => ComponentClass::Negative,
        RepositoryProviderStarOutcome::Skipped { .. } => ComponentClass::Skipped,
    }
}

fn desired_backup_class(outcome: &RepositoryDesiredBackupOutcome) -> ComponentClass {
    match outcome {
        RepositoryDesiredBackupOutcome::Accepted
        | RepositoryDesiredBackupOutcome::AlreadyApplied => ComponentClass::Positive,
        RepositoryDesiredBackupOutcome::Refused { .. }
        | RepositoryDesiredBackupOutcome::Failed { .. } => ComponentClass::Negative,
        RepositoryDesiredBackupOutcome::Skipped { .. } => ComponentClass::Skipped,
    }
}
