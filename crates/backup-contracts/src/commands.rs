//! The command GitHub sends to Vault to apply a desired backup policy (XR-021 CONTRACTS.md S09).

use ratatoskr_event_envelope::CommandPayload;
use ratatoskr_identifiers::Extensions;

use crate::policy::DesiredBackupPolicy;

/// Payload of `vault.backup_policy.apply_requested.v1`: GitHub asks Vault to apply one version
/// of the desired backup policy.
///
/// The policy is the whole catalog: Vault treats the repositories it governs for GitHub and that
/// are absent from this document as no longer wanted. Vault answers with
/// `vault.backup_policy.acknowledged.v1` (`PolicyAcknowledged`), whose `causation_id` names this
/// command, and refuses a `policy_version` that does not exceed the last one it applied.
///
/// The enclosing command envelope is produced by `ratatoskr-github`; its `aggregate_id` and
/// `correlation_id` are `backup_policy:<policy_version>`, its `command_id` is the id of the
/// producer's outbox message, and it carries no `tenant_id` because the policy is catalog-wide.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct VaultBackupPolicyApplyRequested {
    /// The desired policy document for one version.
    pub policy: DesiredBackupPolicy,

    /// Unknown-but-preserved additive payload members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl CommandPayload for VaultBackupPolicyApplyRequested {
    const COMMAND_TYPE: &'static str = "vault.backup_policy.apply_requested.v1";
}
