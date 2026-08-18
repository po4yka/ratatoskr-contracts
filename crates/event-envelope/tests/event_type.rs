//! `EventType` — tests E-2, E-3, E-4, E-12 and E-13.
//!
//! The event name is a validated value, not a `String`: `ARCHITECTURE.md` S9.1 and ADR-0002.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_event_envelope::{EventPayload, EventType, EventTypeError};

/// Every event name printed in `README.md` and `ARCHITECTURE.md` S9.1, in document order.
/// `x.bookmark.observed.v1` is the one-character bounded context.
const DOCUMENTED_EVENT_TYPES: &[&str] = &[
    "content.document.extracted.v1",
    "github.repository.observed.v1",
    "vault.snapshot.verified.v1",
    "x.bookmark.observed.v1",
    "social.source.upserted.v1",
    "chatgpt.export.ingested.v1",
    "claude.project.upserted.v1",
    "knowledge.analysis.completed.v1",
    "platform.operation.progressed.v1",
    "telegram.interaction.received.v1",
];

/// A stand-in for the typed payloads that live in the contract crates downstream of this one.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct ProgressProbe {
    /// Completed work as a whole-number percentage. Present only to give the probe a field.
    progress_percent: u8,
}

impl EventPayload for ProgressProbe {
    const EVENT_TYPE: &'static str = "platform.operation.progressed.v1";
}

/// E-2. All ten documented names parse, including the one-character context `x`.
#[test]
fn parses_every_documented_example() {
    assert_eq!(DOCUMENTED_EVENT_TYPES.len(), 10);
    for raw in DOCUMENTED_EVENT_TYPES {
        let parsed = EventType::parse(raw).expect("a documented event type must parse");
        assert_eq!(parsed.to_wire(), *raw);
        assert_eq!(parsed.to_string(), *raw);
        assert_eq!(parsed.major(), 1);
        assert!(
            parsed.action_looks_past_tense(),
            "{raw} names a completed fact"
        );
    }

    let extracted = EventType::parse("content.document.extracted.v1").unwrap();
    assert_eq!(extracted.bounded_context(), "content");
    assert_eq!(extracted.aggregate(), "document");
    assert_eq!(extracted.action(), "extracted");
    assert_eq!(
        EventType::new("content", "document", "extracted", 1).unwrap(),
        extracted
    );
}

/// E-3. A name with no `.v<major>` suffix is not an event type.
#[test]
fn rejects_missing_version() {
    assert_eq!(
        EventType::parse("content.document.extracted"),
        Err(EventTypeError::WrongShape {
            input: "content.document.extracted".to_owned()
        })
    );
}

/// E-3. `v0` does not exist: majors start at 1.
#[test]
fn rejects_v0() {
    assert_eq!(
        EventType::parse("content.document.extracted.v0"),
        Err(EventTypeError::InvalidMajor {
            input: "content.document.extracted.v0".to_owned()
        })
    );
    assert!(EventType::new("content", "document", "extracted", 0).is_err());
}

/// E-3. A leading zero would give one major two spellings.
#[test]
fn rejects_leading_zero_major() {
    assert_eq!(
        EventType::parse("content.document.extracted.v01"),
        Err(EventTypeError::InvalidMajor {
            input: "content.document.extracted.v01".to_owned()
        })
    );
    assert!(EventType::parse("content.document.extracted.1").is_err());
    assert!(EventType::parse("content.document.extracted.v").is_err());
}

/// E-3. Segments are lowercase `snake_case`; an uppercase name is a different string on the wire.
#[test]
fn rejects_uppercase() {
    assert_eq!(
        EventType::parse("Content.document.extracted.v1"),
        Err(EventTypeError::InvalidSegment {
            segment: "Content".to_owned(),
            pattern: EventType::SEGMENT_PATTERN
        })
    );
    assert!(EventType::parse("content.Document.extracted.v1").is_err());
    assert!(EventType::parse("content.document.Extracted.v1").is_err());
    assert!(EventType::parse("content.document.extracted.V1").is_err());
}

/// E-3. Exactly four segments: three names and the version.
#[test]
fn rejects_three_segments() {
    assert_eq!(
        EventType::parse("document.extracted.v1"),
        Err(EventTypeError::WrongShape {
            input: "document.extracted.v1".to_owned()
        })
    );
    assert!(EventType::parse("a.b.c.d.v1").is_err());
    assert!(EventType::parse("").is_err());
    assert!(EventType::parse("content.document.extracted.v1.").is_err());
}

/// A name longer than the contract maximum reports its length rather than a pattern mismatch.
#[test]
fn rejects_an_over_long_name() {
    let long = format!("{}.document.extracted.v1", "a".repeat(120));
    assert_eq!(
        EventType::parse(&long),
        Err(EventTypeError::TooLong {
            got: long.len(),
            input: long.clone()
        })
    );
    assert!(long.len() > EventType::MAX_LEN);
    // A segment of exactly 32 characters is the published maximum and still parses.
    assert!(EventType::parse(&format!("{}.b.extracted.v1", "a".repeat(32))).is_ok());
    assert!(EventType::parse(&format!("{}.b.extracted.v1", "a".repeat(33))).is_err());
}

/// E-4. Every `EventPayload::EVENT_TYPE` parses, so `event_type()` cannot panic in a shipped
/// build. Each crate that declares payloads carries its own copy of this test over its own
/// types; this one covers the trait's default method.
#[test]
fn event_payload_constants_parse() {
    let parsed =
        EventType::parse(ProgressProbe::EVENT_TYPE).expect("EVENT_TYPE must be a valid name");
    assert_eq!(ProgressProbe::event_type(), parsed);
    assert_eq!(
        ProgressProbe::event_type().to_wire(),
        ProgressProbe::EVENT_TYPE
    );
}

/// E-12. `family()` lets a consumer key on the fact rather than on the payload version.
#[test]
fn family_is_stable_across_payload_majors() {
    let v1 = EventType::parse("content.document.extracted.v1").unwrap();
    let v2 = EventType::parse("content.document.extracted.v2").unwrap();
    assert_eq!(v1.family(), "content.document.extracted");
    assert_eq!(v1.family(), v2.family());
    assert_ne!(v1, v2);
    assert_ne!(
        v1.family(),
        EventType::parse("content.document.deleted.v1")
            .unwrap()
            .family()
    );
}

/// E-13. Bumping the major is the only sanctioned mutation; nothing else moves.
#[test]
fn with_major_changes_only_the_version() {
    let v1 = EventType::parse("platform.operation.progressed.v1").unwrap();
    let v2 = v1.with_major(2);
    assert_eq!(v2.to_wire(), "platform.operation.progressed.v2");
    assert_eq!(v2.major(), 2);
    assert_eq!(v2.family(), v1.family());
    assert_eq!(v2.with_major(1), v1);
    // Every major the wire grammar can spell survives the round trip through its own name.
    for major in [1, 2, 9, 10, 9999] {
        let bumped = v1.with_major(major);
        assert_eq!(bumped.major(), major);
        assert_eq!(EventType::parse(&bumped.to_wire()).unwrap(), bumped);
    }
}

/// A major outside the wire grammar's `1..=9999` is a caller bug, not a supported input.
///
/// A rollback path computing `current - 1` must not be handed a `v1` back with no signal, so the
/// debug assertion fires in every test and debug build. The specification pins the signature as
/// `-> Self`, so the release build still clamps rather than producing an unspellable name.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "outside the wire grammar")]
fn with_major_refuses_a_version_the_grammar_cannot_spell() {
    let name = EventType::parse("platform.operation.progressed.v1").unwrap();
    let _ = name.with_major(0);
}

/// The published `PATTERN` and the parser are the same contract, so JSON Schema and Rust agree.
#[test]
fn published_pattern_agrees_with_the_parser() {
    let pattern = regex::Regex::new(EventType::PATTERN).unwrap();
    for candidate in DOCUMENTED_EVENT_TYPES {
        assert!(pattern.is_match(candidate));
    }
    for candidate in [
        "content.document.extracted",
        "content.document.extracted.v0",
        "content.document.extracted.v01",
        "Content.document.extracted.v1",
        "document.extracted.v1",
        "content.document.extracted.v1\n",
        "content-document.extracted.v1",
    ] {
        assert!(!pattern.is_match(candidate), "{candidate} must not match");
        assert!(
            EventType::parse(candidate).is_err(),
            "{candidate} must not parse"
        );
    }
}

/// The name is a JSON string on the wire and survives serde in both directions.
#[test]
fn serializes_as_a_bare_json_string() {
    let parsed = EventType::parse("content.document.extracted.v1").unwrap();
    let json = serde_json::to_string(&parsed).unwrap();
    assert_eq!(json, r#""content.document.extracted.v1""#);
    assert_eq!(serde_json::from_str::<EventType>(&json).unwrap(), parsed);
    assert!(serde_json::from_str::<EventType>(r#""content.document.extracted.v0""#).is_err());
}
