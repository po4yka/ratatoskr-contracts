//! Property tests for identifiers and timestamps — Q-1, Q-2, Q-3 and Q-4.
//!
//! `ARCHITECTURE.md` S12 requires "property tests for identifiers, timestamps, and envelopes";
//! `AGENTS.md` prefers property-based testing for identifiers, parsers and serialization
//! invariants.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use proptest::prelude::*;
use ratatoskr_identifiers::{
    BlobOwner, CorrelationId, DigestHex, DocumentId, EntityLocalId, EntityRef, EventId, MediaType,
    OperationId, SafeMessage, UserId, WireTimestamp,
};

/// Q-3 helper: the published `PATTERN` is the compiled validator, so a string the pattern rejects
/// must never parse. The converse does not hold: `MAX_LEN` is measured in bytes and can reject a
/// multi-byte string the character-counting pattern accepts.
fn rejects_everything_outside_pattern<T>(
    pattern: &str,
    parse: impl Fn(&str) -> Result<T, ratatoskr_identifiers::IdentifierError>,
    candidate: &str,
) -> Result<(), TestCaseError> {
    let compiled = regex::Regex::new(pattern).expect("PATTERN compiles");
    if !compiled.is_match(candidate) {
        prop_assert!(
            parse(candidate).is_err(),
            "{candidate:?} does not match {pattern} yet parsed"
        );
    }
    Ok(())
}

proptest! {
    /// Q-1.
    #[test]
    fn prop_typed_id_roundtrip(bytes in any::<[u8; 16]>()) {
        let value = uuid::Uuid::from_bytes(bytes);
        let wire = value.to_string();

        prop_assert_eq!(EventId::parse(&wire).unwrap(), EventId(value));
        prop_assert_eq!(DocumentId::parse(&wire).unwrap(), DocumentId(value));
        prop_assert_eq!(UserId::parse(&wire).unwrap(), UserId(value));
        prop_assert_eq!(OperationId::parse(&wire).unwrap(), OperationId(value));
        prop_assert_eq!(CorrelationId::parse(&wire).unwrap(), CorrelationId(value));

        let event_id = EventId(value);
        prop_assert_eq!(EventId::parse(&event_id.to_string()).unwrap(), event_id);
        prop_assert_eq!(
            EventId::try_from(&event_id.as_entity_ref()).unwrap(),
            event_id
        );
    }

    /// Q-2. Every pair drawn from the two published patterns lands in exactly one of two states,
    /// and the split is the ADR-0007 canonical-UUID rule. Stated as both branches so the property
    /// stays total: the rule narrows the accepted set, and a property that only asserted the
    /// accepting branch would be false.
    #[test]
    fn prop_entity_ref_roundtrip(
        kind in "[a-z][a-z0-9_-]{0,31}",
        local_id in "[A-Za-z0-9][A-Za-z0-9._~:@+-]{0,255}",
    ) {
        let wire = format!("{kind}:{local_id}");
        // `EventId::parse` succeeds on exactly the canonical lowercase hyphenated UUIDs, so it is
        // the canonicality oracle without a second copy of the pattern.
        let folded = local_id.to_ascii_lowercase();
        let non_canonical_uuid = folded != local_id && EventId::parse(&folded).is_ok();

        let Ok(parsed) = EntityRef::parse(&wire) else {
            prop_assert!(
                non_canonical_uuid,
                "{wire} was rejected but is not a non-canonical UUID"
            );
            return Ok(());
        };
        prop_assert!(!non_canonical_uuid, "{wire} is a non-canonical UUID yet parsed");

        prop_assert_eq!(parsed.to_wire(), wire.clone());
        prop_assert_eq!(parsed.to_string(), wire.clone());
        prop_assert_eq!(parsed.kind().as_str(), kind.as_str());
        prop_assert_eq!(parsed.local_id().as_str(), local_id.as_str());
        prop_assert_eq!(EntityRef::parse(&parsed.to_wire()).unwrap(), parsed.clone());

        let json = serde_json::to_string(&parsed).unwrap();
        prop_assert_eq!(json, format!("\"{wire}\""));
    }

    /// Q-3, over every macro-generated newtype this crate publishes.
    #[test]
    fn prop_rejects_strings_outside_the_published_pattern(candidate in ".{0,64}") {
        rejects_everything_outside_pattern(
            EntityLocalId::PATTERN,
            EntityLocalId::parse,
            &candidate,
        )?;
        rejects_everything_outside_pattern(BlobOwner::PATTERN, BlobOwner::parse, &candidate)?;
        rejects_everything_outside_pattern(DigestHex::PATTERN, DigestHex::parse, &candidate)?;
        rejects_everything_outside_pattern(MediaType::PATTERN, MediaType::parse, &candidate)?;
        rejects_everything_outside_pattern(SafeMessage::PATTERN, SafeMessage::parse, &candidate)?;
    }

    /// Q-4.
    #[test]
    fn prop_timestamp_roundtrip_and_canonicality(
        second in 0i64..=253_402_300_799i64,
        nanosecond in 0i32..1_000_000_000i32,
    ) {
        let pattern = regex::Regex::new(WireTimestamp::PATTERN).expect("PATTERN compiles");
        let instant = jiff::Timestamp::new(second, nanosecond).unwrap();
        let stamp = WireTimestamp::from_jiff(instant);
        let wire = stamp.to_wire();

        prop_assert!(pattern.is_match(&wire), "{wire} does not match the published pattern");
        prop_assert_eq!(WireTimestamp::parse(&wire).unwrap(), stamp);
        // Re-canonicalization is idempotent.
        prop_assert_eq!(WireTimestamp::parse(&wire).unwrap().to_wire(), wire);
    }
}
