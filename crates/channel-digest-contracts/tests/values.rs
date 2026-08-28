//! Public value-contract behavior.
#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "tests use panic messages to preserve validation context"
)]

use ratatoskr_channel_digest_contracts::{
    ChannelDigestIdempotencyKey, ChannelDigestManifestRef, ChannelDigestResultId,
    ChannelDigestResultRef, ChannelDigestRunId, ChannelDigestRunTrigger,
    ChannelDigestSubscriptionId, ChannelUsername, DigestChannelCount, DigestOccurrenceRef,
    DigestScheduleRef, DigestSourceCount, DigestWindow, KnowledgeAnalysisRef, OutputLanguage,
    SubscriptionDesiredState,
};
use ratatoskr_identifiers::{TenantRef, WireTimestamp};
use schemars::schema_for;

#[test]
fn canonical_channel_username_round_trips() {
    let username = ChannelUsername::parse("example_channel")
        .unwrap_or_else(|error| panic!("canonical username must parse: {error}"));

    assert_eq!(username.as_str(), "example_channel");
}

#[test]
fn channel_username_wire_round_trip_is_canonical() {
    let username = ChannelUsername::parse("example_channel")
        .unwrap_or_else(|error| panic!("canonical username must parse: {error}"));
    let wire = serde_json::to_string(&username)
        .unwrap_or_else(|error| panic!("username must serialize: {error}"));

    assert_eq!(wire, "\"example_channel\"");
    assert_eq!(
        serde_json::from_str::<ChannelUsername>(&wire)
            .unwrap_or_else(|error| panic!("username must deserialize: {error}")),
        username
    );
}

#[test]
fn channel_username_schema_rejects_noncanonical_spellings() {
    let schema = serde_json::to_value(schema_for!(ChannelUsername))
        .unwrap_or_else(|error| panic!("schema must serialize: {error}"));
    let validator = jsonschema::validator_for(&schema)
        .unwrap_or_else(|error| panic!("schema must compile: {error}"));

    for invalid in [
        "Example_Channel",
        "four",
        "1example",
        "example-channel",
        "abcdefghijklmnopqrstuvwxyzabcdefg",
    ] {
        assert!(
            !validator.is_valid(&serde_json::json!(invalid)),
            "schema accepted noncanonical username {invalid:?}"
        );
    }
}

#[test]
fn digest_record_identities_use_distinct_canonical_uuid_types() {
    const RAW: &str = "018f0000-0000-7000-8000-000000000001";

    let subscription = ChannelDigestSubscriptionId::parse(RAW)
        .unwrap_or_else(|error| panic!("subscription id must parse: {error}"));
    let run =
        ChannelDigestRunId::parse(RAW).unwrap_or_else(|error| panic!("run id must parse: {error}"));
    let result = ChannelDigestResultId::parse(RAW)
        .unwrap_or_else(|error| panic!("result id must parse: {error}"));

    assert_eq!(subscription.to_string(), RAW);
    assert_eq!(run.to_string(), RAW);
    assert_eq!(result.to_string(), RAW);
}

#[test]
fn digest_record_identity_wire_round_trip_preserves_canonical_spelling() {
    const RAW: &str = "018f0000-0000-7000-8000-000000000001";
    let run =
        ChannelDigestRunId::parse(RAW).unwrap_or_else(|error| panic!("run id must parse: {error}"));
    let wire = serde_json::to_string(&run)
        .unwrap_or_else(|error| panic!("run id must serialize: {error}"));

    assert_eq!(wire, format!("\"{RAW}\""));
    assert_eq!(
        serde_json::from_str::<ChannelDigestRunId>(&wire)
            .unwrap_or_else(|error| panic!("run id must deserialize: {error}")),
        run
    );
}

#[test]
fn digest_window_accepts_a_bounded_closed_open_range() {
    let start = WireTimestamp::parse("2026-08-20T10:00:00Z")
        .unwrap_or_else(|error| panic!("start must parse: {error}"));
    let end = WireTimestamp::parse("2026-08-21T10:00:00Z")
        .unwrap_or_else(|error| panic!("end must parse: {error}"));

    let window = DigestWindow::new(start, end)
        .unwrap_or_else(|error| panic!("one-day window must be valid: {error}"));

    assert_eq!(window.start_at, start);
    assert_eq!(window.end_at, end);
}

#[test]
fn bounded_references_and_vocabularies_round_trip() {
    let owner =
        TenantRef::parse("user:018f0000-0000-7000-8000-000000000000").expect("owner reference");
    let values = [
        serde_json::to_value(
            ChannelDigestIdempotencyKey::parse("digest.018f0000-0000-7000-8000-000000000001")
                .expect("idempotency key"),
        )
        .expect("serialize key"),
        serde_json::to_value(
            ChannelDigestManifestRef::parse(
                "channel-digest-manifest:018f0000-0000-7000-8000-000000000002",
            )
            .expect("manifest ref"),
        )
        .expect("serialize manifest ref"),
        serde_json::to_value(
            ChannelDigestResultRef::parse(
                "channel-digest-result:018f0000-0000-7000-8000-000000000003",
            )
            .expect("result ref"),
        )
        .expect("serialize result ref"),
        serde_json::to_value(
            KnowledgeAnalysisRef::parse("analysis:018f0000-0000-7000-8000-000000000004")
                .expect("analysis ref"),
        )
        .expect("serialize analysis ref"),
        serde_json::to_value(
            DigestScheduleRef::parse("schedule:018f0000-0000-7000-8000-000000000005")
                .expect("schedule ref"),
        )
        .expect("serialize schedule ref"),
        serde_json::to_value(
            DigestOccurrenceRef::parse("schedule-occurrence:018f0000-0000-7000-8000-000000000006")
                .expect("occurrence ref"),
        )
        .expect("serialize occurrence ref"),
    ];

    assert_eq!(
        values[0],
        serde_json::json!("digest.018f0000-0000-7000-8000-000000000001")
    );
    assert_eq!(
        values[1],
        serde_json::json!("channel-digest-manifest:018f0000-0000-7000-8000-000000000002")
    );
    assert_eq!(
        values[2],
        serde_json::json!("channel-digest-result:018f0000-0000-7000-8000-000000000003")
    );
    assert_eq!(
        values[3],
        serde_json::json!("analysis:018f0000-0000-7000-8000-000000000004")
    );
    assert_eq!(
        values[4],
        serde_json::json!("schedule:018f0000-0000-7000-8000-000000000005")
    );
    assert_eq!(
        values[5],
        serde_json::json!("schedule-occurrence:018f0000-0000-7000-8000-000000000006")
    );
    assert_eq!(
        owner.to_string(),
        "user:018f0000-0000-7000-8000-000000000000"
    );

    assert_eq!(
        serde_json::to_value(OutputLanguage::Ru).expect("language"),
        serde_json::json!("ru")
    );
    assert_eq!(
        serde_json::to_value(OutputLanguage::En).expect("language"),
        serde_json::json!("en")
    );
    assert_eq!(
        serde_json::to_value(SubscriptionDesiredState::Active).expect("state"),
        serde_json::json!("active")
    );
    assert_eq!(
        serde_json::to_value(SubscriptionDesiredState::Inactive).expect("state"),
        serde_json::json!("inactive")
    );
}

#[test]
fn run_trigger_vocabulary_preserves_authority_specific_fields() {
    let instant = WireTimestamp::parse("2026-08-21T10:00:00Z").expect("instant");
    let on_demand = ChannelDigestRunTrigger::OnDemand {
        accepted_at: instant,
    };
    let scheduled = ChannelDigestRunTrigger::Scheduled {
        schedule_ref: DigestScheduleRef::parse("schedule:018f0000-0000-7000-8000-000000000005")
            .expect("schedule ref"),
        occurrence_ref: DigestOccurrenceRef::parse(
            "schedule-occurrence:018f0000-0000-7000-8000-000000000006",
        )
        .expect("occurrence ref"),
        due_at: instant,
    };

    assert_eq!(
        serde_json::to_value(on_demand)
            .expect("trigger")
            .get("kind"),
        Some(&serde_json::json!("on_demand"))
    );
    assert_eq!(
        serde_json::to_value(scheduled)
            .expect("trigger")
            .get("kind"),
        Some(&serde_json::json!("scheduled"))
    );
}

#[test]
fn invalid_values_are_rejected_without_normalization_or_unbounded_inputs() {
    for invalid in ["Example_Channel", "abcd", "1channel", "channel-name"] {
        assert!(
            ChannelUsername::parse(invalid).is_err(),
            "accepted {invalid:?}"
        );
    }
    assert!(ChannelDigestIdempotencyKey::parse(&"x".repeat(129)).is_err());
    assert!(ChannelDigestManifestRef::parse("https://storage.invalid/manifest").is_err());
    assert!(ChannelDigestResultRef::parse("channel-digest-result:123").is_err());
    assert!(KnowledgeAnalysisRef::parse("analysis:123").is_err());
    assert!(DigestScheduleRef::parse("schedule:123").is_err());
    assert!(DigestOccurrenceRef::parse("occurrence:018f0000-0000-7000-8000-000000000006").is_err());

    for invalid in [0, 101] {
        assert!(
            DigestSourceCount::new(invalid).is_err(),
            "accepted source count {invalid}"
        );
    }
    for invalid in [0, 21] {
        assert!(
            DigestChannelCount::new(invalid).is_err(),
            "accepted channel count {invalid}"
        );
    }
    assert!(serde_json::from_value::<OutputLanguage>(serde_json::json!("de")).is_err());
}

#[test]
fn digest_window_deserialization_enforces_order_and_seven_day_bound() {
    let invalid = [
        serde_json::json!({
            "start_at": "2026-08-20T10:00:00Z",
            "end_at": "2026-08-20T10:00:00Z"
        }),
        serde_json::json!({
            "start_at": "2026-08-21T10:00:00Z",
            "end_at": "2026-08-20T10:00:00Z"
        }),
        serde_json::json!({
            "start_at": "2026-08-20T10:00:00Z",
            "end_at": "2026-08-28T10:00:00Z"
        }),
    ];

    for candidate in invalid {
        assert!(
            serde_json::from_value::<DigestWindow>(candidate.clone()).is_err(),
            "accepted invalid window {candidate}"
        );
    }
}

#[test]
fn scheduled_trigger_requires_exact_schedule_and_occurrence_shape() {
    let due_at = "2026-08-21T10:00:00Z";
    let invalid = [
        serde_json::json!({"kind": "scheduled", "due_at": due_at}),
        serde_json::json!({
            "kind": "on_demand",
            "accepted_at": due_at,
            "occurrence_ref": "schedule-occurrence:018f0000-0000-7000-8000-000000000006"
        }),
        serde_json::json!({
            "kind": "scheduled",
            "schedule_ref": "schedule:018f0000-0000-7000-8000-000000000005",
            "occurrence_ref": "occurrence:018f0000-0000-7000-8000-000000000006",
            "due_at": due_at
        }),
    ];

    for candidate in invalid {
        assert!(
            serde_json::from_value::<ChannelDigestRunTrigger>(candidate.clone()).is_err(),
            "accepted invalid trigger {candidate}"
        );
    }
}

#[test]
fn count_schemas_publish_the_same_bounds_as_constructors() {
    let source_schema =
        serde_json::to_value(schema_for!(DigestSourceCount)).expect("source schema");
    let source = jsonschema::validator_for(&source_schema).expect("source schema compiles");
    assert!(source.is_valid(&serde_json::json!(1)));
    assert!(source.is_valid(&serde_json::json!(100)));
    assert!(!source.is_valid(&serde_json::json!(0)));
    assert!(!source.is_valid(&serde_json::json!(101)));

    let channel_schema =
        serde_json::to_value(schema_for!(DigestChannelCount)).expect("channel schema");
    let channel = jsonschema::validator_for(&channel_schema).expect("channel schema compiles");
    assert!(channel.is_valid(&serde_json::json!(1)));
    assert!(channel.is_valid(&serde_json::json!(20)));
    assert!(!channel.is_valid(&serde_json::json!(0)));
    assert!(!channel.is_valid(&serde_json::json!(21)));
}
