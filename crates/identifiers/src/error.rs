//! The single parse-error type shared by every identifier in this crate.

/// Every way a Ratatoskr wire identifier can fail to parse.
///
/// This is a *parse* error type. It is never serialized onto the wire — that is
/// `ratatoskr_error_contracts::ErrorEnvelope`'s job — so echoing the offending input here cannot
/// leak into a broadcast payload.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum IdentifierError {
    /// The input was the empty string. No wire identifier permits it.
    #[error("{type_name} must not be empty")]
    Empty {
        /// Rust name of the identifier type that rejected the input.
        type_name: &'static str,
    },
    /// The input is longer than the contract maximum, measured in UTF-8 bytes.
    #[error("{type_name} is {got} bytes; the contract maximum is {max}")]
    TooLong {
        /// Rust name of the identifier type that rejected the input.
        type_name: &'static str,
        /// Length of the offending input in UTF-8 bytes.
        got: usize,
        /// The contract maximum length in UTF-8 bytes.
        max: usize,
    },
    /// The input does not match the published JSON Schema `pattern` for the type.
    #[error("{type_name} {input:?} does not match {pattern}")]
    PatternMismatch {
        /// Rust name of the identifier type that rejected the input.
        type_name: &'static str,
        /// The published pattern, which is also the compiled runtime validator.
        pattern: &'static str,
        /// The offending input, echoed for diagnosis only.
        input: String,
    },
    /// A qualified reference carried no `:` separating the kind from the local identity.
    #[error("entity reference {input:?} must be `<kind>:<local_id>`")]
    MissingKindSeparator {
        /// The offending input, echoed for diagnosis only.
        input: String,
    },
    /// A qualified reference named an entity kind the target type does not accept.
    #[error("expected entity kind {expected:?}, found {actual:?}")]
    KindMismatch {
        /// The only kind the target type accepts.
        expected: &'static str,
        /// The kind that was actually present on the wire.
        actual: String,
    },
    /// The local half of a qualified reference had to be a UUID here, and was not.
    #[error("entity local id {local_id:?} is not a canonical lowercase hyphenated UUID")]
    NotAUuid {
        /// The offending local identity, echoed for diagnosis only.
        local_id: String,
    },
    /// The input is not an RFC 3339 instant at all.
    #[error("instant {input:?} is not RFC 3339: {reason}")]
    MalformedInstant {
        /// The offending input, echoed for diagnosis only.
        input: String,
        /// The underlying parser's explanation.
        reason: String,
    },
    /// The input is a legal RFC 3339 instant, but not the one canonical spelling of that instant.
    #[error(
        "instant {input:?} is not in canonical form; the only accepted spelling of this \
             instant is {canonical:?}"
    )]
    NonCanonicalInstant {
        /// The offending input, echoed for diagnosis only.
        input: String,
        /// The one spelling of this instant that the contract accepts.
        canonical: String,
    },
}
