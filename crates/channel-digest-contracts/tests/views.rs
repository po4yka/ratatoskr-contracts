//! The owner read-API views (XR-021 CONTRACTS.md S08).

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes run on values the test itself decoded"
)]

use ratatoskr_channel_digest_contracts::{
    ChannelDigestFailureClass, ChannelDigestOutcome, ChannelDigestResultPage,
    ChannelDigestResultSummary, ChannelDigestResultView, ChannelDigestSubscriptionPage,
};
use serde_json::json;

const RESULT: &str = "018f0000-0000-7000-8000-000000000d01";
const RUN: &str = "018f0000-0000-7000-8000-000000000a02";
const RECAP: &str = "018f0000-0000-7000-8000-000000000e01";
const DIGEST: &str = "4c8b8766da59c453385cb50e5677bde62fcc7994f89418e79bb6d153e5438089";

/// The two shapes `GET /v1/results/{result_id}` answers today, taken from the digest service's
/// interface description.
#[test]
fn result_view_round_trips_todays_completed_and_failed_json() {
    let completed = json!({
        "result_id": RESULT,
        "run_id": RUN,
        "outcome": "completed",
        "recap_id": RECAP,
        "citation_count": 2,
        "result_digest": { "algorithm": "sha256", "hex": DIGEST },
        "recap": { "title": "Morning digest", "summary": "Quiet night.", "citations": [] }
    });
    let view: ChannelDigestResultView =
        serde_json::from_value(completed.clone()).expect("the completed shape decodes");
    assert_eq!(view.outcome, ChannelDigestOutcome::Completed);
    assert_eq!(
        view.recap_id.map(|id| id.to_string()).as_deref(),
        Some(RECAP)
    );
    assert_eq!(view.citation_count, Some(2));
    assert_eq!(
        view.result_digest
            .as_ref()
            .map(|digest| digest.hex.as_str()),
        Some(DIGEST)
    );
    assert!(view.recap.is_some());
    assert!(view.safe_failure_class.is_none());
    assert_eq!(
        serde_json::to_value(&view).expect("serializes"),
        completed,
        "the completed shape is reproduced exactly"
    );

    let failed = json!({
        "result_id": RESULT,
        "run_id": RUN,
        "outcome": "failed",
        "safe_failure_class": "provider_unavailable"
    });
    let view: ChannelDigestResultView =
        serde_json::from_value(failed.clone()).expect("the failed shape decodes");
    assert_eq!(view.outcome, ChannelDigestOutcome::Failed);
    assert!(view.recap_id.is_none() && view.citation_count.is_none());
    assert!(view.result_digest.is_none() && view.recap.is_none());
    assert_eq!(
        view.safe_failure_class
            .as_ref()
            .map(ChannelDigestFailureClass::as_str),
        Some("provider_unavailable")
    );
    assert_eq!(
        serde_json::to_value(&view).expect("serializes"),
        failed,
        "absent optionals are skipped, not serialized as null"
    );

    let partial = json!({
        "result_id": RESULT,
        "run_id": RUN,
        "outcome": "partial",
        "recap_id": RECAP,
        "citation_count": 1,
        "result_digest": { "algorithm": "sha256", "hex": DIGEST },
        "recap": {}
    });
    let view: ChannelDigestResultView =
        serde_json::from_value(partial).expect("the partial shape decodes");
    assert_eq!(view.outcome, ChannelDigestOutcome::Partial);
}

#[test]
fn result_summary_rejects_unknown_outcome() {
    let summary = |outcome: &str| {
        json!({
            "result_id": RESULT,
            "run_id": RUN,
            "outcome": outcome,
            "created_at": "2026-08-27T06:00:00Z"
        })
    };
    let decoded: ChannelDigestResultSummary =
        serde_json::from_value(summary("partial")).expect("a documented outcome decodes");
    assert_eq!(decoded.outcome, ChannelDigestOutcome::Partial);
    assert_eq!(
        serde_json::to_value(&decoded).expect("serializes"),
        summary("partial")
    );

    for unknown in ["cancelled", "Completed", "COMPLETED", ""] {
        let error = serde_json::from_value::<ChannelDigestResultSummary>(summary(unknown))
            .expect_err("an unknown outcome must stop processing");
        assert!(error.to_string().contains("unknown variant"), "{error}");
        assert!(
            serde_json::from_value::<ChannelDigestResultView>(json!({
                "result_id": RESULT,
                "run_id": RUN,
                "outcome": unknown
            }))
            .is_err(),
            "the full view must refuse {unknown:?} too"
        );
    }

    let missing_instant = json!({ "result_id": RESULT, "run_id": RUN, "outcome": "failed" });
    assert!(serde_json::from_value::<ChannelDigestResultSummary>(missing_instant).is_err());
}

#[test]
fn subscription_page_round_trips() {
    let page = json!({
        "subscriptions": [
            {
                "subscription_id": "018f0000-0000-7000-8000-000000000f01",
                "channel_username": "rust_weekly",
                "enabled": true
            },
            {
                "subscription_id": "018f0000-0000-7000-8000-000000000f02",
                "channel_username": "daily_news_ru",
                "enabled": false
            }
        ]
    });
    let decoded: ChannelDigestSubscriptionPage =
        serde_json::from_value(page.clone()).expect("the page decodes");
    assert_eq!(decoded.subscriptions.len(), 2);
    assert_eq!(
        decoded.subscriptions[0].channel_username.as_str(),
        "rust_weekly"
    );
    assert!(decoded.subscriptions[0].enabled && !decoded.subscriptions[1].enabled);
    assert_eq!(serde_json::to_value(&decoded).expect("serializes"), page);

    let empty = json!({ "subscriptions": [] });
    let decoded: ChannelDigestSubscriptionPage =
        serde_json::from_value(empty.clone()).expect("an empty page decodes");
    assert_eq!(serde_json::to_value(&decoded).expect("serializes"), empty);

    let mixed_case = json!({
        "subscriptions": [{
            "subscription_id": "018f0000-0000-7000-8000-000000000f01",
            "channel_username": "Rust_Weekly",
            "enabled": true
        }]
    });
    assert!(
        serde_json::from_value::<ChannelDigestSubscriptionPage>(mixed_case).is_err(),
        "usernames stay canonical lowercase"
    );
}

#[test]
fn result_page_round_trips() {
    let page = json!({
        "results": [
            {
                "result_id": RESULT,
                "run_id": RUN,
                "outcome": "completed",
                "created_at": "2026-08-27T06:00:00Z"
            },
            {
                "result_id": "018f0000-0000-7000-8000-000000000d02",
                "run_id": "018f0000-0000-7000-8000-000000000a03",
                "outcome": "failed",
                "safe_failure_class": "manifest_invalid",
                "created_at": "2026-08-26T06:00:00Z"
            }
        ]
    });
    let decoded: ChannelDigestResultPage =
        serde_json::from_value(page.clone()).expect("the page decodes");
    assert_eq!(decoded.results.len(), 2);
    assert_eq!(serde_json::to_value(&decoded).expect("serializes"), page);
}
