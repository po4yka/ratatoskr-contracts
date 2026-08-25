//! [`NotificationContractError`]: every way a notification invariant can fail.

/// Every way a notification contract's invariants can fail.
///
/// A *parse* error type: it is never serialized onto the wire — field-level rejection happens in
/// the identifier and text types, and only the cross-field rules surface here.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NotificationContractError {
    /// A payload declared taxonomy registry version zero, which no registry can carry: versions
    /// start at 1, and zero would make "the producer speaks no vocabulary" unrepresentable
    /// silence rather than data.
    #[error("class_registry_version must be greater than zero")]
    ZeroRegistryVersion,
    /// A class token violated the taxonomy token grammar, so it can be neither recognized nor
    /// safely preserved: uppercase variants, over-length tokens and empty strings are typos or
    /// hostile input, not future vocabulary.
    #[error("notification class {input:?} does not match {pattern}")]
    MalformedClassToken {
        /// The offending class token, echoed for diagnosis only.
        input: String,
        /// The published token grammar, which is also the compiled runtime validator.
        pattern: &'static str,
    },
    /// A quiet-hours offset fell outside the seconds of one UTC day, so no daily window can name
    /// it.
    #[error("quiet-hours offset is {got} seconds; one UTC day has at most {max} seconds")]
    QuietHoursOffsetOutOfRange {
        /// The offset that was on the wire, in seconds from UTC midnight.
        got: u32,
        /// The largest offset one UTC day can name: `86_399`.
        max: u32,
    },
    /// A quiet-hours window named equal start and end offsets. Such a window cannot say whether
    /// it means an empty day or a full one, so a consumer would have to guess between "never
    /// deliver" and "always defer" — opposite behaviours from one ambiguous value.
    #[error("quiet-hours window must not start and end on the same offset")]
    QuietHoursWindowDegenerate,
}
