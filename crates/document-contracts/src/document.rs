//! Document IR version one.

use ratatoskr_identifiers::{BlobRef, ContentDigest, DocumentId, wire_string_newtype};

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
/// of `blocks` alone. Block order, discriminants and text are significant; identity, source
/// address, title, language and provenance are not. A producer does not rewrite extracted text
/// after it enters a block.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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

/// One typed block in document reading order.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
pub enum DocumentBlock {
    /// A section heading.
    Heading {
        /// Source heading level.
        level: u8,
        /// Heading text.
        text: String,
    },
    /// A paragraph of prose.
    Paragraph {
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
