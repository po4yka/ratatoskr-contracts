//! The event the extractor publishes when a document has been extracted (XR-021 CONTRACTS.md
//! S09).

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{BlobRef, Extensions};

use crate::{Document, DocumentValidationError};

/// The only service that may own `document_blob` of a [`ContentDocumentExtracted`].
pub const DOCUMENT_BLOB_OWNER: &str = "ratatoskr-extractor";

/// Why a [`ContentDocumentExtracted`] payload is not acceptable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DocumentExtractedError {
    /// `document_blob` names bytes owned by a service other than the extractor.
    #[error("document_blob must be owned by ratatoskr-extractor, but it names `{actual}`")]
    ForeignDocumentBlob {
        /// The owner service the blob reference named.
        actual: String,
    },
    /// The embedded document breaks an invariant of the Document IR.
    #[error("the embedded document is invalid: {0}")]
    InvalidDocument(#[from] DocumentValidationError),
}

/// Payload of `content.document.extracted.v1`: the extractor finished extracting one document.
///
/// A fact, not a request. The document travels inline so the consumer never resolves an
/// extractor blob: the extractor has no blob service. `document_blob` is the extractor-owned
/// blob that stores the Document IR bytes, and it is the same reference the extractor reports as
/// the result of the operation, so a consumer can tell the two records describe one document.
///
/// The enclosing event envelope carries the owner (`tenant_id`, required), the aggregate
/// `document:<document_id>`, and the correlation and causation of the capture command that
/// started the work. The NATS payload limit bounds the document size; an oversized document is
/// not publishable.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ContentDocumentExtracted {
    /// The extracted document, exactly as stored.
    pub document: Document,

    /// The extractor-owned blob that stores the Document IR bytes.
    pub document_blob: BlobRef,

    /// Unknown-but-preserved additive payload members.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl ContentDocumentExtracted {
    /// Checks that the blob is extractor-owned and that the document is internally consistent.
    ///
    /// # Errors
    ///
    /// [`DocumentExtractedError::ForeignDocumentBlob`] when `document_blob` is owned by another
    /// service and [`DocumentExtractedError::InvalidDocument`] when the document breaks the
    /// Document IR invariants.
    pub fn validate(&self) -> Result<(), DocumentExtractedError> {
        let owner = self.document_blob.owner_service.as_str();
        if owner != DOCUMENT_BLOB_OWNER {
            return Err(DocumentExtractedError::ForeignDocumentBlob {
                actual: owner.to_owned(),
            });
        }
        self.document.validate()?;
        Ok(())
    }
}

impl EventPayload for ContentDocumentExtracted {
    const EVENT_TYPE: &'static str = "content.document.extracted.v1";
}
