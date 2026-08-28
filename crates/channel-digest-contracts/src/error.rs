//! Content-free channel-digest contract errors.

/// A stable reason a channel-digest wire value or payload is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ChannelDigestContractError {
    /// A public channel username does not match the canonical wire grammar.
    #[error("channel username is not canonical")]
    InvalidChannelUsername,
    /// A channel-digest record identity is not a canonical UUID.
    #[error("channel-digest identifier is not canonical")]
    InvalidIdentifier,
    /// A digest window is empty or ordered backwards.
    #[error("digest window must have an exclusive end after its start")]
    InvalidWindowOrder,
    /// A digest window exceeds the first-version seven-day bound.
    #[error("digest window exceeds seven days")]
    WindowTooLong,
    /// A recap source count is outside the inclusive first-version bound 1 through 100.
    #[error("digest source count must be between 1 and 100")]
    InvalidSourceCount,
    /// A recap channel count is outside the inclusive first-version bound 1 through 20.
    #[error("digest channel count must be between 1 and 20")]
    InvalidChannelCount,
    /// A run trigger's authoritative instant differs from the requested window end.
    #[error("digest run trigger instant must equal the window end")]
    TriggerWindowMismatch,
    /// A producer attempted to author an untyped additive member.
    #[error("producer-authored channel-digest extensions must be empty")]
    ProducerExtensionsNotEmpty,
    /// A request names more channels than selected source records.
    #[error("digest channel count cannot exceed source count")]
    ChannelCountExceedsSourceCount,
    /// Completion coverage does not add up to the selected source count.
    #[error("digest coverage counts are inconsistent")]
    InconsistentCoverage,
    /// A completion contains no complete included source record.
    #[error("digest completion must include at least one source")]
    EmptyIncludedCoverage,
    /// A result pointer does not name the declared result identity.
    #[error("digest result reference contradicts result identity")]
    ResultReferenceMismatch,
}
