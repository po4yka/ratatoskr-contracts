//! The desired-backup-policy document, its entries and vocabularies, and the drift semantics
//! expressed as pure functions.

use ratatoskr_event_envelope::ProducerName;
use ratatoskr_identifiers::{EntityRef, Extensions, WireTimestamp};

use crate::error::BackupContractError;

/// How often a repository's mirror is refreshed.
///
/// **Closed on purpose.** The class decides backup frequency, so an unrecognized value must stop
/// processing instead of being read as some default cadence. Coarse classes only: mapping a
/// class onto concrete intervals is Vault-scheduler territory, not wire grammar. Adding a
/// variant later is an additive, non-breaking expansion.
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
pub enum MirrorCadence {
    /// Mirror immediately on every observed change.
    Eager,
    /// Refresh the mirror at least once per UTC day.
    Daily,
    /// Refresh the mirror at least once per UTC week.
    Weekly,
}

/// How urgently a repository's mirror should be brought up relative to its siblings.
///
/// **Closed on purpose**: ordering drives scheduling under contention, and an unrecognized hint
/// silently reorders backups. A hint, never a guarantee — Vault schedules within its own
/// capacity.
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
pub enum BackupPriorityHint {
    /// Mirror before anything else; loss of fresh state is unacceptable.
    Critical,
    /// Ordinary scheduling.
    Standard,
    /// Mirror only when capacity allows; archival depth matters more than freshness.
    Bulk,
}

/// What an exclusion's expression matches inside a repository.
///
/// **Closed on purpose**: the scope decides which matcher interprets the expression downstream,
/// and guessing a scope silently changes what is mirrored.
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
pub enum BackupExclusionScope {
    /// The expression matches git refnames (branches and tags) of the mirrored repository.
    RefsMatching,
    /// The expression matches paths inside the mirrored tree.
    PathsMatching,
}

/// One validated exclusion expression.
///
/// Carrier safety only: non-empty, at most [`Self::MAX_LEN`] UTF-8 bytes, no C0 control
/// character and no DEL. Whether an expression names refname globs or path patterns is decided
/// by the enclosing entry's [`BackupExclusionScope`], and its matching semantics belong to the
/// consuming mirror implementation — this contract fixes the carrier, not the glob grammar.
///
/// The published JSON Schema carries only the length bounds: draft 2020-12 cannot express a
/// character-class ban here without the pattern also freezing the whole alphabet, so the
/// control-character rule is enforced in `Deserialize` and pinned by an invalid fixture naming
/// the `serde` layer (ADR-0001 lower bound).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct ExclusionExpression(String);

impl ExclusionExpression {
    /// Contract maximum length in UTF-8 bytes, published as JSON Schema `maxLength`.
    pub const MAX_LEN: usize = 256;

    /// Validates `raw` against the carrier-safety contract and wraps it.
    ///
    /// # Errors
    ///
    /// [`BackupContractError::EmptyExclusionExpression`] for the empty string,
    /// [`BackupContractError::ExclusionExpressionTooLong`] beyond [`Self::MAX_LEN`] UTF-8 bytes,
    /// [`BackupContractError::ExclusionExpressionControlCharacter`] when `raw` carries a C0
    /// control character or DEL.
    pub fn parse(raw: &str) -> Result<Self, BackupContractError> {
        if raw.is_empty() {
            return Err(BackupContractError::EmptyExclusionExpression);
        }
        let byte_len = raw.len();
        if byte_len > Self::MAX_LEN {
            return Err(BackupContractError::ExclusionExpressionTooLong {
                got: byte_len,
                max: Self::MAX_LEN,
            });
        }
        if raw.chars().any(char::is_control) {
            return Err(BackupContractError::ExclusionExpressionControlCharacter);
        }
        Ok(Self(raw.to_owned()))
    }

    /// The validated expression text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for ExclusionExpression {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl core::str::FromStr for ExclusionExpression {
    type Err = BackupContractError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

impl core::convert::TryFrom<String> for ExclusionExpression {
    type Error = BackupContractError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<ExclusionExpression> for String {
    fn from(value: ExclusionExpression) -> Self {
        value.0
    }
}

impl schemars::JsonSchema for ExclusionExpression {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ExclusionExpression")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::ExclusionExpression"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "title": "ExclusionExpression",
            "description": "One exclusion expression: carrier-safe opaque text whose \
                            interpretation - refname globs or path patterns - follows the \
                            enclosing exclusion's scope. Non-empty and at most 256 UTF-8 bytes; \
                            the ban on C0 control characters and DEL is enforced by the Rust \
                            type and is narrower than this schema.",
            "minLength": 1,
            "maxLength": Self::MAX_LEN,
            "examples": ["refs/heads/scratch/*", "tmp/**"],
        })
    }
}

/// One explicit narrowing of a repository's mirrored set.
///
/// An exclusion only ever removes candidates; it can never widen what is mirrored.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct BackupExclusion {
    /// Which kind of thing `expression` matches. Required.
    pub scope: BackupExclusionScope,

    /// The carrier-safe expression the consuming mirror applies. Required.
    pub expression: ExclusionExpression,
}

/// One repository's desired backup depth inside a policy version.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct RepositoryBackupEntry {
    /// The repository this entry governs, spelled `repository:<uuid>` in the shared pointer
    /// grammar. Required. Two entries of one policy never name the same reference.
    pub repository_ref: EntityRef,

    /// Coarse refresh-cadence class. Required; see [`MirrorCadence`].
    pub mirror_cadence: MirrorCadence,

    /// Relative urgency hint. Required; see [`BackupPriorityHint`].
    pub priority_hint: BackupPriorityHint,

    /// Upper-bound size estimate for the repository, in bytes. Omitted when the producer does
    /// not know it; never zero-meaningful.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_hint_bytes: Option<u64>,

    /// Explicit exclusions narrowing the mirrored set. Omitted when nothing is excluded; an
    /// exclusion only ever narrows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclusions: Vec<BackupExclusion>,
}

/// The versioned policy document GitHub publishes: which repositories must be preserved and at
/// what depth.
///
/// Named `DesiredBackupPolicy`, not `BackupPolicy`, because it states intent, not achieved
/// state: Vault answers each version through [`PolicyAcknowledged`]
/// (`crate::PolicyAcknowledged`). Successive documents for the same estate carry strictly
/// increasing `policy_version` values; succession itself is checked by
/// [`validate_policy_succession`], because one document cannot know its predecessor.
///
/// `Deserialize` is hand-written because two invariants are cross-field: the version floor and
/// per-document uniqueness of repository references. It parses a private mirror struct and then
/// checks. A field added to the public struct and not to the mirror would be silently dropped;
/// the committed full-policy fixture round-trips through `cargo contracts check`, which fails
/// the moment that happens.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct DesiredBackupPolicy {
    /// Monotonic version of this document. Required; greater than zero, and each successor for
    /// the same estate carries a strictly greater value.
    pub policy_version: u64,

    /// Deployable service that produced this document, e.g. `ratatoskr-github`. Required.
    pub producing_service: ProducerName,

    /// Instant this document was produced, on the producing service's clock. Required.
    /// Producer-asserted authority: it is neither a publish nor a receive instant, and no other
    /// party can verify it.
    pub produced_at: WireTimestamp,

    /// Per-repository entries. Omitted when this version names no repository, which leaves
    /// every catalog repository uncovered until a successor names it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repositories: Vec<RepositoryBackupEntry>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl DesiredBackupPolicy {
    /// Re-checks every cross-field invariant. `Deserialize` calls this; a producer that mutates
    /// a policy in place calls it again before emitting.
    ///
    /// # Errors
    ///
    /// [`BackupContractError::ZeroPolicyVersion`] when `policy_version` is zero,
    /// [`BackupContractError::DuplicateRepositoryEntry`] when two entries name the same
    /// repository reference.
    pub fn validate(&self) -> Result<(), BackupContractError> {
        if self.policy_version == 0 {
            return Err(BackupContractError::ZeroPolicyVersion);
        }

        let mut seen: std::collections::HashSet<String> =
            std::collections::HashSet::with_capacity(self.repositories.len());
        for entry in &self.repositories {
            let wire = entry.repository_ref.to_wire();
            if !seen.insert(wire.clone()) {
                return Err(BackupContractError::DuplicateRepositoryEntry {
                    repository_ref: wire,
                });
            }
        }

        Ok(())
    }
}

/// The wire mirror of [`DesiredBackupPolicy`], parsed before the cross-field invariants run.
///
/// Serde offers no post-deserialization validation hook, so the public type's `Deserialize` is
/// hand-written and delegates the field-by-field work here.
///
/// **Drift guard:** a field added to [`DesiredBackupPolicy`] and not to this mirror is silently
/// dropped on parse; the full-policy fixture round-trip fails the moment that happens. Do not
/// add a field to one of these structs alone.
#[derive(Debug, serde::Deserialize)]
struct DesiredBackupPolicyWire {
    policy_version: u64,
    producing_service: ProducerName,
    produced_at: WireTimestamp,
    #[serde(default)]
    repositories: Vec<RepositoryBackupEntry>,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for DesiredBackupPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = DesiredBackupPolicyWire::deserialize(deserializer)?;
        let policy = Self {
            policy_version: wire.policy_version,
            producing_service: wire.producing_service,
            produced_at: wire.produced_at,
            repositories: wire.repositories,
            extensions: wire.extensions,
        };
        policy.validate().map_err(serde::de::Error::custom)?;
        Ok(policy)
    }
}

/// Checks that a candidate policy version may succeed the previously applied one.
///
/// Pure succession grammar: versions start above zero and strictly increase. This is the check
/// a publisher runs before emitting a successor and a consumer runs on receipt; it is a free
/// function rather than part of [`DesiredBackupPolicy::validate`] because one document cannot
/// know which version preceded it.
///
/// # Errors
///
/// [`BackupContractError::ZeroPolicyVersion`] when `candidate_version` is zero,
/// [`BackupContractError::SuccessionNotIncreasing`] when it does not strictly exceed
/// `previously_applied_version`.
pub fn validate_policy_succession(
    candidate_version: u64,
    previously_applied_version: u64,
) -> Result<(), BackupContractError> {
    if candidate_version == 0 {
        return Err(BackupContractError::ZeroPolicyVersion);
    }
    if candidate_version <= previously_applied_version {
        return Err(BackupContractError::SuccessionNotIncreasing {
            candidate: candidate_version,
            previous: previously_applied_version,
        });
    }
    Ok(())
}

/// Lists catalog repositories no entry of the current policy names.
///
/// A catalog repository absent from the policy is out of scope until a future version names
/// it; this function makes that set enumerable so an operator can see what a version does not
/// cover. Order follows the catalog; nothing is deduplicated beyond what the caller supplies.
#[must_use]
pub fn uncovered_catalog_repositories(
    catalog: &[EntityRef],
    entries: &[RepositoryBackupEntry],
) -> Vec<EntityRef> {
    let named: std::collections::HashSet<String> = entries
        .iter()
        .map(|entry| entry.repository_ref.to_wire())
        .collect();
    let mut uncovered = Vec::new();
    for reference in catalog {
        if !named.contains(&reference.to_wire()) {
            uncovered.push(reference.clone());
        }
    }
    uncovered
}

/// Lists policy entries whose repository reference is absent from the catalog snapshot.
///
/// An entry pointing outside the catalog is reportable drift, never a silent skip: the policy
/// asks for something the catalog cannot name. Order follows the entries.
#[must_use]
pub fn entries_absent_from_catalog(
    catalog: &[EntityRef],
    entries: &[RepositoryBackupEntry],
) -> Vec<EntityRef> {
    let known: std::collections::HashSet<String> = catalog.iter().map(EntityRef::to_wire).collect();
    let mut absent = Vec::new();
    for entry in entries {
        if !known.contains(&entry.repository_ref.to_wire()) {
            absent.push(entry.repository_ref.clone());
        }
    }
    absent
}

/// Applies exclusions to a scoped candidate set through a caller-supplied match predicate.
///
/// The result is an order-preserving subset of `values`: an exclusion only ever narrows. The
/// predicate decides whether one candidate value matches one exclusion, which keeps glob
/// semantics out of the contract; a value is dropped when any exclusion matches it.
#[must_use]
pub fn apply_exclusions<'a, T>(
    values: &'a [T],
    exclusions: &[BackupExclusion],
    mut is_excluded: impl FnMut(&T, &BackupExclusion) -> bool,
) -> Vec<&'a T> {
    values
        .iter()
        .filter(|value| {
            !exclusions
                .iter()
                .any(|exclusion| is_excluded(value, exclusion))
        })
        .collect()
}
