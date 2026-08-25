//! [`BackupContractError`]: every way a backup-policy invariant can fail.

/// Every way a backup contract's invariants can fail.
///
/// A *parse* error type: it is never serialized onto the wire — field-level rejection happens
/// in the identifier and expression types, and only the cross-field rules surface here.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BackupContractError {
    /// A policy document or acknowledgment named version zero, which no published policy can
    /// carry: versions are monotonic and start above zero.
    #[error("policy_version must be greater than zero")]
    ZeroPolicyVersion,
    /// A candidate policy version did not strictly exceed the previously applied version, so
    /// accepting it would replay or rewind state.
    #[error(
        "candidate policy_version {candidate} does not exceed the previously applied \
             version {previous}"
    )]
    SuccessionNotIncreasing {
        /// The version a successor document proposes.
        candidate: u64,
        /// The last version known to have been applied.
        previous: u64,
    },
    /// Two entries of one policy named the same repository, so the document contradicts itself
    /// about what that repository's backup depth is.
    #[error("policy names repository {repository_ref} in more than one entry")]
    DuplicateRepositoryEntry {
        /// The duplicated repository reference.
        repository_ref: String,
    },
    /// An exclusion expression was the empty string, which matches nothing and only obscures
    /// the entry's intent.
    #[error("exclusion expression must not be empty")]
    EmptyExclusionExpression,
    /// An exclusion expression exceeded the carrier-safety byte budget.
    #[error("exclusion expression is {got} bytes; the contract allows at most {max}")]
    ExclusionExpressionTooLong {
        /// The length the input carried, in UTF-8 bytes.
        got: usize,
        /// The contract maximum, in UTF-8 bytes.
        max: usize,
    },
    /// An exclusion expression carried a C0 control character or DEL, which makes the carrier
    /// unsafe: control bytes corrupt logs, terminals and line-oriented tooling downstream.
    #[error("exclusion expression contains a control character")]
    ExclusionExpressionControlCharacter,
    /// An acknowledgment named acknowledged version zero, which no published policy carries.
    #[error("acknowledged_policy_version must be greater than zero")]
    AcknowledgedVersionZero,
    /// Vault reported a rejection but gave no reason, leaving GitHub unable to act on it.
    #[error("outcome `rejected` requires at least one reason")]
    RejectedWithoutReason,
    /// Vault accepted a policy yet attached rejection reasons to the acceptance.
    #[error("outcome `accepted` forbids rejection reasons")]
    AcceptedWithReason,
    /// A reason carried a repository reference its code forbids: only
    /// `repository_unknown_in_catalog` concerns one specific repository.
    #[error("rejection reason with code `{code}` carries a repository reference it must not")]
    ReasonReferenceForbidden {
        /// The offending code's wire token.
        code: String,
    },
    /// A `repository_unknown_in_catalog` reason omitted the repository reference that names the
    /// unknown entry, leaving nothing to reconcile.
    #[error("rejection reason with code `{code}` requires a repository reference")]
    ReasonReferenceRequired {
        /// The offending code's wire token.
        code: String,
    },
    /// Vault accepted a policy version at or below the last one it fully applied, which is
    /// indistinguishable from a replayed acknowledgment.
    #[error(
        "accepted outcome acknowledges version {acknowledged} at or below \
             last_applied_policy_version {last_applied}"
    )]
    AcceptedWithoutProgress {
        /// The version the acceptance acknowledges.
        acknowledged: u64,
        /// The last version Vault had fully applied before this decision.
        last_applied: u64,
    },
}
