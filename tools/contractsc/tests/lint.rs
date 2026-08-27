//! The field lint — tests L-1 to L-8.
//!
//! `AGENTS.md`'s review rules, mechanised: "Reject fields that are added with vague names such as
//! `status`, `data`, `metadata`, or `timestamp` without a precise documented contract",
//! principle 5 (timestamp authority) and principle 6 (explicit units).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{Finding, GENERATOR_VERSION, Metadata, generate, lint};

/// Where the synthetic probe document pretends to live.
const PROBE_OUTPUT: &str = "schemas/json-schema/core/probe.v1.schema.json";

/// A justification long enough to clear `min_waiver_justification_chars`.
const REAL_JUSTIFICATION: &str = "Name fixed by the normative wire example; the value is a closed enum with documented \
     terminality, so the contract is precise despite the generic name.";

/// The repository root, derived from the compiled-in manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

/// A synthetic metadata document declaring one `Probe` root type, plus whatever governance the
/// caller appends.
fn probe_metadata(tail: &str) -> Metadata {
    let text = format!(
        r#"metadata_version = 1

[services]
known = ["ratatoskr-platform"]

[entity_kinds]
known = ["user"]

[lint]
banned_property_names = ["data", "metadata", "status", "timestamp"]
required_numeric_suffixes = ["_bytes", "_count"]
timestamp_property_names = ["status_changed_at", "imported_at"]
min_waiver_justification_chars = 40

[[contract]]
id             = "core.probe"
family         = "core"
major_version  = 1
lifecycle      = "proposed"
classification = "internal"
owner          = "ratatoskr-platform"
producers      = ["ratatoskr-platform"]
consumers      = ["ratatoskr-platform"]
crate_name     = "ratatoskr-event-envelope"
canonical_path = "crates/event-envelope/src/envelope.rs"
fixtures_dir   = "fixtures/core/event-envelope"
summary        = "A synthetic contract that exists only inside the lint tests."

[[contract.root_type]]
rust_path      = "probe::Probe"
output         = "{PROBE_OUTPUT}"
schema_id      = "urn:ratatoskr:contracts:core:v1:Probe"
unknown_policy = "preserve"
privacy        = "boundary_metadata"
{tail}
"#
    );
    Metadata::parse(&text).expect("the probe metadata parses")
}

/// A synthetic generated document titled `Probe`, carrying the supplied properties.
fn probe_schemas(properties: &serde_json::Value) -> BTreeMap<PathBuf, String> {
    let document = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Probe",
        "type": "object",
        "properties": properties,
    });
    let body = serde_json::to_string_pretty(&document).expect("a probe document serializes");
    BTreeMap::from([(PathBuf::from(PROBE_OUTPUT), body)])
}

/// Every `(rule, pointer)` the lint reported.
fn lint_hits(metadata: &Metadata, schemas: &BTreeMap<PathBuf, String>) -> Vec<(String, String)> {
    lint::run(metadata, schemas)
        .into_iter()
        .filter_map(|finding| match finding {
            Finding::Lint { rule, pointer, .. } => Some((rule.to_owned(), pointer)),
            _ => None,
        })
        .collect()
}

/// A described string property.
fn described(kind: &str) -> serde_json::Value {
    serde_json::json!({ "type": kind, "description": "A described property." })
}

/// L-1. Every property of every real generated schema carries a description, because rustdoc is
/// what schemars turns into `description`. `AGENTS.md` principle 1 is a build failure, not a wish.
#[test]
fn every_property_has_a_description() {
    let text = std::fs::read_to_string(repo_root().join(Metadata::FILE_NAME))
        .expect("contracts.toml is committed");
    let metadata = Metadata::parse(&text).expect("contracts.toml parses");
    let generated =
        generate(&metadata, GENERATOR_VERSION).expect("the committed contracts generate");
    let undocumented: Vec<(String, String)> = lint_hits(&metadata, &generated)
        .into_iter()
        .filter(|(rule, _)| rule == "L5")
        .collect();
    assert!(undocumented.is_empty(), "{undocumented:?}");
}

/// L-2. Four vague names yield four findings; four justified waivers clear exactly those four.
#[test]
fn rejects_vague_property_names() {
    let schemas = probe_schemas(&serde_json::json!({
        "data": described("object"),
        "metadata": described("object"),
        "status": described("string"),
        "timestamp": described("string"),
    }));

    let bare = probe_metadata("");
    let mut flagged: Vec<String> = lint_hits(&bare, &schemas)
        .into_iter()
        .filter(|(rule, _)| rule == "L1")
        .map(|(_, pointer)| pointer)
        .collect();
    flagged.sort();
    assert_eq!(
        flagged,
        vec![
            "Probe#/properties/data",
            "Probe#/properties/metadata",
            "Probe#/properties/status",
            "Probe#/properties/timestamp",
        ]
    );

    let waivers = ["data", "metadata", "status", "timestamp"]
        .map(|name| {
            format!(
                "\n[[contract.vague_field_waiver]]\npointer       = \"/properties/{name}\"\njustification = \"{REAL_JUSTIFICATION}\"\n"
            )
        })
        .join("");
    let waived = probe_metadata(&waivers);
    assert!(
        lint_hits(&waived, &schemas).is_empty(),
        "{:?}",
        lint_hits(&waived, &schemas)
    );
}

/// L-3. An integer without a unit suffix fails; the same value renamed passes.
/// `AGENTS.md` principle 6: "Money, sizes, durations, and counters have explicit units."
#[test]
fn rejects_unitless_numeric_property() {
    let metadata = probe_metadata("");
    let unitless = probe_schemas(&serde_json::json!({ "size": described("integer") }));
    assert_eq!(
        lint_hits(&metadata, &unitless),
        vec![("L2".to_owned(), "Probe#/properties/size".to_owned())]
    );

    let with_unit = probe_schemas(&serde_json::json!({ "size_bytes": described("integer") }));
    assert!(lint_hits(&metadata, &with_unit).is_empty());
}

/// L-4. A `format: date-time` property outside the timestamp vocabulary fails; one inside it with
/// a `[[contract.field]]` entry declaring its clock authority passes.
/// `AGENTS.md` principle 5: "Document whether a timestamp is provider-authored, observed,
/// received, or processed."
#[test]
fn rejects_unknown_timestamp_authority() {
    let instant = serde_json::json!({
        "type": "string",
        "format": "date-time",
        "description": "An instant.",
    });

    let ungoverned = probe_schemas(&serde_json::json!({ "updated_time": instant.clone() }));
    let hits = lint_hits(&probe_metadata(""), &ungoverned);
    // Both L4 branches fail here — the name is outside the vocabulary *and* no `[[contract.field]]`
    // entry declares its clock authority — and the report must carry both. Collapsing them would
    // send an operator round the fix-and-rerun loop twice for one property.
    assert_eq!(
        hits.iter()
            .filter(|(rule, pointer)| rule == "L4" && pointer == "Probe#/properties/updated_time")
            .count(),
        2,
        "both L4 branches must be reported in one run: {hits:?}"
    );

    let governed = probe_metadata(
        "\n[[contract.field]]\n\
         pointer     = \"/properties/status_changed_at\"\n\
         authority   = \"observed\"\n\
         nullability = \"required\"\n\
         unit        = \"instant_rfc3339_utc_canonical\"\n\
         note        = \"Instant the platform observed the status change.\"\n",
    );
    let named = probe_schemas(&serde_json::json!({ "status_changed_at": instant }));
    assert!(
        lint_hits(&governed, &named).is_empty(),
        "{:?}",
        lint_hits(&governed, &named)
    );
}

/// Nested types are governed by the type that declares their property, even when they appear
/// below an event root. Repeating the governance on every root would let one declaration drift.
#[test]
fn accepts_governance_for_a_nested_declaring_type() {
    let instant = serde_json::json!({
        "type": "string",
        "format": "date-time",
        "description": "The instant the archive import completed.",
    });
    let document = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Probe",
        "type": "object",
        "properties": {
            "provenance": {
                "$ref": "#/$defs/Provenance",
                "description": "Immutable evidence for the normalized record."
            }
        },
        "$defs": {
            "Provenance": {
                "type": "object",
                "properties": { "imported_at": instant }
            }
        }
    });
    let body = serde_json::to_string_pretty(&document).expect("a probe document serializes");
    let schemas = BTreeMap::from([(PathBuf::from(PROBE_OUTPUT), body)]);
    let metadata = probe_metadata(
        "\n[[contract.field]]\n\
         pointer     = \"Provenance#/properties/imported_at\"\n\
         authority   = \"observed\"\n\
         nullability = \"required\"\n\
         unit        = \"instant_rfc3339_utc_canonical\"\n\
         note        = \"Instant the archive importer completed normalized processing.\"\n",
    );

    assert!(lint_hits(&metadata, &schemas).is_empty());
}

/// L-5. A floating-point property is rejected outright: it is not waivable, because float
/// rendering is platform-dependent and no contract here needs one.
#[test]
fn rejects_floating_point_property() {
    let schemas = probe_schemas(&serde_json::json!({ "ratio": described("number") }));
    assert_eq!(
        lint_hits(&probe_metadata(""), &schemas),
        vec![("L3".to_owned(), "Probe#/properties/ratio".to_owned())]
    );
}

/// L-6. A waiver justified with `TODO` is not a justification.
#[test]
fn rejects_empty_waiver_justification() {
    let metadata = probe_metadata(
        "\n[[contract.vague_field_waiver]]\npointer       = \"/properties/status\"\njustification = \"TODO\"\n",
    );
    let schemas = probe_schemas(&serde_json::json!({ "status": described("string") }));
    let hits = lint_hits(&metadata, &schemas);
    assert!(
        hits.contains(&("L7".to_owned(), "Probe#/properties/status".to_owned())),
        "{hits:?}"
    );
}

/// L-8. ADR-0002: `schema_version` is the envelope major and nothing else may be called that.
/// The committed catalogue is clean, and a payload that declares its own `schema_version` — which
/// is exactly what `ARCHITECTURE.md` S6.1's `Document` does — fails the gate rather than shipping
/// two same-named integers in one message.
#[test]
fn reserves_schema_version_for_the_envelope() {
    let text = std::fs::read_to_string(repo_root().join(Metadata::FILE_NAME))
        .expect("contracts.toml is committed");
    let metadata = Metadata::parse(&text).expect("contracts.toml parses");
    let generated =
        generate(&metadata, GENERATOR_VERSION).expect("the committed contracts generate");
    assert!(
        !lint_hits(&metadata, &generated)
            .iter()
            .any(|(rule, _)| rule == "L8"),
        "only recognized envelope roots declare schema_version"
    );

    let colliding = probe_schemas(&serde_json::json!({
        "schema_version": described("integer"),
    }));
    let hits = lint_hits(&probe_metadata(""), &colliding);
    assert!(
        hits.contains(&(
            "L8".to_owned(),
            "Probe#/properties/schema_version".to_owned()
        )),
        "{hits:?}"
    );
}

/// L-7. Event tense is governed in the repository, not in the runtime parser (ADR-0002,
/// `AGENTS.md` principle 9). The committed catalogue is clean; a present-participle action is not.
#[test]
fn registered_event_actions_are_past_tense() {
    let text = std::fs::read_to_string(repo_root().join(Metadata::FILE_NAME))
        .expect("contracts.toml is committed");
    let metadata = Metadata::parse(&text).expect("contracts.toml parses");
    let generated =
        generate(&metadata, GENERATOR_VERSION).expect("the committed contracts generate");
    assert!(
        !lint_hits(&metadata, &generated)
            .iter()
            .any(|(rule, _)| rule == "L6"),
        "the committed event catalogue is past tense"
    );

    let present_tense = probe_metadata(
        "\n[contract.event]\n\
         event_type   = \"platform.operation.progressing.v1\"\n\
         payload_type = \"probe::Probe\"\n",
    );
    let hits = lint_hits(&present_tense, &probe_schemas(&serde_json::json!({})));
    assert!(
        hits.contains(&(
            "L6".to_owned(),
            "platform.operation.progressing.v1".to_owned()
        )),
        "{hits:?}"
    );
}
