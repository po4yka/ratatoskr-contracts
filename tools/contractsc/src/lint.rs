//! The field lint — the machine-readable form of `AGENTS.md`'s review rule "Reject fields that
//! are added with vague names such as `status`, `data`, `metadata`, or `timestamp` without a
//! precise documented contract".
//!
//! It walks every property of every type reachable from a generated schema, driven by the
//! `[lint]` vocabulary in `contracts.toml`. Rules L4 and L1/L7 are also metadata rules R10 and
//! R11; they are implemented once, here, so a violation is reported once.
//!
//! # Why the lint is keyed by declaring type, not by file
//!
//! `OperationSnapshot` appears at the root of `operation-snapshot.v1.schema.json` and again in
//! the `$defs` of `platform.operation.progressed.v1.schema.json`. A property is a property of
//! its **declaring type**, so it is linted once and one waiver clears it everywhere. That is why
//! a lint pointer is spelled `OperationSnapshot#/properties/status`: the type, then the pointer
//! `contracts.toml` uses.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::check::Finding;
use crate::metadata::Metadata;
use crate::registry;

/// Justifications that are not justifications (L7), compared case-insensitively after trimming.
pub const PLACEHOLDER_JUSTIFICATIONS: &[&str] = &["todo", "n/a", "na", "none", "see above", "-"];

/// L8: the property name that means "envelope major", and the envelope types allowed to declare
/// it.
///
/// ADR-0002 gives the repository two version axes: `event_type`'s `.v<major>` is the payload
/// major, and `schema_version` is the envelope major. They describe different objects, so a second
/// property with the same name anywhere else — most concretely the `schema_version: u32` that
/// `ARCHITECTURE.md` S6.1 puts inside `Document` — would put two same-named integers in one
/// message that look like they must agree and do not. The two axes are kept apart by a name that
/// only an envelope may use, checked here rather than asked for in prose.
pub const ENVELOPE_MAJOR_PROPERTY: (&str, &[&str]) =
    ("schema_version", &["CommandEnvelope", "EventEnvelope"]);

/// The waiver / governance key of a property: `TypeName#/properties/name`.
#[must_use]
pub fn key(type_name: &str, pointer: &str) -> String {
    format!("{type_name}#{pointer}")
}

/// Resolves a governance pointer to the identity used by the lint.
///
/// Most entries are relative to their contract root. A reusable nested type instead names its
/// declaring type explicitly, for example `AiArchiveProvenance#/properties/imported_at`, so one
/// declaration governs the property wherever the type appears.
fn governance_key(root_type: &str, pointer: &str) -> String {
    if pointer.contains('#') {
        pointer.to_owned()
    } else {
        key(root_type, pointer)
    }
}

/// Runs L1–L8 over the in-memory generated schemas.
#[must_use]
pub fn run(metadata: &Metadata, generated: &BTreeMap<PathBuf, String>) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut waivers: BTreeMap<String, &str> = BTreeMap::new();
    let mut governed: BTreeSet<String> = BTreeSet::new();

    for contract in &metadata.contracts {
        let Some(declared) = contract.root_types.first() else {
            continue;
        };
        let type_name = registry::short_name(&declared.rust_path);
        for waiver in &contract.vague_field_waivers {
            waivers.insert(
                governance_key(type_name, &waiver.pointer),
                waiver.justification.as_str(),
            );
            let trimmed = waiver.justification.trim();
            let placeholder = PLACEHOLDER_JUSTIFICATIONS
                .iter()
                .any(|candidate| trimmed.eq_ignore_ascii_case(candidate));
            if trimmed.chars().count() < metadata.lint.min_waiver_justification_chars || placeholder
            {
                findings.push(Finding::Lint {
                    rule: "L7",
                    pointer: governance_key(type_name, &waiver.pointer),
                    detail: format!(
                        "justification must be a real one of at least {} characters",
                        metadata.lint.min_waiver_justification_chars
                    ),
                });
            }
        }
        for field in &contract.fields {
            governed.insert(governance_key(type_name, &field.pointer));
        }
        if let Some(event) = contract.event.as_ref()
            && let Ok(parsed) = ratatoskr_event_envelope::EventType::parse(&event.event_type)
            && !parsed.action_looks_past_tense()
        {
            findings.push(Finding::Lint {
                rule: "L6",
                pointer: event.event_type.clone(),
                detail: format!(
                    "action segment {:?} does not look past tense; an event is a completed fact",
                    parsed.action()
                ),
            });
        }
    }

    for (type_name, subject) in declared_types(generated) {
        lint_type(
            &type_name,
            &subject,
            metadata,
            &waivers,
            &governed,
            &mut findings,
        );
    }
    findings.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
    findings.dedup_by(|left, right| left.sort_key() == right.sort_key());
    findings
}

/// One type under lint, with the `$defs` its `$ref`s resolve against.
#[derive(Debug, Clone)]
pub struct TypeUnderLint {
    /// The type's own schema object.
    pub schema: serde_json::Value,
    /// The `$defs` of the document the type was found in.
    pub defs: serde_json::Map<String, serde_json::Value>,
}

/// Every named type reachable from the generated schemas: each file's root, keyed by its `title`,
/// plus every `$defs` member, keyed by its definition name.
///
/// A type inlined into several documents is linted once; the documents are byte-identical for it
/// by construction, because each is built from the same Rust type.
#[must_use]
pub fn declared_types(generated: &BTreeMap<PathBuf, String>) -> BTreeMap<String, TypeUnderLint> {
    let mut declared = BTreeMap::new();
    for body in generated.values() {
        let Ok(document) = serde_json::from_str::<serde_json::Value>(body) else {
            continue;
        };
        let defs = document
            .get("$defs")
            .and_then(serde_json::Value::as_object)
            .cloned()
            .unwrap_or_default();
        if let Some(title) = document.get("title").and_then(serde_json::Value::as_str) {
            declared.insert(
                title.to_owned(),
                TypeUnderLint {
                    schema: document.clone(),
                    defs: defs.clone(),
                },
            );
        }
        for (name, definition) in &defs {
            declared.insert(
                name.clone(),
                TypeUnderLint {
                    schema: definition.clone(),
                    defs: defs.clone(),
                },
            );
        }
    }
    declared
}

/// Lints every property one type declares.
fn lint_type(
    type_name: &str,
    subject: &TypeUnderLint,
    metadata: &Metadata,
    waivers: &BTreeMap<String, &str>,
    governed: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    let Some(properties) = subject
        .schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    let defs = Some(&subject.defs);
    for (name, property) in properties {
        let pointer = format!("/properties/{name}");
        let identity = key(type_name, &pointer);
        let resolved = resolve(property, defs);

        if metadata
            .lint
            .banned_property_names
            .iter()
            .any(|banned| banned == name)
            && !waivers.contains_key(&identity)
        {
            findings.push(Finding::Lint {
                rule: "L1",
                pointer: identity.clone(),
                detail: format!(
                    "{name:?} is in [lint].banned_property_names; add a \
                     [[contract.vague_field_waiver]] with a real justification, or rename it"
                ),
            });
        }

        let (reserved_name, reserved_owners) = ENVELOPE_MAJOR_PROPERTY;
        if name == reserved_name && !reserved_owners.contains(&type_name) {
            findings.push(Finding::Lint {
                rule: "L8",
                pointer: identity.clone(),
                detail: format!(
                    "{name:?} is reserved for the envelope major on {reserved_owners:?} (ADR-0002); \
                     a payload that needs its own version names it after what it versions, e.g. \
                     `document_ir_version`"
                ),
            });
        }

        let declared_type = resolved.get("type").and_then(serde_json::Value::as_str);
        if declared_type == Some("integer")
            && !metadata
                .lint
                .required_numeric_suffixes
                .iter()
                .any(|suffix| name.ends_with(suffix))
        {
            findings.push(Finding::Lint {
                rule: "L2",
                pointer: identity.clone(),
                detail: format!(
                    "integer property {name:?} carries no unit suffix; one of {:?} is required",
                    metadata.lint.required_numeric_suffixes
                ),
            });
        }
        if declared_type == Some("number") {
            findings.push(Finding::Lint {
                rule: "L3",
                pointer: identity.clone(),
                detail: format!(
                    "{name:?} is a floating-point property; floats make canonical serialization \
                     platform-dependent and are banned outright"
                ),
            });
        }

        if resolved.get("format").and_then(serde_json::Value::as_str) == Some("date-time") {
            lint_timestamp(name, &identity, metadata, governed, findings);
        }

        let described = property
            .get("description")
            .or_else(|| resolved.get("description"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|text| !text.trim().is_empty());
        if !described {
            findings.push(Finding::Lint {
                rule: "L5",
                pointer: identity,
                detail: format!("{name:?} has no description; write the rustdoc"),
            });
        }
    }
}

/// L4, both branches: a `format: date-time` property must be in the timestamp vocabulary **and**
/// carry a `[[contract.field]]` entry naming its clock authority. Both are reported in one run, so
/// an operator is not sent round the fix-and-rerun loop twice for one property.
fn lint_timestamp(
    name: &str,
    identity: &str,
    metadata: &Metadata,
    governed: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    if !metadata
        .lint
        .timestamp_property_names
        .iter()
        .any(|known| known == name)
    {
        findings.push(Finding::Lint {
            rule: "L4",
            pointer: identity.to_owned(),
            detail: format!(
                "{name:?} is a date-time property outside [lint].timestamp_property_names"
            ),
        });
    }
    if !governed.contains(identity) {
        findings.push(Finding::Lint {
            rule: "L4",
            pointer: identity.to_owned(),
            detail: format!(
                "{name:?} is a date-time property with no [[contract.field]] entry declaring its \
                 clock authority"
            ),
        });
    }
}

/// A property's effective schema: itself, merged over the single local `$ref` it may carry.
///
/// A property whose type is a newtype is emitted as `{"$ref": "#/$defs/EntityRef", …}`, so
/// `type` and `format` live in the definition. Without this resolution L2, L3 and L4 could never
/// fire on the very properties they exist to police. The property's own members win.
#[must_use]
pub fn resolve(
    property: &serde_json::Value,
    defs: Option<&serde_json::Map<String, serde_json::Value>>,
) -> serde_json::Value {
    let mut merged = property.clone();
    let candidates: Vec<&serde_json::Value> = match property.get("$ref") {
        Some(reference) => vec![reference],
        None => property
            .get("anyOf")
            .or_else(|| property.get("oneOf"))
            .and_then(serde_json::Value::as_array)
            .map(|branches| {
                branches
                    .iter()
                    .filter_map(|branch| branch.get("$ref"))
                    .collect()
            })
            .unwrap_or_default(),
    };
    for candidate in candidates {
        let Some(name) = candidate
            .as_str()
            .and_then(|reference| reference.strip_prefix("#/$defs/"))
        else {
            continue;
        };
        let Some(definition) = defs.and_then(|defs| defs.get(name)) else {
            continue;
        };
        let (Some(target), Some(source)) = (merged.as_object_mut(), definition.as_object()) else {
            continue;
        };
        for (member, value) in source {
            target
                .entry(member.clone())
                .or_insert_with(|| value.clone());
        }
    }
    merged
}
