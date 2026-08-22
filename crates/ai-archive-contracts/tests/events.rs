//! The three AI-archive event payloads and their envelope composition.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

mod common;

use ratatoskr_ai_archive_contracts::{
    AiArchiveCompleteness, AiArchiveImport, AiConversationAdded, AiConversationUpdated,
};
use ratatoskr_event_envelope::{EventEnvelope, EventPayload};
use ratatoskr_identifiers::{Extensions, dropped_field_pointers};

pub(crate) use common::MINIMAL_ENVELOPE;

/// All three payloads are registered facts with past-tense actions at major 1.
#[test]
fn event_type_constants_are_the_registered_names() {
    assert_eq!(
        AiArchiveImport::EVENT_TYPE,
        "ai_archive.archive.imported.v1"
    );
    assert_eq!(
        AiConversationAdded::EVENT_TYPE,
        "ai_archive.conversation.added.v1"
    );
    assert_eq!(
        AiConversationUpdated::EVENT_TYPE,
        "ai_archive.conversation.updated.v1"
    );

    for (payload_type, aggregate, action) in [
        (
            <AiArchiveImport as ratatoskr_event_envelope::EventPayload>::event_type(),
            "archive",
            "imported",
        ),
        (AiConversationAdded::event_type(), "conversation", "added"),
        (
            AiConversationUpdated::event_type(),
            "conversation",
            "updated",
        ),
    ] {
        assert_eq!(payload_type.bounded_context(), "ai_archive");
        assert_eq!(payload_type.aggregate(), aggregate);
        assert_eq!(payload_type.action(), action);
        assert_eq!(payload_type.major(), 1);
        assert!(payload_type.action_looks_past_tense());
    }
}

/// The imported event carries the import head inside a real envelope; head and payload are one
/// type.
#[test]
fn imported_payload_travels_inside_a_real_envelope() {
    let head: AiArchiveImport =
        common::import_with_report(common::report(AiArchiveCompleteness::Complete, 1, 0));

    let mut envelope =
        EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes()).expect("the minimal envelope parses");
    envelope
        .set_payload(&head)
        .expect("the payload serializes to a JSON object");
    assert_eq!(envelope.event_type, AiArchiveImport::event_type());

    let wire = envelope.to_canonical_json().expect("re-serializes");
    let reparsed = EventEnvelope::from_json(wire.as_bytes()).expect("round trips");
    assert_eq!(reparsed, envelope);

    let sent: serde_json::Value = serde_json::from_str(&wire).expect("wire is JSON");
    let received: serde_json::Value = serde_json::to_value(&reparsed).expect("re-serialize");
    assert_eq!(
        dropped_field_pointers(&sent, &received),
        Vec::<String>::new(),
        "the envelope must not discard any member of its own payload"
    );

    let decoded: AiArchiveImport = reparsed
        .payload_as::<AiArchiveImport>()
        .expect("typed read");
    assert_eq!(decoded, head);
}

/// Added and updated events each carry one whole conversation plus the owning import.
#[test]
fn conversation_events_carry_the_whole_conversation() {
    let conversation = common::minimal_conversation();
    for payload in [
        AiConversationPayload::Added(AiConversationAdded {
            ai_archive_id: common::archive_id(),
            conversation: conversation.clone(),
            extensions: Extensions::new(),
        }),
        AiConversationPayload::Updated(AiConversationUpdated {
            ai_archive_id: common::archive_id(),
            conversation: conversation.clone(),
            extensions: Extensions::new(),
        }),
    ] {
        let mut envelope = EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes())
            .expect("the minimal envelope parses");
        match &payload {
            AiConversationPayload::Added(added) => {
                envelope.set_payload(added).expect("a JSON object body");
                assert_eq!(envelope.event_type, AiConversationAdded::event_type());
                let decoded = envelope
                    .payload_as::<AiConversationAdded>()
                    .expect("typed read");
                assert_eq!(decoded.conversation, conversation);
                assert_eq!(
                    decoded.ai_archive_id,
                    common::archive_id(),
                    "the owning import travels beside the record"
                );
            }
            AiConversationPayload::Updated(updated) => {
                envelope.set_payload(updated).expect("a JSON object body");
                assert_eq!(envelope.event_type, AiConversationUpdated::event_type());
                let decoded = envelope
                    .payload_as::<AiConversationUpdated>()
                    .expect("typed read");
                assert_eq!(decoded.conversation, conversation);
            }
        }

        let wire = envelope.to_canonical_json().expect("re-serializes");
        let reparsed = EventEnvelope::from_json(wire.as_bytes()).expect("round trips");
        assert_eq!(reparsed, envelope);
        let sent: serde_json::Value = serde_json::from_str(&wire).expect("wire is JSON");
        let received: serde_json::Value = serde_json::to_value(&reparsed).expect("re-serialize");
        assert_eq!(
            dropped_field_pointers(&sent, &received),
            Vec::<String>::new()
        );
    }
}

/// A consumer that asks for an archive payload from an unrelated envelope is refused.
#[test]
fn mismatched_or_unrelated_envelopes_are_refused() {
    let payload = AiConversationAdded {
        ai_archive_id: common::archive_id(),
        conversation: common::minimal_conversation(),
        extensions: Extensions::new(),
    };
    let mut envelope =
        EventEnvelope::from_json(MINIMAL_ENVELOPE.as_bytes()).expect("the envelope parses");
    envelope.set_payload(&payload).expect("a JSON object body");

    let error = envelope
        .payload_as::<AiConversationUpdated>()
        .expect_err("an added event is not an updated event");
    assert!(
        error
            .to_string()
            .contains("ai_archive.conversation.updated.v1"),
        "unexpected error: {error}"
    );

    let unrelated = MINIMAL_ENVELOPE.replace(
        "\"event_type\": \"ai_archive.archive.imported.v1\"",
        "\"event_type\": \"platform.operation.progressed.v1\"",
    );
    let envelope = EventEnvelope::from_json(unrelated.as_bytes()).expect("parses");
    let error = envelope
        .payload_as::<AiArchiveImport>()
        .expect_err("a progressed operation is not an imported archive");
    assert!(
        error
            .to_string()
            .contains("platform.operation.progressed.v1"),
        "unexpected error: {error}"
    );
}

enum AiConversationPayload {
    Added(AiConversationAdded),
    Updated(AiConversationUpdated),
}
