//! The AI-archive event payloads and their envelope composition.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

mod common;

use ratatoskr_ai_archive_contracts::{
    AiArchiveCompleteness, AiArchiveImport, AiArchiveProvenance, AiArchiveTombstone,
    AiConversationAdded, AiConversationUpdated,
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
    assert_eq!(
        AiArchiveTombstone::EVENT_TYPE,
        "ai_archive.subject.tombstoned.v1"
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
        (AiArchiveTombstone::event_type(), "subject", "tombstoned"),
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
            import_provenance: provenance(),
            conversation: conversation.clone(),
            extensions: Extensions::new(),
        }),
        AiConversationPayload::Updated(AiConversationUpdated {
            import_provenance: provenance(),
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
                    decoded.import_provenance.ai_archive_id,
                    common::archive_id(),
                    "the owning import travels beside the record"
                );
                assert!(decoded.validate().is_ok(), "the imported linkage agrees");
            }
            AiConversationPayload::Updated(updated) => {
                envelope.set_payload(updated).expect("a JSON object body");
                assert_eq!(envelope.event_type, AiConversationUpdated::event_type());
                let decoded = envelope
                    .payload_as::<AiConversationUpdated>()
                    .expect("typed read");
                assert_eq!(decoded.conversation, conversation);
                assert!(decoded.validate().is_ok(), "the imported linkage agrees");
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

/// A conversation fact must stand on its own when a consumer has not retained
/// the corresponding import event.
#[test]
fn conversation_event_requires_self_contained_import_provenance() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/events/ai_archive.conversation.added.v1/valid/claude-added.json");
    let body = std::fs::read_to_string(&fixture)
        .unwrap_or_else(|error| panic!("{} is unreadable: {error}", fixture.display()));
    let mut payload: serde_json::Value = serde_json::from_str(&body).expect("the fixture is JSON");
    let removed = payload
        .as_object_mut()
        .expect("the fixture is an object")
        .remove("import_provenance");
    assert!(removed.is_some(), "the valid fixture carries provenance");

    let parsed = serde_json::from_value::<AiConversationAdded>(payload);
    assert!(
        parsed.is_err(),
        "a conversation event without immutable import provenance must be rejected"
    );
}

/// An authoritative tombstone must retain the exact deletion evidence and
/// cannot be represented by a missing-snapshot observation.
#[test]
fn tombstone_fixture_carries_authoritative_evidence() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/events/ai_archive.subject.tombstoned.v1/valid/conversation-tombstoned.json",
    );
    let body = std::fs::read_to_string(&fixture)
        .unwrap_or_else(|error| panic!("{} is unreadable: {error}", fixture.display()));
    let value: serde_json::Value = serde_json::from_str(&body).expect("the fixture is JSON");

    assert_eq!(
        value
            .pointer("/subject/subject_kind")
            .and_then(serde_json::Value::as_str),
        Some("conversation")
    );
    assert_eq!(
        value.pointer("/reason").and_then(serde_json::Value::as_str),
        Some("provider_deletion_event")
    );
    assert!(
        value
            .pointer("/evidence_ref/digest/hex")
            .is_some_and(serde_json::Value::is_string)
    );
    assert!(value.get("missing_from_latest_snapshot").is_none());
}

/// The published event family covers every state-carrying record and the two terminal facts
/// Knowledge and archive services need to converge after an import or tombstone.
#[test]
fn project_artifact_tombstone_and_linkage_event_schemas_exist() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the crate sits two levels below the repository root");
    for event in [
        "ai_archive.project.added.v1",
        "ai_archive.project.updated.v1",
        "ai_archive.artifact.added.v1",
        "ai_archive.artifact.updated.v1",
        "ai_archive.subject.tombstoned.v1",
        "knowledge.ai_archive_analysis.completed.v1",
    ] {
        assert!(
            root.join("schemas/events")
                .join(format!("{event}.schema.json"))
                .is_file(),
            "the generated schema for {event} must be published"
        );
    }
}

/// A consumer that asks for an archive payload from an unrelated envelope is refused.
#[test]
fn mismatched_or_unrelated_envelopes_are_refused() {
    let payload = AiConversationAdded {
        import_provenance: provenance(),
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

fn provenance() -> AiArchiveProvenance {
    AiArchiveProvenance::from_import(&common::import_with_report(common::report(
        AiArchiveCompleteness::Complete,
        1,
        0,
    )))
}

enum AiConversationPayload {
    Added(AiConversationAdded),
    Updated(AiConversationUpdated),
}
