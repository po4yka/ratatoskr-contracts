//! Subscription and run command payload behavior.
#![allow(clippy::expect_used, clippy::panic, reason = "test diagnostics")]

use ratatoskr_channel_digest_contracts::{
    ChannelDigestRunRequested, ChannelDigestSubscriptionSetRequested,
};
use ratatoskr_event_envelope::CommandPayload;

#[test]
fn subscription_command_is_owner_scoped_replay_safe_and_content_free() {
    let raw = serde_json::json!({
        "operation_id": "018f0000-0000-7000-8000-000000000101",
        "owner": "user:018f0000-0000-7000-8000-000000000102",
        "idempotency_key": "telegram.subscribe.018f0000-0000-7000-8000-000000000103",
        "channel_username": "example_channel",
        "desired_state": "active"
    });

    let payload: ChannelDigestSubscriptionSetRequested =
        serde_json::from_value(raw.clone()).expect("valid subscription command");
    assert_eq!(
        ChannelDigestSubscriptionSetRequested::COMMAND_TYPE,
        "channel_digest.subscription.set_requested.v1"
    );
    assert_eq!(serde_json::to_value(&payload).expect("serialize"), raw);
    payload
        .validate_for_publish()
        .expect("canonical producer payload");

    let members = serde_json::to_value(payload).expect("serialize");
    for forbidden in [
        "telegram_chat_id",
        "telegram_user_id",
        "provider_id",
        "session",
        "invite_hash",
        "credential",
        "display_title",
        "post_content",
        "current_state",
    ] {
        assert!(
            members.get(forbidden).is_none(),
            "published forbidden member {forbidden}"
        );
    }
}

#[test]
fn subscription_producer_cannot_author_untyped_business_state() {
    let payload: ChannelDigestSubscriptionSetRequested =
        serde_json::from_value(serde_json::json!({
            "operation_id": "018f0000-0000-7000-8000-000000000101",
            "owner": "user:018f0000-0000-7000-8000-000000000102",
            "idempotency_key": "telegram.subscribe.018f0000-0000-7000-8000-000000000103",
            "channel_username": "example_channel",
            "desired_state": "active",
            "claimed_current_state": "inactive"
        }))
        .expect("consumer preserves additive member");

    assert!(payload.validate_for_publish().is_err());
}

#[test]
fn run_command_encodes_exact_trigger_and_window_semantics() {
    for trigger in [
        serde_json::json!({
            "kind": "on_demand",
            "accepted_at": "2026-08-21T10:00:00Z"
        }),
        serde_json::json!({
            "kind": "scheduled",
            "schedule_ref": "schedule:018f0000-0000-7000-8000-000000000104",
            "occurrence_ref": "schedule-occurrence:018f0000-0000-7000-8000-000000000105",
            "due_at": "2026-08-21T10:00:00Z"
        }),
    ] {
        let raw = serde_json::json!({
            "operation_id": "018f0000-0000-7000-8000-000000000101",
            "owner": "user:018f0000-0000-7000-8000-000000000102",
            "digest_run_id": "018f0000-0000-7000-8000-000000000106",
            "idempotency_key": "telegram.digest.018f0000-0000-7000-8000-000000000107",
            "window": {
                "start_at": "2026-08-20T10:00:00Z",
                "end_at": "2026-08-21T10:00:00Z"
            },
            "output_language": "ru",
            "trigger": trigger
        });
        let payload: ChannelDigestRunRequested =
            serde_json::from_value(raw.clone()).expect("valid run command");
        assert_eq!(
            ChannelDigestRunRequested::COMMAND_TYPE,
            "channel_digest.run.requested.v1"
        );
        assert_eq!(serde_json::to_value(&payload).expect("serialize"), raw);
        payload
            .validate_for_publish()
            .expect("canonical producer payload");
    }
}

#[test]
fn run_command_rejects_trigger_window_contradictions_and_producer_extensions() {
    let base = serde_json::json!({
        "operation_id": "018f0000-0000-7000-8000-000000000101",
        "owner": "user:018f0000-0000-7000-8000-000000000102",
        "digest_run_id": "018f0000-0000-7000-8000-000000000106",
        "idempotency_key": "telegram.digest.018f0000-0000-7000-8000-000000000107",
        "window": {
            "start_at": "2026-08-20T10:00:00Z",
            "end_at": "2026-08-21T10:00:00Z"
        },
        "output_language": "ru",
        "trigger": {
            "kind": "scheduled",
            "schedule_ref": "schedule:018f0000-0000-7000-8000-000000000104",
            "occurrence_ref": "schedule-occurrence:018f0000-0000-7000-8000-000000000105",
            "due_at": "2026-08-21T09:59:59Z"
        }
    });
    assert!(serde_json::from_value::<ChannelDigestRunRequested>(base).is_err());

    let with_selector = serde_json::json!({
        "operation_id": "018f0000-0000-7000-8000-000000000101",
        "owner": "user:018f0000-0000-7000-8000-000000000102",
        "digest_run_id": "018f0000-0000-7000-8000-000000000106",
        "idempotency_key": "telegram.digest.018f0000-0000-7000-8000-000000000107",
        "window": {
            "start_at": "2026-08-20T10:00:00Z",
            "end_at": "2026-08-21T10:00:00Z"
        },
        "output_language": "ru",
        "trigger": {"kind": "on_demand", "accepted_at": "2026-08-21T10:00:00Z"},
        "model": "provider/model",
        "tenant_selector": "all"
    });
    let preserved: ChannelDigestRunRequested =
        serde_json::from_value(with_selector).expect("consumer preserves additive members");
    assert!(preserved.validate_for_publish().is_err());
}
