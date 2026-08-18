//! The typed pointer to what an operation produced.

use ratatoskr_identifiers::{BlobRef, EntityRef, Extensions};

use crate::kind::OperationResultKind;

/// A typed pointer to something the operation produced.
///
/// A pointer, never inlined content: an operation may produce megabytes, and
/// `ARCHITECTURE.md` S14 requires user content to be separable from metadata so telemetry can
/// omit it.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct OperationResultRef {
    /// What the target is.
    pub result_kind: OperationResultKind,

    /// Namespaced reference to the produced entity, e.g. `document:018f…`.
    pub target: EntityRef,

    /// Content-addressed handle when the result is stored bytes rather than a modelled entity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<BlobRef>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}
