//! Typed UUID identities — tests I-4, I-5, I-6 and I-7.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::{
    CorrelationId, EntityKind, EntityRef, EventId, IdentifierError, OperationId, UserId,
};

const EVENT_UUID: &str = "018f0000-0000-7000-8000-000000000001";
const USER_UUID: &str = "018f0000-0000-7000-8000-000000000005";

/// I-4. `ARCHITECTURE.md` S5.2 shows `event_id` bare, with no `event:` prefix.
#[test]
fn typed_id_serializes_as_bare_uuid() {
    let event_id = EventId::parse(EVENT_UUID).expect("a canonical UUID parses");
    assert_eq!(
        serde_json::to_string(&event_id).unwrap(),
        format!("\"{EVENT_UUID}\"")
    );
    assert_eq!(event_id.to_string(), EVENT_UUID);

    let decoded: EventId =
        serde_json::from_str(&format!("\"{EVENT_UUID}\"")).expect("and deserializes");
    assert_eq!(decoded, event_id);

    // A prefixed form is not a typed id.
    assert!(EventId::parse(&format!("event:{EVENT_UUID}")).is_err());
}

/// I-5. The two identifier roles bridge in both directions, and the bridge is kind-checked (D4).
#[test]
fn bridges_between_typed_id_and_entity_ref() {
    let user_id = UserId::parse(USER_UUID).expect("a canonical UUID parses");
    let as_ref: EntityRef = user_id.into();
    assert_eq!(as_ref.to_wire(), format!("user:{USER_UUID}"));
    assert_eq!(as_ref.kind(), &EntityKind::User);
    assert_eq!(
        UserId::try_from(&as_ref).expect("the kind matches"),
        user_id
    );

    let operation_ref = OperationId::parse(EVENT_UUID)
        .expect("a canonical UUID parses")
        .as_entity_ref();
    assert_eq!(operation_ref.to_wire(), format!("operation:{EVENT_UUID}"));
    let error = UserId::try_from(&operation_ref).expect_err("an operation is not a user");
    assert!(matches!(
        error,
        IdentifierError::KindMismatch { expected: "user", ref actual } if actual == "operation"
    ));

    // A provider-minted local id cannot become a typed id at all.
    let provider_ref = EntityRef::parse("user:123").expect("a provider local id is a legal ref");
    assert!(matches!(
        UserId::try_from(&provider_ref),
        Err(IdentifierError::NotAUuid { .. })
    ));

    // `CorrelationId` widens to the open vocabulary through a kind this build does not name.
    let correlation = CorrelationId::parse(EVENT_UUID).expect("a canonical UUID parses");
    assert_eq!(
        correlation.as_entity_ref().kind().as_str(),
        CorrelationId::KIND
    );
    assert_eq!(
        CorrelationId::try_from(&correlation.as_entity_ref()).expect("round trip"),
        correlation
    );
}

/// I-6. `uuid::Uuid::parse_str` accepts every one of these; the contract does not, because one
/// identity must have exactly one spelling.
#[test]
fn rejects_non_canonical_uuid_text() {
    for rejected in [
        "018F0000-0000-7000-8000-000000000001",
        "{018f0000-0000-7000-8000-000000000001}",
        "urn:uuid:018f0000-0000-7000-8000-000000000001",
        "018f0000000070008000000000000001",
    ] {
        assert!(
            uuid::Uuid::parse_str(rejected).is_ok(),
            "{rejected} is accepted by uuid::Uuid"
        );
        assert!(
            matches!(
                EventId::parse(rejected),
                Err(IdentifierError::PatternMismatch { .. })
            ),
            "{rejected} must not parse as an EventId"
        );
    }
}

/// I-7. Deserialization accepts any RFC 9562 UUID so identities minted before the v7 rule stay
/// replayable; `is_uuid_v7` reports the truth rather than lying about provenance.
#[test]
fn accepts_non_v7_uuid_and_reports_version() {
    let v4 = EventId::parse("018f0000-0000-4000-8000-000000000001").expect("a v4 UUID parses");
    assert!(!v4.is_uuid_v7());

    let v7 = EventId::parse(EVENT_UUID).expect("a v7 UUID parses");
    assert!(v7.is_uuid_v7());
    assert!(EventId::new_v7().is_uuid_v7());
    assert!(UserId::new_v7().is_uuid_v7());
    assert!(OperationId::new_v7().is_uuid_v7());
    assert!(CorrelationId::new_v7().is_uuid_v7());
}
