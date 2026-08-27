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
    BlobOwner, BlobRef, BlockId, ContentDigest, DigestAlgorithm, DigestHex, DocumentId, MediaType,
};

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn document_blocks_round_trip_with_distinct_stable_identifiers() {
    let fixture = include_str!("../../../fixtures/content/document/valid/two-blocks.json");
    let decoded: Document = serde_json::from_str(fixture)
        .expect("a document block id must be accepted by the current Document IR contract");
    let wire = serde_json::to_value(&decoded).expect("the document serializes");
    let blocks = wire
        .get("blocks")
        .and_then(serde_json::Value::as_array)
        .expect("blocks are an array");
    let first_id = blocks
        .first()
        .and_then(|block| block.get("block_id"))
        .expect("the first block has an identifier");
    let second_id = blocks
        .get(1)
        .and_then(|block| block.get("block_id"))
        .expect("the second block has an identifier");

    assert_ne!(
        first_id, second_id,
        "distinct blocks require distinct stable identifiers"
    );
}

#[test]
fn duplicate_block_identifiers_are_rejected() {
    let fixture = include_str!("../../../fixtures/content/document/valid/two-blocks.json");
    let mut value: serde_json::Value = serde_json::from_str(fixture).expect("the fixture is JSON");
    let blocks = value
        .get_mut("blocks")
        .and_then(serde_json::Value::as_array_mut)
        .expect("blocks are an array");
    let first_id = blocks
        .first()
        .and_then(|block| block.get("block_id"))
        .cloned()
        .expect("the first block has an identifier");
    let second = blocks
        .get_mut(1)
        .and_then(serde_json::Value::as_object_mut)
        .expect("the second block is an object");
    second.insert("block_id".to_owned(), first_id);

    let error = serde_json::from_value::<Document>(value)
        .expect_err("a document revision cannot contain duplicate block identifiers");
    assert!(
        error.to_string().contains("duplicate block identifier"),
        "the rejection names the broken invariant: {error}"
    );
}

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
                block_id: BlockId::parse("018f0000-0000-7000-8000-000000000041")
                    .expect("a block id"),
                level: 1,
                text: "An example".to_owned(),
            },
            DocumentBlock::Paragraph {
                block_id: BlockId::parse("018f0000-0000-7000-8000-000000000042")
                    .expect("a block id"),
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
