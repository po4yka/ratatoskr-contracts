//! Document IR version one.

use std::collections::BTreeSet;
use std::fmt;

use ratatoskr_identifiers::{BlobRef, BlockId, ContentDigest, DocumentId, wire_string_newtype};

wire_string_newtype! {
    /// URI from which the document was extracted.
    pub struct DocumentAddress {
        pattern  = r"^[a-z][a-z0-9+.-]{1,31}:[^ \x09-\x0d]{1,2015}$",
        max_len  = 2048,
        examples = ["document:fixture-source"],
    }
}

wire_string_newtype! {
    /// BCP 47 language tag asserted for the document.
    pub struct LanguageTag {
        pattern  = r"^[A-Za-z]{2,8}(-[A-Za-z0-9]{1,8})*$",
        max_len  = 63,
        examples = ["en", "en-US"],
    }
}

wire_string_newtype! {
    /// Extraction strategy that produced the normalized blocks.
    pub struct ExtractionStrategy {
        pattern  = r"^[a-z][a-z0-9_]{0,63}$",
        max_len  = 64,
        examples = ["semantic_article", "readability"],
    }
}

/// Canonical Document IR version one.
///
/// The shape is the shared intersection between Extractor and Knowledge. Rendered Markdown,
/// quality-scoring diagnostics and service-private storage fields are deliberately absent.
///
/// # Content hashing
///
/// `content_digest` is SHA-256 over the UTF-8 bytes of the repository's canonical JSON rendering
/// of blocks with their identifiers omitted. Block order, discriminants and text are significant;
/// identity, source address, title, language, block identifiers and provenance are not. A producer
/// does not rewrite extracted text after it enters a block.
#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Document {
    /// Stable identity assigned to the normalized document.
    pub document_id: DocumentId,

    /// URI from which the source was acquired.
    pub source_address: DocumentAddress,

    /// Digest of the normalized document content.
    pub content_digest: ContentDigest,

    /// Extracted title. Absent when the source provides no usable title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Document language. Absent when it cannot be determined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<LanguageTag>,

    /// Typed content blocks in reading order.
    pub blocks: Vec<DocumentBlock>,

    /// Block-addressable evidence that identifies the stored source and extraction strategy.
    pub provenance: Vec<DocumentProvenance>,
}

impl Document {
    /// Verifies cross-block invariants of this immutable Document IR revision.
    ///
    /// # Errors
    ///
    /// Returns [`DocumentValidationError::DuplicateBlockIdentifier`] when two blocks use the same
    /// stable identifier.
    pub fn validate(&self) -> Result<(), DocumentValidationError> {
        let mut identifiers = BTreeSet::new();
        for block in &self.blocks {
            let block_id = match block {
                DocumentBlock::Heading { block_id, .. }
                | DocumentBlock::Paragraph { block_id, .. } => *block_id,
            };
            if !identifiers.insert(block_id) {
                return Err(DocumentValidationError::DuplicateBlockIdentifier(block_id));
            }
        }
        Ok(())
    }
}

impl<'de> serde::Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let document: Self = DocumentWire::deserialize(deserializer).map(Into::into)?;
        document.validate().map_err(serde::de::Error::custom)?;
        Ok(document)
    }
}

/// The wire shape validated before it becomes a [`Document`].
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentWire {
    document_id: DocumentId,
    source_address: DocumentAddress,
    content_digest: ContentDigest,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    language: Option<LanguageTag>,
    blocks: Vec<DocumentBlock>,
    provenance: Vec<DocumentProvenance>,
}

impl From<DocumentWire> for Document {
    fn from(value: DocumentWire) -> Self {
        Self {
            document_id: value.document_id,
            source_address: value.source_address,
            content_digest: value.content_digest,
            title: value.title,
            language: value.language,
            blocks: value.blocks,
            provenance: value.provenance,
        }
    }
}

/// A cross-block invariant failure in immutable Document IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentValidationError {
    /// Two blocks in one immutable document revision have the same identifier.
    DuplicateBlockIdentifier(BlockId),
}

impl fmt::Display for DocumentValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateBlockIdentifier(block_id) => {
                write!(formatter, "duplicate block identifier: {block_id}")
            }
        }
    }
}

impl std::error::Error for DocumentValidationError {}

/// One typed block in document reading order.
///
/// The variant set and its fields grow only through the block-kind extension procedure
/// recorded in ADR-0010 (`docs/adr/0010-document-ir-block-kind-extension-procedure.md`):
/// the producer proposes with a landed extraction path, the consumer accepts by naming a
/// consumption site, and readers reject kinds they do not know at both layers.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
pub enum DocumentBlock {
    /// A section heading.
    Heading {
        /// Stable typed identity within this immutable document revision.
        block_id: BlockId,
        /// Source heading level.
        level: u8,
        /// Heading text.
        text: String,
    },
    /// A paragraph of prose.
    Paragraph {
        /// Stable typed identity within this immutable document revision.
        block_id: BlockId,
        /// Paragraph text.
        text: String,
    },
}

/// Evidence for how and from which stored bytes a document was extracted.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DocumentProvenance {
    /// Zero-based index of the block this evidence describes.
    pub block_index: u32,

    /// Strategy that produced the normalized blocks.
    pub extraction_strategy: ExtractionStrategy,

    /// Content-addressed raw source owned by the extracting service.
    pub source_blob: BlobRef,
}
