//! Envelope versioning — tests E-10 and E-11.
//!
//! The two compatibility axes behave differently on purpose (`DOMAIN.md` invariant 6, ADR-0002):
//! an unknown **field** inside the current envelope major is preserved, an unknown **envelope
//! major** is rejected.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_event_envelope::{EnvelopeError, EnvelopeSchemaVersion, EventEnvelope};

/// The `ARCHITECTURE.md` S5.2 envelope in canonical form, used as the base for each mutation.
const BASE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000001",
  "event_type": "content.document.extracted.v1",
  "occurred_at": "2026-08-17T10:00:00Z",
  "producer": "ratatoskr-extractor",
  "aggregate_id": "document:018f0000-0000-7000-8000-000000000002",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000003",
  "schema_version": 1,
  "payload": {}
}
"#;

/// E-10. `DOMAIN.md` invariant 6, **rejected explicitly** branch: a future envelope major is
/// refused at parse time, field-locally, with a named error. A build that cannot know which
/// members changed meaning must not half-interpret the record.
#[test]
fn rejects_future_envelope_major() {
    match EnvelopeSchemaVersion::try_from(2_u32) {
        Err(EnvelopeError::UnsupportedSchemaVersion { found, supported }) => {
            assert_eq!(found, 2);
            assert_eq!(supported, 1);
        }
        other => panic!("expected UnsupportedSchemaVersion, got {other:?}"),
    }

    let future = BASE.replace(r#""schema_version": 1"#, r#""schema_version": 2"#);
    let error = EventEnvelope::from_json(future.as_bytes())
        .expect_err("an envelope from a future envelope major must not parse");
    let reported = format!("{}", DisplayChain(&error));
    assert!(
        reported.contains("unsupported envelope schema_version 2"),
        "the rejection must name the offending major: {reported}"
    );
    assert!(
        reported.contains("this build supports 1"),
        "the rejection must name the supported major: {reported}"
    );

    // Zero is not a version this build ever emitted either.
    assert!(EnvelopeSchemaVersion::try_from(0_u32).is_err());
    assert!(
        EventEnvelope::from_json(
            BASE.replace(r#""schema_version": 1"#, r#""schema_version": 0"#)
                .as_bytes()
        )
        .is_err()
    );
}

/// E-11. Inside the current major, an unknown member is tolerated — the other axis.
#[test]
fn unknown_fields_within_the_current_major_are_tolerated() {
    let newer = BASE.replace(
        r#"  "payload": {}"#,
        "  \"payload\": {},\n  \"dispatch_attempt_count\": 3",
    );
    let envelope = EventEnvelope::from_json(newer.as_bytes())
        .expect("an additive member inside the current major is not a version problem");

    assert_eq!(envelope.schema_version, EnvelopeSchemaVersion::CURRENT);
    assert_eq!(envelope.schema_version.get(), 1);
    assert_eq!(
        envelope.extensions.get("dispatch_attempt_count"),
        Some(&serde_json::json!(3))
    );
    assert_eq!(envelope.to_canonical_json().unwrap(), newer);
}

/// The version is an integer on the wire, and `CURRENT` is the only value it takes.
#[test]
fn envelope_schema_version_serializes_as_a_bare_integer() {
    assert_eq!(
        serde_json::to_string(&EnvelopeSchemaVersion::CURRENT).unwrap(),
        "1"
    );
    assert_eq!(
        serde_json::from_str::<EnvelopeSchemaVersion>("1").unwrap(),
        EnvelopeSchemaVersion::CURRENT
    );
    assert!(serde_json::from_str::<EnvelopeSchemaVersion>("2").is_err());
    assert_eq!(u32::from(EnvelopeSchemaVersion::CURRENT), 1);
}

/// `serde_json` reports the cause of a `try_from` rejection through the error source chain, so
/// the assertions above read the whole chain rather than only the outermost message.
struct DisplayChain<'a>(&'a EnvelopeError);

impl std::fmt::Display for DisplayChain<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        let mut source = std::error::Error::source(self.0);
        while let Some(cause) = source {
            write!(f, ": {cause}")?;
            source = cause.source();
        }
        Ok(())
    }
}
