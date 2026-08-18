//! Property tests for envelopes — Q-5 and Q-6.
//!
//! `ARCHITECTURE.md` S12 requires "property tests for identifiers, timestamps, and envelopes";
//! `AGENTS.md` prefers property-based testing for parsers and serialization invariants.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use proptest::prelude::*;
use ratatoskr_event_envelope::{EnvelopeSchemaVersion, EventEnvelope, EventType, ProducerName};
use ratatoskr_identifiers::{
    EntityKind, EntityLocalId, EntityRef, EventId, Extensions, TenantRef, UserId, WireTimestamp,
    dropped_field_pointers,
};

/// A well-formed name segment: the published `SEGMENT_PATTERN` alphabet, kept short so shrinking
/// stays readable.
fn segment() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,7}"
}

/// A `<kind>:<local_id>` reference whose kind may be one this build has never heard of.
fn entity_ref() -> impl Strategy<Value = EntityRef> {
    (
        "[a-z][a-z0-9_-]{0,7}",
        "[A-Za-z0-9][A-Za-z0-9._~:@+-]{0,15}",
    )
        .prop_map(|(kind, local)| {
            EntityRef::new(
                EntityKind::parse(&kind).unwrap(),
                EntityLocalId::parse(&local).unwrap(),
            )
        })
}

/// An extension map whose keys cannot collide with a known envelope member.
fn extensions() -> impl Strategy<Value = Extensions> {
    proptest::collection::btree_map(
        "ext_[a-z]{1,8}",
        prop_oneof![
            Just(serde_json::Value::Null),
            any::<bool>().prop_map(serde_json::Value::from),
            any::<i32>().prop_map(serde_json::Value::from),
            "[a-z ]{0,16}".prop_map(serde_json::Value::from),
        ],
        0..4,
    )
    .prop_map(|entries| {
        let mut preserved = Extensions::new();
        for (key, value) in entries {
            preserved.insert(key, value);
        }
        preserved
    })
}

prop_compose! {
    /// An arbitrary well-formed envelope: every required member present, both optional members
    /// independently present or absent, and an arbitrary preserved-extension map.
    fn arbitrary_envelope()(
        event_bytes in any::<[u8; 16]>(),
        bounded_context in segment(),
        aggregate in segment(),
        action in segment(),
        major in 1_u32..=9999,
        seconds in 0_i64..4_000_000_000,
        nanoseconds in 0_i32..1_000_000_000,
        producer in "[a-z][a-z0-9-]{1,15}",
        aggregate_id in entity_ref(),
        correlation_id in entity_ref(),
        causation_id in proptest::option::of(entity_ref()),
        tenant_bytes in proptest::option::of(any::<[u8; 16]>()),
        payload_count in 0_u8..4,
        extensions in extensions(),
    ) -> EventEnvelope {
        let mut payload = serde_json::Map::new();
        for index in 0..payload_count {
            payload.insert(format!("field_{index}"), serde_json::json!(index));
        }
        EventEnvelope {
            event_id: EventId(uuid::Uuid::from_bytes(event_bytes)),
            event_type: EventType::new(&bounded_context, &aggregate, &action, major).unwrap(),
            occurred_at: WireTimestamp::from_jiff(
                jiff::Timestamp::new(seconds, nanoseconds).unwrap(),
            ),
            producer: ProducerName::parse(&producer).unwrap(),
            aggregate_id,
            correlation_id,
            causation_id,
            tenant_id: tenant_bytes
                .map(|bytes| TenantRef::of_user(UserId(uuid::Uuid::from_bytes(bytes)))),
            schema_version: EnvelopeSchemaVersion::CURRENT,
            payload,
            extensions,
        }
    }
}

proptest! {
    /// Q-5. An arbitrary well-formed envelope survives canonical JSON in both directions with no
    /// member dropped and no byte moved.
    #[test]
    fn prop_envelope_roundtrip_is_lossless(envelope in arbitrary_envelope()) {
        let rendered = envelope.to_canonical_json().unwrap();
        let parsed = EventEnvelope::from_json(rendered.as_bytes()).unwrap();

        prop_assert_eq!(&parsed, &envelope);
        prop_assert_eq!(parsed.to_canonical_json().unwrap(), rendered.clone());
        prop_assert!(rendered.ends_with("}\n"), "canonical JSON ends with one newline");

        let input: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let roundtripped: serde_json::Value =
            serde_json::from_str(&parsed.to_canonical_json().unwrap()).unwrap();
        prop_assert_eq!(dropped_field_pointers(&input, &roundtripped), Vec::<String>::new());
    }

    /// Q-6. Well-formed names round-trip with the right `major()`; anything the published
    /// `PATTERN` rejects is rejected by the parser too, so JSON Schema and Rust cannot drift.
    #[test]
    fn prop_event_type_roundtrip(
        bounded_context in segment(),
        aggregate in segment(),
        action in segment(),
        major in 1_u32..=9999,
    ) {
        let wire = format!("{bounded_context}.{aggregate}.{action}.v{major}");
        let parsed = EventType::parse(&wire).unwrap();

        prop_assert_eq!(parsed.to_wire(), wire.clone());
        prop_assert_eq!(parsed.major(), major);
        prop_assert_eq!(parsed.family(), format!("{bounded_context}.{aggregate}.{action}"));
        prop_assert_eq!(EventType::parse(&parsed.to_wire()).unwrap(), parsed.clone());
        prop_assert_eq!(
            EventType::new(&bounded_context, &aggregate, &action, major).unwrap(),
            parsed
        );
    }

    /// Q-6, negative half: the published `PATTERN` is the contract, and the parser agrees with it
    /// on every candidate string.
    #[test]
    fn prop_parser_agrees_with_the_published_pattern(candidate in "[a-zA-Z0-9._v-]{0,48}") {
        let published = regex::Regex::new(EventType::PATTERN).unwrap();
        prop_assert_eq!(published.is_match(&candidate), EventType::parse(&candidate).is_ok());
    }
}
