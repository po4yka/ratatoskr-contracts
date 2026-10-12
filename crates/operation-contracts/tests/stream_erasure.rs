//! The stream erasure command (XR-021 CONTRACTS.md R5-12).

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "assertions in a test binary; the index writes into a value the test built"
)]

use ratatoskr_event_envelope::{CommandEnvelope, CommandPayload};
use ratatoskr_operation_contracts::{
    MAX_JSON_SAFE_SEQUENCE, MAX_STREAM_SEQUENCES, OperationContractError,
    PlatformStreamErasureRequested,
};
use serde_json::json;

fn request(sequences: &[u64]) -> serde_json::Value {
    json!({
        "request_ref": "privacy_request:018f0000-0000-7000-8000-000000000c01",
        "owner": "user:018f0000-0000-7000-8000-000000000005",
        "stream_sequences": sequences
    })
}

fn typed(sequences: &[u64]) -> PlatformStreamErasureRequested {
    serde_json::from_value(request(sequences)).expect("the example decodes")
}

fn with_sequences(sequences: Vec<u64>) -> PlatformStreamErasureRequested {
    PlatformStreamErasureRequested {
        stream_sequences: sequences,
        ..typed(&[1])
    }
}

#[test]
fn request_round_trips_through_a_command_envelope() {
    assert_eq!(
        PlatformStreamErasureRequested::COMMAND_TYPE,
        "platform.stream_erasure.requested.v1"
    );
    let example = request(&[7, 9, 4_000_000_000]);
    let payload: PlatformStreamErasureRequested =
        serde_json::from_value(example.clone()).expect("decodes");
    assert_eq!(serde_json::to_value(&payload).unwrap(), example);

    let mut envelope: CommandEnvelope = serde_json::from_value(json!({
        "command_id": "018f0000-0000-7000-8000-000000000d01",
        "command_type": "platform.stream_erasure.requested.v1",
        "issued_at": "2026-10-12T08:00:00Z",
        "producer": "ratatoskr-chatgpt",
        "aggregate_id": "privacy_request:018f0000-0000-7000-8000-000000000c01",
        "correlation_id": "operation:018f0000-0000-7000-8000-000000000d03",
        "tenant_id": "user:018f0000-0000-7000-8000-000000000005",
        "schema_version": 1,
        "payload": {}
    }))
    .expect("the envelope parses");
    envelope.set_payload(&payload).expect("an object payload");
    let reread: PlatformStreamErasureRequested = envelope.payload_as().expect("typed read");
    assert_eq!(reread, payload);
}

#[test]
fn validate_accepts_exactly_one_and_exactly_five_hundred_ascending_sequences() {
    assert_eq!(MAX_STREAM_SEQUENCES, 500);
    assert_eq!(with_sequences(vec![1]).validate(), Ok(()));
    assert_eq!(with_sequences((1..=500).collect()).validate(), Ok(()));
    assert_eq!(
        with_sequences(vec![MAX_JSON_SAFE_SEQUENCE]).validate(),
        Ok(())
    );
}

#[test]
fn validate_rejects_an_empty_list_and_five_hundred_and_one_entries() {
    assert_eq!(
        with_sequences(vec![]).validate(),
        Err(OperationContractError::EmptyStreamSequences)
    );
    assert_eq!(
        with_sequences((1..=501).collect()).validate(),
        Err(OperationContractError::TooManyStreamSequences { max: 500, got: 501 })
    );
}

#[test]
fn validate_rejects_a_zero_a_descending_list_and_a_duplicate() {
    assert_eq!(
        with_sequences(vec![0, 5]).validate(),
        Err(OperationContractError::ZeroStreamSequence { index: 0 })
    );
    assert_eq!(
        with_sequences(vec![3, 2]).validate(),
        Err(OperationContractError::StreamSequencesNotAscending {
            index: 1,
            previous: 3,
            next: 2
        })
    );
    assert_eq!(
        with_sequences(vec![2, 5, 5]).validate(),
        Err(OperationContractError::StreamSequencesNotAscending {
            index: 2,
            previous: 5,
            next: 5
        })
    );
}

#[test]
fn validate_rejects_a_request_reference_of_another_kind() {
    for other in [
        "operation:018f0000-0000-7000-8000-000000000c01",
        "user:018f0000-0000-7000-8000-000000000005",
        "privacy-request:018f0000-0000-7000-8000-000000000c01",
    ] {
        let mut payload = typed(&[1]);
        payload.request_ref = other.parse().expect("a well-formed reference");
        assert!(
            matches!(
                payload.validate(),
                Err(OperationContractError::StreamErasureRequestKind { .. })
            ),
            "{other} must be refused"
        );
        let mut example = request(&[1]);
        example["request_ref"] = json!(other);
        assert!(serde_json::from_value::<PlatformStreamErasureRequested>(example).is_err());
    }
}

/// Sequences are `u64` and JSON numbers above 2^53 - 1 are not read exactly by JavaScript or
/// TypeScript consumers, so the contract refuses them instead of letting a rounded sequence name
/// a different stream message. Below that bound the value is preserved exactly.
#[test]
fn sequences_above_two_to_the_fifty_third_are_rejected_and_below_it_preserved_exactly() {
    assert_eq!(MAX_JSON_SAFE_SEQUENCE, (1_u64 << 53) - 1);
    assert_eq!(
        with_sequences(vec![MAX_JSON_SAFE_SEQUENCE + 1]).validate(),
        Err(OperationContractError::StreamSequenceAboveJsonSafeRange {
            index: 0,
            got: MAX_JSON_SAFE_SEQUENCE + 1
        })
    );
    assert!(
        serde_json::from_value::<PlatformStreamErasureRequested>(request(&[
            MAX_JSON_SAFE_SEQUENCE + 1
        ]))
        .is_err()
    );
    let exact = typed(&[MAX_JSON_SAFE_SEQUENCE]);
    let text = serde_json::to_string(&exact).unwrap();
    assert!(text.contains("9007199254740991"), "{text}");
    let back: PlatformStreamErasureRequested = serde_json::from_str(&text).unwrap();
    assert_eq!(back.stream_sequences, vec![MAX_JSON_SAFE_SEQUENCE]);
}

#[test]
fn deserialization_applies_the_same_rules() {
    for (sequences, needle) in [
        (vec![], "at least one"),
        (vec![0], "start at 1"),
        (vec![2, 1], "strictly ascending"),
        (vec![4, 4], "strictly ascending"),
        ((1..=501).collect(), "at most 500"),
    ] {
        let error = serde_json::from_value::<PlatformStreamErasureRequested>(request(&sequences))
            .expect_err("an invalid list is not a payload");
        assert!(error.to_string().contains(needle), "{error}");
    }
}
