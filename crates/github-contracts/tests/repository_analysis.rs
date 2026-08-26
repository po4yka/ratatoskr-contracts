//! Repository-analysis request contract tests.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "assertions in a test binary"
)]

use ratatoskr_github_contracts::{
    AnalysisFailureCode, ReadmeAbsenceReason, ReadmeRevision, RepositoryAnalysisAttributes,
    RepositoryAnalysisCompleted, RepositoryAnalysisContract, RepositoryAnalysisFailed,
    RepositoryAnalysisRequested, RepositoryAnalysisRevision, RepositoryFullName,
};
use ratatoskr_identifiers::{
    ContentDigest, DigestAlgorithm, DigestHex, EntityRef, Extensions, RepositoryAnalysisRequestId,
    RepositoryId, TenantRef, WireTimestamp,
};

/// A request must name the immutable source revision rather than an alias that later metadata
/// refreshes can change.
#[test]
fn requested_payload_carries_an_immutable_source_revision() {
    let payload = RepositoryAnalysisRequested {
        owner: TenantRef::parse("user:018f0000-0000-7000-8000-000000000005")
            .expect("a legal owner"),
        repository_id: RepositoryId::parse("018f0000-0000-7000-8000-000000000601")
            .expect("a legal catalog repository identity"),
        github_repository_numeric_id: 42,
        request_id: RepositoryAnalysisRequestId::parse("018f0000-0000-7000-8000-000000000602")
            .expect("a legal request identity"),
        source_revision: RepositoryAnalysisRevision {
            attributes_digest: digest(),
            readme: ReadmeRevision::Absent {
                reason: ReadmeAbsenceReason::NotFound,
            },
        },
        repository_attributes: RepositoryAnalysisAttributes {
            repository_full_name: RepositoryFullName::parse("owner/repository")
                .expect("a bounded repository alias"),
            description: None,
            primary_language: None,
        },
        requested_contract: RepositoryAnalysisContract::RepositoryAnalysis,
        idempotency_key: digest(),
        extensions: Extensions::new(),
    };

    let wire = serde_json::to_value(payload).expect("the request serializes");
    assert_eq!(
        wire.pointer("/source_revision/attributes_digest/algorithm")
            .and_then(serde_json::Value::as_str),
        Some("sha256"),
        "a request must identify the exact repository-attribute revision"
    );
    assert_eq!(
        wire.pointer("/repository_attributes/repository_full_name")
            .and_then(serde_json::Value::as_str),
        Some("owner/repository"),
        "Knowledge needs the bounded metadata snapshot, not a digest it cannot resolve"
    );
}

/// A terminal completion repeats the full matching identity and exposes only an opaque result
/// reference, so a Catalog projection cannot accidentally resolve a different revision.
#[test]
fn completed_payload_carries_the_request_identity_and_result_reference() {
    let payload = RepositoryAnalysisCompleted {
        owner: owner(),
        repository_id: repository_id(),
        github_repository_numeric_id: 42,
        request_id: request_id(),
        source_revision: revision(),
        analysis_result_ref: EntityRef::parse("analysis:018f0000-0000-7000-8000-000000000603")
            .expect("an opaque analysis reference"),
        completed_at: timestamp("2026-08-26T14:00:00Z"),
        extensions: Extensions::new(),
    };

    let wire = serde_json::to_value(payload).expect("the completion serializes");
    assert_eq!(
        wire.pointer("/analysis_result_ref")
            .and_then(serde_json::Value::as_str),
        Some("analysis:018f0000-0000-7000-8000-000000000603"),
        "the Catalog needs the opaque Knowledge result linkage"
    );
    assert_eq!(
        wire.pointer("/source_revision/attributes_digest/algorithm")
            .and_then(serde_json::Value::as_str),
        Some("sha256"),
        "the completion must echo the immutable input it actually analysed"
    );
}

/// A terminal failure resolves pending state without revealing implementation or provider text.
#[test]
fn failed_payload_carries_a_safe_code_and_retryability() {
    let payload = RepositoryAnalysisFailed {
        owner: owner(),
        repository_id: repository_id(),
        github_repository_numeric_id: 42,
        request_id: request_id(),
        source_revision: revision(),
        failure_code: AnalysisFailureCode::SourceUnavailable,
        retryable: true,
        failed_at: timestamp("2026-08-26T14:00:00Z"),
        extensions: Extensions::new(),
    };

    let wire = serde_json::to_value(payload).expect("the failure serializes");
    assert_eq!(
        wire.pointer("/failure_code")
            .and_then(serde_json::Value::as_str),
        Some("source_unavailable"),
        "a terminal failure needs a machine-safe reason"
    );
    assert_eq!(
        wire.pointer("/retryable")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "a fresh request is only eligible for retry when Knowledge says so"
    );
}

fn digest() -> ContentDigest {
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        hex: DigestHex::parse("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .expect("a legal sha-256 digest"),
    }
}

fn owner() -> TenantRef {
    TenantRef::parse("user:018f0000-0000-7000-8000-000000000005").expect("a legal owner")
}

fn repository_id() -> RepositoryId {
    RepositoryId::parse("018f0000-0000-7000-8000-000000000601")
        .expect("a legal repository identity")
}

fn request_id() -> RepositoryAnalysisRequestId {
    RepositoryAnalysisRequestId::parse("018f0000-0000-7000-8000-000000000602")
        .expect("a legal request identity")
}

fn revision() -> RepositoryAnalysisRevision {
    RepositoryAnalysisRevision {
        attributes_digest: digest(),
        readme: ReadmeRevision::Absent {
            reason: ReadmeAbsenceReason::NotFound,
        },
    }
}

fn timestamp(raw: &str) -> WireTimestamp {
    raw.parse().expect("a canonical timestamp")
}
