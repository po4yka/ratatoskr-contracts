//! Knowledge recap request and terminal-fact behavior.
#![allow(clippy::expect_used, clippy::panic, reason = "test diagnostics")]

use ratatoskr_channel_digest_contracts::{
    KnowledgeChannelDigestRecapCompleted, KnowledgeChannelDigestRecapFailed,
    KnowledgeChannelDigestRecapRequested,
};
use ratatoskr_event_envelope::{CommandPayload, EventPayload};

fn manifest_digest() -> serde_json::Value {
    serde_json::json!({
        "algorithm": "sha256",
        "hex": "0000000000000000000000000000000000000000000000000000000000000000"
    })
}

fn request_json() -> serde_json::Value {
    serde_json::json!({
        "operation_id": "018f0000-0000-7000-8000-000000000201",
        "owner": "user:018f0000-0000-7000-8000-000000000202",
        "digest_run_id": "018f0000-0000-7000-8000-000000000203",
        "window": {
            "start_at": "2026-08-20T10:00:00Z",
            "end_at": "2026-08-21T10:00:00Z"
        },
        "output_language": "ru",
        "source_count": 12,
        "channel_count": 3,
        "manifest_ref": "channel-digest-manifest:018f0000-0000-7000-8000-000000000204",
        "manifest_digest": manifest_digest(),
        "analysis_family": "channel_digest_recap",
        "analysis_contract": "channel_digest_recap.v1"
    })
}

fn insert_field(value: &mut serde_json::Value, field: &str, replacement: serde_json::Value) {
    value
        .as_object_mut()
        .expect("test fixture is an object")
        .insert(field.to_owned(), replacement);
}

#[test]
fn request_names_immutable_manifest_without_source_content() {
    let raw = request_json();
    let request: KnowledgeChannelDigestRecapRequested =
        serde_json::from_value(raw.clone()).expect("valid recap request");
    assert_eq!(
        KnowledgeChannelDigestRecapRequested::COMMAND_TYPE,
        "knowledge.channel_digest_recap.requested.v1"
    );
    assert_eq!(serde_json::to_value(&request).expect("serialize"), raw);
    request
        .validate_for_publish()
        .expect("canonical producer request");

    let members = serde_json::to_value(request).expect("serialize");
    for forbidden in [
        "posts",
        "text",
        "channel_username",
        "display_title",
        "url",
        "provider_id",
        "session",
        "credential",
        "prompt",
        "model",
        "raw_error",
    ] {
        assert!(
            members.get(forbidden).is_none(),
            "published forbidden member {forbidden}"
        );
    }
}

#[test]
fn request_rejects_inconsistent_counts_and_fixed_contract_drift() {
    for (field, value) in [
        ("source_count", serde_json::json!(0)),
        ("source_count", serde_json::json!(101)),
        ("channel_count", serde_json::json!(0)),
        ("channel_count", serde_json::json!(21)),
        ("channel_count", serde_json::json!(13)),
        ("analysis_family", serde_json::json!("generic_summary")),
        (
            "analysis_contract",
            serde_json::json!("channel_digest_recap.v2"),
        ),
    ] {
        let mut raw = request_json();
        insert_field(&mut raw, field, value);
        assert!(
            serde_json::from_value::<KnowledgeChannelDigestRecapRequested>(raw).is_err(),
            "accepted invalid {field}"
        );
    }

    let mut extended = request_json();
    insert_field(
        &mut extended,
        "posts",
        serde_json::json!([{"text": "private"}]),
    );
    let preserved: KnowledgeChannelDigestRecapRequested =
        serde_json::from_value(extended).expect("consumer preserves additive member");
    assert!(preserved.validate_for_publish().is_err());
}

#[test]
fn completion_links_one_verified_result_and_exact_coverage() {
    let raw = serde_json::json!({
        "owner": "user:018f0000-0000-7000-8000-000000000202",
        "operation_id": "018f0000-0000-7000-8000-000000000201",
        "digest_run_id": "018f0000-0000-7000-8000-000000000203",
        "manifest_digest": manifest_digest(),
        "analysis_ref": "analysis:018f0000-0000-7000-8000-000000000205",
        "digest_result_id": "018f0000-0000-7000-8000-000000000206",
        "result_ref": "channel-digest-result:018f0000-0000-7000-8000-000000000206",
        "result_digest": {
            "algorithm": "sha256",
            "hex": "1111111111111111111111111111111111111111111111111111111111111111"
        },
        "completed_at": "2026-08-21T10:01:00Z",
        "coverage": {
            "selected_count": 12,
            "included_count": 10,
            "omitted_count": 2,
            "channel_count": 3
        }
    });
    let completed: KnowledgeChannelDigestRecapCompleted =
        serde_json::from_value(raw.clone()).expect("valid completion");
    assert_eq!(
        KnowledgeChannelDigestRecapCompleted::EVENT_TYPE,
        "knowledge.channel_digest_recap.completed.v1"
    );
    assert_eq!(serde_json::to_value(&completed).expect("serialize"), raw);
    completed
        .validate_for_publish()
        .expect("canonical completion");
}

#[test]
fn completion_rejects_inconsistent_or_empty_coverage() {
    let cases = [
        (10, 8, 1, 3),
        (10, 0, 10, 3),
        (10, 10, 0, 0),
        (101, 100, 1, 3),
        (10, 10, 0, 21),
    ];
    for (selected, included, omitted, channels) in cases {
        let raw = serde_json::json!({
            "owner": "user:018f0000-0000-7000-8000-000000000202",
            "operation_id": "018f0000-0000-7000-8000-000000000201",
            "digest_run_id": "018f0000-0000-7000-8000-000000000203",
            "manifest_digest": manifest_digest(),
            "analysis_ref": "analysis:018f0000-0000-7000-8000-000000000205",
            "digest_result_id": "018f0000-0000-7000-8000-000000000206",
            "result_ref": "channel-digest-result:018f0000-0000-7000-8000-000000000206",
            "result_digest": {"algorithm": "sha256", "hex": "1111111111111111111111111111111111111111111111111111111111111111"},
            "completed_at": "2026-08-21T10:01:00Z",
            "coverage": {
                "selected_count": selected,
                "included_count": included,
                "omitted_count": omitted,
                "channel_count": channels
            }
        });
        assert!(
            serde_json::from_value::<KnowledgeChannelDigestRecapCompleted>(raw).is_err(),
            "accepted inconsistent coverage {selected}/{included}/{omitted}/{channels}"
        );
    }
}

#[test]
fn failure_is_closed_content_free_and_has_no_success_reference() {
    for code in [
        "manifest_unavailable",
        "manifest_integrity",
        "unsupported_language",
        "context_budget",
        "provider_unavailable",
        "provider_timeout",
        "invalid_output",
        "cost_budget",
        "cancelled",
    ] {
        let raw = serde_json::json!({
            "owner": "user:018f0000-0000-7000-8000-000000000202",
            "operation_id": "018f0000-0000-7000-8000-000000000201",
            "digest_run_id": "018f0000-0000-7000-8000-000000000203",
            "manifest_digest": manifest_digest(),
            "failure_code": code,
            "failed_at": "2026-08-21T10:01:00Z"
        });
        let failed: KnowledgeChannelDigestRecapFailed =
            serde_json::from_value(raw.clone()).expect("valid safe failure");
        assert_eq!(
            KnowledgeChannelDigestRecapFailed::EVENT_TYPE,
            "knowledge.channel_digest_recap.failed.v1"
        );
        assert_eq!(serde_json::to_value(failed).expect("serialize"), raw);
    }

    let mut unsafe_failure = serde_json::json!({
        "owner": "user:018f0000-0000-7000-8000-000000000202",
        "operation_id": "018f0000-0000-7000-8000-000000000201",
        "digest_run_id": "018f0000-0000-7000-8000-000000000203",
        "manifest_digest": manifest_digest(),
        "failure_code": "provider_unavailable",
        "failed_at": "2026-08-21T10:01:00Z"
    });
    insert_field(
        &mut unsafe_failure,
        "provider_error",
        serde_json::json!("Authorization: secret; source text"),
    );
    insert_field(
        &mut unsafe_failure,
        "result_ref",
        serde_json::json!("channel-digest-result:018f0000-0000-7000-8000-000000000206"),
    );
    let preserved: KnowledgeChannelDigestRecapFailed =
        serde_json::from_value(unsafe_failure).expect("consumer preserves additive members");
    assert!(preserved.validate_for_publish().is_err());

    let mut unknown_code = serde_json::json!({
        "owner": "user:018f0000-0000-7000-8000-000000000202",
        "operation_id": "018f0000-0000-7000-8000-000000000201",
        "digest_run_id": "018f0000-0000-7000-8000-000000000203",
        "manifest_digest": manifest_digest(),
        "failure_code": "provider_dump",
        "failed_at": "2026-08-21T10:01:00Z"
    });
    assert!(
        serde_json::from_value::<KnowledgeChannelDigestRecapFailed>(unknown_code.take()).is_err()
    );
}
