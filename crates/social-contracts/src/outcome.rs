//! The closed outcome taxonomy for an explicit social capture.

use core::convert::TryFrom;

use ratatoskr_error_contracts::ErrorCode;

/// The machine-actionable terminal or partial outcome of a social capture.
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
#[non_exhaustive]
pub enum SocialCaptureOutcomeCode {
    /// The provider could not resolve the public source.
    #[serde(rename = "social.source.unavailable")]
    SourceUnavailable,
    /// The provider indicated that the source no longer exists.
    #[serde(rename = "social.source.deleted")]
    SourceDeleted,
    /// The social post was preserved but linked-article extraction failed.
    #[serde(rename = "social.linked_article.extraction_failed")]
    LinkedArticleExtractionFailed,
}

/// An unrecognised social-capture outcome spelling.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown social capture outcome code {input:?}")]
pub struct SocialCaptureOutcomeCodeError {
    /// The unsupported wire spelling.
    pub input: String,
}

impl SocialCaptureOutcomeCode {
    /// Parses a closed social-capture outcome spelling.
    ///
    /// # Errors
    ///
    /// Returns [`SocialCaptureOutcomeCodeError`] for every spelling outside this taxonomy.
    pub fn parse(raw: &str) -> Result<Self, SocialCaptureOutcomeCodeError> {
        match raw {
            "social.source.unavailable" => Ok(Self::SourceUnavailable),
            "social.source.deleted" => Ok(Self::SourceDeleted),
            "social.linked_article.extraction_failed" => Ok(Self::LinkedArticleExtractionFailed),
            _ => Err(SocialCaptureOutcomeCodeError {
                input: raw.to_owned(),
            }),
        }
    }

    /// The stable wire spelling of this taxonomy member.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceUnavailable => "social.source.unavailable",
            Self::SourceDeleted => "social.source.deleted",
            Self::LinkedArticleExtractionFailed => "social.linked_article.extraction_failed",
        }
    }

    /// Whether this outcome means no social post was preserved.
    #[must_use]
    pub const fn is_terminal_source_failure(self) -> bool {
        matches!(self, Self::SourceUnavailable | Self::SourceDeleted)
    }

    /// Maps this closed taxonomy member into the shared error-envelope vocabulary.
    ///
    /// # Panics
    ///
    /// Only if a static spelling above no longer satisfies `ErrorCode` grammar, which the
    /// taxonomy test guards as a contract regression.
    #[must_use]
    #[allow(
        clippy::expect_used,
        reason = "each static closed taxonomy spelling is covered by the contract test"
    )]
    pub fn error_code(self) -> ErrorCode {
        ErrorCode::parse(self.as_str())
            .expect("the static social outcome code satisfies ErrorCode grammar")
    }
}

impl TryFrom<String> for SocialCaptureOutcomeCode {
    type Error = SocialCaptureOutcomeCodeError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<SocialCaptureOutcomeCode> for String {
    fn from(value: SocialCaptureOutcomeCode) -> Self {
        value.as_str().to_owned()
    }
}
