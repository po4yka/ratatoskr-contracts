//! `EventEnvelope` serialization — tests E-1, E-5, E-6, E-7, E-8 and E-9.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use std::path::{Path, PathBuf};

use ratatoskr_event_envelope::{EnvelopeError, EventEnvelope, EventPayload, EventType};
use ratatoskr_identifiers::dropped_field_pointers;

/// The envelope printed in `ARCHITECTURE.md` S5.2, byte for byte, in canonical form.
const ARCHITECTURE_S5_2_EXAMPLE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000001",
  "event_type": "content.document.extracted.v1",
  "occurred_at": "2026-08-17T10:00:00Z",
  "producer": "ratatoskr-extractor",
  "aggregate_id": "document:018f0000-0000-7000-8000-000000000002",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000003",
  "causation_id": "event:018f0000-0000-7000-8000-000000000004",
  "tenant_id": "user:018f0000-0000-7000-8000-000000000005",
  "schema_version": 1,
  "payload": {}
}
"#;

/// The smallest legal envelope: both optional references absent, empty payload.
const MINIMAL_EXAMPLE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000011",
  "event_type": "platform.operation.progressed.v1",
  "occurred_at": "2026-08-17T10:00:00Z",
  "producer": "ratatoskr-platform",
  "aggregate_id": "operation:018f0000-0000-7000-8000-000000000012",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000012",
  "schema_version": 1,
  "payload": {
    "progress_percent": 42
  }
}
"#;

/// An envelope from a newer producer: one additive top-level member this build never heard of.
const UNKNOWN_ADDITIVE_FIELD_EXAMPLE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000021",
  "event_type": "content.document.extracted.v1",
  "occurred_at": "2026-08-17T10:00:00Z",
  "producer": "ratatoskr-extractor",
  "aggregate_id": "document:018f0000-0000-7000-8000-000000000022",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000023",
  "schema_version": 1,
  "payload": {},
  "retention_class": {
    "labels": [
      "audit"
    ],
    "retention_days": 30
  }
}
"#;

/// The canonical envelopes this crate carries in-tree, so the round-trip tests prove something
/// even before the `fixtures/` tree is populated.
const EMBEDDED_VALID_ENVELOPES: &[(&str, &str)] = &[
    ("architecture-s5-2-example", ARCHITECTURE_S5_2_EXAMPLE),
    ("minimal", MINIMAL_EXAMPLE),
    ("unknown-additive-field", UNKNOWN_ADDITIVE_FIELD_EXAMPLE),
];

/// A stand-in for `platform.operation.progressed.v1`'s real payload, which lives in
/// `ratatoskr-operation-contracts`. This crate must not depend on it: the dependency runs the
/// other way.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct ProgressProbe {
    /// Completed work as a whole-number percentage.
    progress_percent: u8,
}

impl EventPayload for ProgressProbe {
    const EVENT_TYPE: &'static str = "platform.operation.progressed.v1";
}

/// A payload bound to a different event type, so the binding can be shown to be enforced.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct ExtractionProbe {
    /// Number of blocks the extractor produced.
    block_count: u32,
}

impl EventPayload for ExtractionProbe {
    const EVENT_TYPE: &'static str = "content.document.extracted.v1";
}

/// The repository root, two levels above this crate.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the crate directory has a repository root above it")
}

/// Every `*.json` file directly inside `directory`, sorted by name. Empty when the directory does
/// not exist yet, so this crate's tests do not depend on the fixture tree being populated first.
pub(crate) fn json_files_in(directory: &Path) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<(String, String)> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| {
            let bytes = std::fs::read_to_string(&path).expect("a fixture must be readable UTF-8");
            (path.display().to_string(), bytes)
        })
        .collect();
    found.sort();
    found
}

/// The committed `valid/` envelope fixtures, plus the envelopes embedded above.
fn every_valid_envelope() -> Vec<(String, String)> {
    let mut all: Vec<(String, String)> = EMBEDDED_VALID_ENVELOPES
        .iter()
        .map(|(name, body)| ((*name).to_owned(), (*body).to_owned()))
        .collect();
    all.extend(json_files_in(
        &repo_root().join("fixtures/core/event-envelope/valid"),
    ));
    all
}

/// E-1. The normative document example parses, field for field.
#[test]
fn architecture_s5_2_example_is_a_valid_envelope() {
    let envelope = EventEnvelope::from_json(ARCHITECTURE_S5_2_EXAMPLE.as_bytes())
        .expect("the documented envelope must parse");

    assert_eq!(
        envelope.event_id.to_string(),
        "018f0000-0000-7000-8000-000000000001"
    );
    assert_eq!(
        envelope.event_type,
        EventType::parse("content.document.extracted.v1").unwrap()
    );
    assert_eq!(envelope.occurred_at.to_wire(), "2026-08-17T10:00:00Z");
    assert_eq!(envelope.producer.as_str(), "ratatoskr-extractor");
    assert_eq!(
        envelope.aggregate_id.to_wire(),
        "document:018f0000-0000-7000-8000-000000000002"
    );
    assert_eq!(
        envelope.correlation_id.to_wire(),
        "operation:018f0000-0000-7000-8000-000000000003"
    );
    assert_eq!(
        envelope
            .causation_id
            .as_ref()
            .map(ratatoskr_identifiers::EntityRef::to_wire),
        Some("event:018f0000-0000-7000-8000-000000000004".to_owned())
    );
    assert_eq!(
        envelope.tenant_id.map(|tenant| tenant.to_string()),
        Some("user:018f0000-0000-7000-8000-000000000005".to_owned())
    );
    assert_eq!(envelope.schema_version.get(), 1);
    assert!(envelope.payload.is_empty());
    assert!(envelope.extensions.is_empty());
}

/// E-5. JSON → `EventEnvelope` → JSON is byte-identical for every valid envelope, so a fixture
/// is a golden file and not merely a parseable one.
#[test]
fn envelope_json_roundtrip_is_byte_stable() {
    for (name, body) in every_valid_envelope() {
        let envelope = EventEnvelope::from_json(body.as_bytes())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(
            envelope.to_canonical_json().unwrap(),
            body,
            "{name} is not byte-stable"
        );
        // Nothing was silently discarded on the way through, either.
        let input: serde_json::Value = serde_json::from_str(&body).unwrap();
        let roundtripped: serde_json::Value =
            serde_json::from_str(&envelope.to_canonical_json().unwrap()).unwrap();
        assert_eq!(
            dropped_field_pointers(&input, &roundtripped),
            Vec::<String>::new(),
            "{name} lost members"
        );
    }
}

/// E-6. `set_payload` then `payload_as` is the identity, and `event_type` follows the payload.
#[test]
fn payload_as_and_set_payload_are_inverse() {
    let mut envelope = EventEnvelope::from_json(ARCHITECTURE_S5_2_EXAMPLE.as_bytes()).unwrap();
    let payload = ProgressProbe {
        progress_percent: 42,
    };

    envelope.set_payload(&payload).unwrap();

    assert_eq!(envelope.event_type, ProgressProbe::event_type());
    assert_eq!(
        envelope.event_type.to_wire(),
        "platform.operation.progressed.v1"
    );
    assert_eq!(envelope.payload_as::<ProgressProbe>().unwrap(), payload);
    assert_eq!(
        envelope.payload.get("progress_percent"),
        Some(&serde_json::json!(42))
    );
}

/// E-7. Asking for the wrong payload type is a named error, not a silently wrong struct.
#[test]
fn payload_as_wrong_type_fails() {
    let mut envelope = EventEnvelope::from_json(ARCHITECTURE_S5_2_EXAMPLE.as_bytes()).unwrap();
    envelope
        .set_payload(&ProgressProbe {
            progress_percent: 42,
        })
        .unwrap();

    match envelope.payload_as::<ExtractionProbe>() {
        Err(EnvelopeError::PayloadType { expected, found }) => {
            assert_eq!(expected, "content.document.extracted.v1");
            assert_eq!(found, "platform.operation.progressed.v1");
        }
        other => panic!("expected a PayloadType error, got {other:?}"),
    }
}

/// E-8. A scalar or array payload fails to deserialize: `payload` is an object by type.
#[test]
fn payload_must_be_a_json_object() {
    for body in [
        ARCHITECTURE_S5_2_EXAMPLE.replace(r#""payload": {}"#, r#""payload": 42"#),
        ARCHITECTURE_S5_2_EXAMPLE.replace(r#""payload": {}"#, r#""payload": []"#),
        ARCHITECTURE_S5_2_EXAMPLE.replace(r#""payload": {}"#, r#""payload": "text""#),
        ARCHITECTURE_S5_2_EXAMPLE.replace(r#""payload": {}"#, r#""payload": null"#),
    ] {
        assert!(
            EventEnvelope::from_json(body.as_bytes()).is_err(),
            "a non-object payload must be rejected: {body}"
        );
    }
}

/// E-9. `DOMAIN.md` invariant 6, **preserved** branch: an unknown top-level member survives the
/// parse and is re-emitted verbatim, which is what makes "add an optional field" backward
/// compatible for a consumer built today.
#[test]
fn unknown_top_level_fields_are_preserved_and_reemitted() {
    let envelope =
        EventEnvelope::from_json(UNKNOWN_ADDITIVE_FIELD_EXAMPLE.as_bytes()).expect("tolerant read");

    assert_eq!(envelope.extensions.len(), 1);
    assert_eq!(
        envelope.extensions.keys().collect::<Vec<_>>(),
        vec!["retention_class"]
    );
    assert_eq!(
        envelope.extensions.get("retention_class"),
        Some(&serde_json::json!({"labels": ["audit"], "retention_days": 30}))
    );

    let reemitted = envelope.to_canonical_json().unwrap();
    assert_eq!(reemitted, UNKNOWN_ADDITIVE_FIELD_EXAMPLE);

    let input: serde_json::Value = serde_json::from_str(UNKNOWN_ADDITIVE_FIELD_EXAMPLE).unwrap();
    let roundtripped: serde_json::Value = serde_json::from_str(&reemitted).unwrap();
    assert_eq!(
        dropped_field_pointers(&input, &roundtripped),
        Vec::<String>::new()
    );
}

/// `None` and an absent member are the same thing, and `None` always serializes as absent
/// (`DOMAIN.md` invariant 4).
#[test]
fn absent_and_null_optional_references_are_both_none() {
    let absent = EventEnvelope::from_json(MINIMAL_EXAMPLE.as_bytes()).unwrap();
    assert_eq!(absent.causation_id, None);
    assert_eq!(absent.tenant_id, None);

    let explicit_null = MINIMAL_EXAMPLE.replace(
        r#"  "schema_version": 1,"#,
        "  \"causation_id\": null,\n  \"tenant_id\": null,\n  \"schema_version\": 1,",
    );
    let parsed = EventEnvelope::from_json(explicit_null.as_bytes()).unwrap();
    assert_eq!(parsed, absent);
    assert_eq!(parsed.to_canonical_json().unwrap(), MINIMAL_EXAMPLE);
}
