//! [`SocialContractError`]: every way a social contract's invariants can fail.

/// Every way a social-source invariant can fail.
///
/// A *parse* error type: it is never serialized onto the wire — field-level rejection happens
/// in the identifier and vocabulary types, and only the cross-field rules surface here.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SocialContractError {
    /// Invariant S1: a capture declared `partial` carries no warning saying what is missing,
    /// which leaves the consumer unable to act on the gap.
    #[error("completeness `partial` requires at least one warning")]
    PartialWithoutWarning,
}
