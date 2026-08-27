//! The typed pointer to what an operation produced.

use ratatoskr_ai_archive_contracts::AiArchiveOperationSummary;
use ratatoskr_identifiers::{BlobRef, EntityRef, Extensions};

use crate::error::OperationContractError;
use crate::kind::OperationResultKind;

/// A typed pointer to something the operation produced.
///
/// A pointer, never inlined content: an operation may produce megabytes, and
/// `ARCHITECTURE.md` S14 requires user content to be separable from metadata so telemetry can
/// omit it.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
pub struct OperationResultRef {
    /// What the target is.
    pub result_kind: OperationResultKind,

    /// Namespaced reference to the produced entity, e.g. `document:018f…`.
    pub target: EntityRef,

    /// Content-addressed handle when the result is stored bytes rather than a modelled entity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<BlobRef>,

    /// Privacy-safe summary of an imported AI archive when this result is `ai_archive.import`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_archive_import_summary: Option<AiArchiveOperationSummary>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl OperationResultRef {
    /// Verifies the association between an optional AI archive summary and its generic result.
    ///
    /// # Errors
    ///
    /// Returns [`OperationContractError::AiArchiveSummaryUnexpectedResultKind`] when the summary
    /// is attached to a result other than `ai_archive.import`, and
    /// [`OperationContractError::AiArchiveSummaryTargetMismatch`] when the target does not name
    /// the archive held by that summary.
    pub fn validate(&self) -> Result<(), OperationContractError> {
        let Some(summary) = &self.ai_archive_import_summary else {
            return Ok(());
        };

        if self.result_kind.as_str() != "ai_archive.import" {
            return Err(
                OperationContractError::AiArchiveSummaryUnexpectedResultKind {
                    actual: self.result_kind.as_str().to_owned(),
                },
            );
        }

        let expected = EntityRef::from(summary.ai_archive_id);
        if self.target != expected {
            return Err(OperationContractError::AiArchiveSummaryTargetMismatch {
                expected: expected.to_wire(),
                actual: self.target.to_wire(),
            });
        }

        Ok(())
    }
}

/// The wire mirror of [`OperationResultRef`], parsed before its cross-field association check.
#[derive(serde::Deserialize)]
struct OperationResultRefWire {
    result_kind: OperationResultKind,
    target: EntityRef,
    #[serde(default)]
    blob: Option<BlobRef>,
    #[serde(default)]
    ai_archive_import_summary: Option<AiArchiveOperationSummary>,
    #[serde(flatten)]
    extensions: Extensions,
}

impl<'de> serde::Deserialize<'de> for OperationResultRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = OperationResultRefWire::deserialize(deserializer)?;
        let result = Self {
            result_kind: wire.result_kind,
            target: wire.target,
            blob: wire.blob,
            ai_archive_import_summary: wire.ai_archive_import_summary,
            extensions: wire.extensions,
        };
        result.validate().map_err(serde::de::Error::custom)?;
        Ok(result)
    }
}
