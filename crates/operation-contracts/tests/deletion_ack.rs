//! The deletion acknowledgement (XR-021 CONTRACTS.md R5-11).

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "assertions in a test binary; the index writes into a value the test built"
)]

use ratatoskr_event_envelope::{EventEnvelope, EventPayload};
use ratatoskr_operation_contracts::{
    DeletionAckFamily, DeletionAckOutcome, DeletionAckRefusal, KnowledgeDeletionAcknowledged,
    OperationContractError,
};
use serde_json::json;

fn applied() -> serde_json::Value {
    json!({
        "acknowledged_event_id": "018f0000-0000-7000-8000-000000000a01",
        "family": "ai_archive",
        "owner": "user:018f0000-0000-7000-8000-000000000005",
        "subject": "ai_conversation:018f0000-0000-7000-8000-000000000403",
        "outcome": "applied",
        "acknowledged_at": "2026-10-12T08:00:00Z"
    })
}

fn refused() -> serde_json::Value {
    json!({
        "acknowledged_event_id": "018f0000-0000-7000-8000-000000000a02",
        "family": "social",
        "owner": "user:018f0000-0000-7000-8000-000000000005",
        "subject": "social_source:018f0000-0000-7000-8000-000000000202",
        "outcome": "refused",
        "refusal": "foreign_subject",
        "acknowledged_at": "2026-10-12T08:00:01Z"
    })
}

fn typed(value: serde_json::Value) -> KnowledgeDeletionAcknowledged {
    serde_json::from_value(value).expect("the example decodes")
}

#[test]
fn acknowledgement_round_trips_through_an_event_envelope() {
    assert_eq!(
        KnowledgeDeletionAcknowledged::EVENT_TYPE,
        "knowledge.deletion.acknowledged.v1"
    );
    for example in [applied(), refused()] {
        let payload = typed(example.clone());
        assert_eq!(
            serde_json::to_value(&payload).expect("serializes"),
            example,
            "the example is reproduced exactly"
        );

        let mut envelope: EventEnvelope = serde_json::from_value(json!({
            "event_id": "018f0000-0000-7000-8000-000000000b01",
            "event_type": "knowledge.deletion.acknowledged.v1",
            "occurred_at": "2026-10-12T08:00:02Z",
            "producer": "ratatoskr-knowledge",
            "aggregate_id": payload.subject.to_wire(),
            "correlation_id": "operation:018f0000-0000-7000-8000-000000000b03",
            "causation_id": format!("event:{}", example["acknowledged_event_id"].as_str().unwrap()),
            "tenant_id": example["owner"],
            "schema_version": 1,
            "payload": {}
        }))
        .expect("the envelope parses");
        envelope.set_payload(&payload).expect("an object payload");
        let reread: KnowledgeDeletionAcknowledged = envelope.payload_as().expect("typed read");
        assert_eq!(reread, payload);
    }
}

#[test]
fn enums_use_the_exact_snake_case_words() {
    let words = |family, outcome, refusal| {
        (
            serde_json::to_value(family).unwrap(),
            serde_json::to_value(outcome).unwrap(),
            serde_json::to_value(refusal).unwrap(),
        )
    };
    assert_eq!(
        words(
            DeletionAckFamily::AiArchive,
            DeletionAckOutcome::Applied,
            DeletionAckRefusal::ForeignSubject
        ),
        (
            json!("ai_archive"),
            json!("applied"),
            json!("foreign_subject")
        )
    );
    assert_eq!(
        serde_json::to_value(DeletionAckFamily::Social).unwrap(),
        json!("social")
    );
    assert_eq!(
        serde_json::to_value(DeletionAckOutcome::Refused).unwrap(),
        json!("refused")
    );
    for unknown in ["github", "AiArchive", ""] {
        assert!(serde_json::from_value::<DeletionAckFamily>(json!(unknown)).is_err());
    }
}

#[test]
fn validate_requires_a_refusal_exactly_when_the_outcome_is_refused() {
    let mut without = typed(refused());
    without.refusal = None;
    assert_eq!(
        without.validate(),
        Err(OperationContractError::DeletionAckRefusedWithoutRefusal)
    );

    let mut with = typed(applied());
    with.refusal = Some(DeletionAckRefusal::ForeignSubject);
    assert_eq!(
        with.validate(),
        Err(OperationContractError::DeletionAckAppliedWithRefusal)
    );

    assert_eq!(typed(applied()).validate(), Ok(()));
    assert_eq!(typed(refused()).validate(), Ok(()));
}

#[test]
fn deserialization_enforces_the_outcome_rule() {
    let mut without = refused();
    without.as_object_mut().unwrap().remove("refusal");
    let error = serde_json::from_value::<KnowledgeDeletionAcknowledged>(without)
        .expect_err("refused without a refusal is not a payload");
    assert!(error.to_string().contains("requires a refusal"), "{error}");

    let mut with = applied();
    with["refusal"] = json!("foreign_subject");
    let error = serde_json::from_value::<KnowledgeDeletionAcknowledged>(with)
        .expect_err("applied with a refusal is not a payload");
    assert!(error.to_string().contains("forbids a refusal"), "{error}");
}

#[test]
fn an_unknown_member_lands_in_extensions_and_survives_a_round_trip() {
    let mut future = applied();
    future["retry_hint"] = json!({ "after_seconds": 5 });
    let payload = typed(future.clone());
    assert!(payload.extensions.get("retry_hint").is_some());
    assert_eq!(serde_json::to_value(&payload).unwrap(), future);
}

#[test]
fn a_bad_event_id_or_owner_fails_to_deserialize() {
    for (member, value) in [
        ("acknowledged_event_id", json!("not-a-uuid")),
        (
            "acknowledged_event_id",
            json!("018F0000-0000-7000-8000-000000000A01"),
        ),
        ("owner", json!("018f0000-0000-7000-8000-000000000005")),
        (
            "owner",
            json!("organisation:018f0000-0000-7000-8000-000000000005"),
        ),
        ("owner", json!("user:not-a-uuid")),
        ("subject", json!("no-colon")),
    ] {
        let mut example = applied();
        example[member] = value.clone();
        assert!(
            serde_json::from_value::<KnowledgeDeletionAcknowledged>(example).is_err(),
            "{member} = {value} must be refused"
        );
    }
}
