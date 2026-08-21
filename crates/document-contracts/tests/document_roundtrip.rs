//! Document IR version one round-trip tests.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_document_contracts::{
    Document, DocumentAddress, DocumentBlock, DocumentProvenance, ExtractionStrategy, LanguageTag,
};
use ratatoskr_identifiers::{
    BlobOwner, BlobRef, ContentDigest, DigestAlgorithm, DigestHex, DocumentId, MediaType,
};

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn a_document_with_two_block_kinds_and_provenance_round_trips() {
    let digest = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        hex: DigestHex::parse(DIGEST).expect("a SHA-256 digest"),
    };
    let document = Document {
        document_id: DocumentId::parse("018f0000-0000-7000-8000-000000000021")
            .expect("a document id"),
        source_address: DocumentAddress::parse("document:fixture-source")
            .expect("a source address"),
        content_digest: digest.clone(),
        title: Some("An example".to_owned()),
        language: Some(LanguageTag::parse("en").expect("a language tag")),
        blocks: vec![
            DocumentBlock::Heading {
                level: 1,
                text: "An example".to_owned(),
            },
            DocumentBlock::Paragraph {
                text: "The first paragraph.".to_owned(),
            },
        ],
        provenance: vec![
            DocumentProvenance {
                block_index: 0,
                extraction_strategy: ExtractionStrategy::parse("semantic_article")
                    .expect("a strategy"),
                source_blob: BlobRef {
                    owner_service: BlobOwner::parse("ratatoskr-extractor").expect("a blob owner"),
                    digest: digest.clone(),
                    media_type: MediaType::parse("text/html").expect("a media type"),
                    length_bytes: 42,
                },
            },
            DocumentProvenance {
                block_index: 1,
                extraction_strategy: ExtractionStrategy::parse("semantic_article")
                    .expect("a strategy"),
                source_blob: BlobRef {
                    owner_service: BlobOwner::parse("ratatoskr-extractor").expect("a blob owner"),
                    digest,
                    media_type: MediaType::parse("text/html").expect("a media type"),
                    length_bytes: 42,
                },
            },
        ],
    };

    let wire = serde_json::to_string(&document).expect("the document serializes");
    let decoded: Document = serde_json::from_str(&wire).expect("the document deserializes");

    assert_eq!(decoded, document);
    assert!(wire.find("heading").expect("heading") < wire.find("paragraph").expect("paragraph"));
}
