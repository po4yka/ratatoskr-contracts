//! The compatibility classifier: what changed between two generated schemas, and whether it
//! breaks a deployed consumer.
//!
//! Golden-snapshot drift proves only that *something* changed. `docs/TESTING.md` requires a test
//! that demonstrates detection of a deliberate **breaking** change, which needs a classifier that
//! knows this repository's policy. It is deliberately small — a recursive descent keyed by JSON
//! pointer, no external schema-diff dependency — because these rules must be readable and
//! testable here.
//!
//! # Stated limit
//!
//! The classifier is **structural**. `AGENTS.md` lists "changing a field's meaning, unit,
//! authority, or nullability" as breaking, and a meaning change is invisible to any diff tool.
//! Only a moved `source_digest` plus human review catches it. A green `compat` run is never a
//! semantic guarantee.

use std::collections::BTreeSet;

use crate::provenance::PROVENANCE_KEY;

/// How a single difference affects a deployed consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// A deployed consumer keeps working.
    Compatible,
    /// Documentation only; no consumer behaviour changes.
    Informational,
    /// A deployed consumer breaks.
    Breaking,
    /// The rule table does not model this difference. Treated as breaking.
    NeedsReview,
}

impl Class {
    /// `true` when the CLI must exit non-zero for this class.
    #[must_use]
    pub fn is_blocking(self) -> bool {
        matches!(self, Self::Breaking | Self::NeedsReview)
    }

    /// The lowercase token used in text and JSON reports.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::Informational => "informational",
            Self::Breaking => "breaking",
            Self::NeedsReview => "needs-review",
        }
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl serde::Serialize for Class {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// One classified difference.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Finding {
    /// How the difference affects a deployed consumer.
    pub class: Class,
    /// The rule identifier, e.g. `property.removed`.
    pub rule: &'static str,
    /// JSON pointer of the differing node, relative to the schema document root.
    pub pointer: String,
    /// What the baseline said, when the rule has a meaningful before value.
    pub baseline: Option<serde_json::Value>,
    /// What the current document says, when the rule has a meaningful after value.
    pub current: Option<serde_json::Value>,
    /// A one-line explanation for a human reading the report.
    pub message: String,
}

impl Finding {
    /// The `(pointer, rule)` ordering key the report is sorted by, so a committed report diffs
    /// cleanly.
    #[must_use]
    pub fn sort_key(&self) -> (&str, &str) {
        (self.pointer.as_str(), self.rule)
    }
}

impl std::fmt::Display for Finding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:<14} {:<32} {}",
            self.class, self.rule, self.pointer
        )?;
        write!(formatter, "\n               {}", self.message)
    }
}

/// The output shape of `contractsc compat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ReportFormat {
    /// One human-readable block per finding.
    Text,
    /// A JSON array, sorted, suitable for committing and diffing.
    Json,
}

/// Classifies every difference between two generated schema documents.
///
/// Fails **closed**: anything the rule table does not model is [`Class::NeedsReview`], which the
/// CLI treats as breaking. A classifier that silently passes what it does not understand is worse
/// than none.
///
/// `x-ratatoskr-provenance` and `$id` are stripped from both sides first, so a generator-version
/// bump is never reported as a contract change.
#[must_use]
pub fn classify(old: &serde_json::Value, new: &serde_json::Value) -> Vec<Finding> {
    let mut findings = Vec::new();
    compare(&strip(old), &strip(new), "", &mut findings);
    findings.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
    findings
}

/// `true` when any finding blocks the change.
#[must_use]
pub fn is_blocking(findings: &[Finding]) -> bool {
    findings.iter().any(|finding| finding.class.is_blocking())
}

/// Renders a classified report.
///
/// # Errors
///
/// Propagates a `serde_json` serialization failure, which cannot occur for these value types.
pub fn report(findings: &[Finding], format: ReportFormat) -> Result<String, serde_json::Error> {
    match format {
        ReportFormat::Json => {
            let mut rendered = serde_json::to_string_pretty(findings)?;
            rendered.push('\n');
            Ok(rendered)
        }
        ReportFormat::Text => {
            if findings.is_empty() {
                return Ok("compatible: no contract difference\n".to_owned());
            }
            let mut rendered = String::new();
            for finding in findings {
                rendered.push_str(&finding.to_string());
                rendered.push('\n');
            }
            Ok(rendered)
        }
    }
}

/// Removes the members that describe the generator rather than the contract.
fn strip(document: &serde_json::Value) -> serde_json::Value {
    let mut stripped = document.clone();
    if let Some(members) = stripped.as_object_mut() {
        members.remove(PROVENANCE_KEY);
        members.remove("$id");
    }
    stripped
}

/// Keywords the rule table models. Anything else that differs is `unclassified`.
const MODELLED: &[&str] = &[
    "$comment",
    "$defs",
    "$ref",
    "$schema",
    "additionalProperties",
    "allOf",
    "anyOf",
    "const",
    "description",
    "enum",
    "examples",
    "format",
    "items",
    "maxItems",
    "maxLength",
    "maximum",
    "minItems",
    "minLength",
    "minimum",
    "oneOf",
    "pattern",
    "prefixItems",
    "properties",
    "required",
    "title",
    "type",
    "x-ratatoskr-unknown-policy",
];

/// Every upper bound, paired with the direction that relaxes it.
const UPPER_BOUNDS: &[&str] = &["maxItems", "maxLength", "maximum"];

/// Every lower bound.
const LOWER_BOUNDS: &[&str] = &["minItems", "minLength", "minimum"];

/// Recursive descent over one node pair.
fn compare(
    old: &serde_json::Value,
    new: &serde_json::Value,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    if old == new {
        return;
    }
    let (Some(old_members), Some(new_members)) = (old.as_object(), new.as_object()) else {
        findings.push(unclassified(pointer, old, new, "the node changed shape"));
        return;
    };

    compare_properties(old_members, new_members, pointer, findings);
    compare_required(old_members, new_members, pointer, findings);
    compare_type(old_members, new_members, pointer, findings);
    compare_scalars(old_members, new_members, pointer, findings);
    compare_bounds(old_members, new_members, pointer, findings);
    compare_enum(old_members, new_members, pointer, findings);
    compare_additional_properties(old_members, new_members, pointer, findings);
    compare_unknown_policy(old_members, new_members, pointer, findings);
    compare_annotations(old_members, new_members, pointer, findings);
    recurse_children(old_members, new_members, pointer, findings);
    compare_residual(old_members, new_members, pointer, findings);
}

/// `property.removed`, `property.added.required`, `property.added.optional`.
fn compare_properties(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    let empty = serde_json::Map::new();
    let old_properties = object_or(old.get("properties"), &empty);
    let new_properties = object_or(new.get("properties"), &empty);
    let new_required = string_set(new.get("required"));

    for (name, baseline) in old_properties {
        if !new_properties.contains_key(name) {
            findings.push(Finding {
                class: Class::Breaking,
                rule: "property.removed",
                pointer: child(pointer, "properties", name),
                baseline: Some(baseline.clone()),
                current: None,
                message: format!("{name:?} was removed; a consumer reading it now sees nothing"),
            });
        }
    }
    for (name, current) in new_properties {
        if old_properties.contains_key(name) {
            continue;
        }
        let required = new_required.contains(name.as_str());
        findings.push(Finding {
            class: if required {
                Class::Breaking
            } else {
                Class::Compatible
            },
            rule: if required {
                "property.added.required"
            } else {
                "property.added.optional"
            },
            pointer: child(pointer, "properties", name),
            baseline: None,
            current: Some(current.clone()),
            message: if required {
                format!("{name:?} is new and required; every deployed producer omits it")
            } else {
                format!("{name:?} is new and optional")
            },
        });
    }
}

/// `required.added`, `required.removed`. A property that is new *and* required is reported once,
/// by `property.added.required`.
fn compare_required(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    let empty = serde_json::Map::new();
    let old_properties = object_or(old.get("properties"), &empty);
    let old_required = string_set(old.get("required"));
    let new_required = string_set(new.get("required"));

    for name in new_required.difference(&old_required) {
        if !old_properties.contains_key(*name) {
            continue;
        }
        findings.push(Finding {
            class: Class::Breaking,
            rule: "required.added",
            pointer: format!("{pointer}/required"),
            baseline: None,
            current: Some(serde_json::json!(name)),
            message: format!(
                "{name:?} became required; a deployed producer that omits it is now invalid"
            ),
        });
    }
    for name in old_required.difference(&new_required) {
        findings.push(Finding {
            class: Class::Compatible,
            rule: "required.removed",
            pointer: format!("{pointer}/required"),
            baseline: Some(serde_json::json!(name)),
            current: None,
            message: format!("{name:?} is no longer required"),
        });
    }
}

/// `type.widened`, `type.changed`, and `$ref` retargeting.
fn compare_type(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    let old_type = type_set(old.get("type"));
    let new_type = type_set(new.get("type"));
    if old_type != new_type {
        let widened = old_type.is_subset(&new_type);
        findings.push(Finding {
            class: if widened {
                Class::Compatible
            } else {
                Class::Breaking
            },
            rule: if widened {
                "type.widened"
            } else {
                "type.changed"
            },
            pointer: format!("{pointer}/type"),
            baseline: old.get("type").cloned(),
            current: new.get("type").cloned(),
            message: if widened {
                "the accepted type set grew".to_owned()
            } else {
                "the accepted type set changed; a deployed consumer decodes the wrong thing"
                    .to_owned()
            },
        });
    }
    if old.get("$ref") != new.get("$ref") {
        findings.push(Finding {
            class: Class::Breaking,
            rule: "type.changed",
            pointer: format!("{pointer}/$ref"),
            baseline: old.get("$ref").cloned(),
            current: new.get("$ref").cloned(),
            message: "the referenced definition changed".to_owned(),
        });
    }
}

/// `format.changed`, `pattern.changed`, `const.changed`.
fn compare_scalars(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    for (keyword, rule, message) in [
        ("format", "format.changed", "the semantic format changed"),
        (
            "pattern",
            "pattern.changed",
            "the accepted lexical form changed; a relaxation is not machine-provable, so every \
             pattern change is breaking",
        ),
        ("const", "const.changed", "the only accepted value changed"),
    ] {
        if old.get(keyword) != new.get(keyword) {
            findings.push(Finding {
                class: Class::Breaking,
                rule,
                pointer: format!("{pointer}/{keyword}"),
                baseline: old.get(keyword).cloned(),
                current: new.get(keyword).cloned(),
                message: message.to_owned(),
            });
        }
    }
}

/// `bound.relaxed`, `bound.tightened`.
fn compare_bounds(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    for keyword in UPPER_BOUNDS.iter().chain(LOWER_BOUNDS) {
        let (before, after) = (old.get(*keyword), new.get(*keyword));
        if before == after {
            continue;
        }
        let upper = UPPER_BOUNDS.contains(keyword);
        let relaxed = match (
            before.and_then(serde_json::Value::as_i64),
            after.and_then(serde_json::Value::as_i64),
        ) {
            // Removing a bound always relaxes; adding one always tightens.
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (Some(before), Some(after)) => {
                if upper {
                    after > before
                } else {
                    after < before
                }
            }
            (None, None) => continue,
        };
        findings.push(Finding {
            class: if relaxed {
                Class::Compatible
            } else {
                Class::Breaking
            },
            rule: if relaxed {
                "bound.relaxed"
            } else {
                "bound.tightened"
            },
            pointer: format!("{pointer}/{keyword}"),
            baseline: before.cloned(),
            current: after.cloned(),
            message: if relaxed {
                format!("{keyword} accepts strictly more than before")
            } else {
                format!("{keyword} rejects values a deployed producer still emits")
            },
        });
    }
}

/// `enum.value.added`, `enum.value.removed`.
///
/// Both are breaking here: `AGENTS.md` classifies an enum addition as breaking when consumers are
/// exhaustive, and every consumer of an `OperationStatus` is (decision D6).
///
/// Two spellings of one concept are handled. A bare `enum` array is the obvious one. schemars
/// renders a **documented** unit-variant Rust enum as a `oneOf` of `const` branches instead — one
/// branch per variant so each keeps its own rustdoc `description` — and that is the spelling every
/// closed enum in this repository actually has, so it must classify identically.
fn compare_enum(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    if let (Some((keyword, before)), Some((_, after))) =
        (const_enumeration(old), const_enumeration(new))
    {
        diff_values(&before, &after, &format!("{pointer}/{keyword}"), findings);
        return;
    }

    let (Some(before), Some(after)) = (old.get("enum"), new.get("enum")) else {
        if old.get("enum") != new.get("enum") {
            findings.push(unclassified(
                &format!("{pointer}/enum"),
                old.get("enum").unwrap_or(&serde_json::Value::Null),
                new.get("enum").unwrap_or(&serde_json::Value::Null),
                "an enum constraint appeared or disappeared",
            ));
        }
        return;
    };
    let (Some(before), Some(after)) = (before.as_array(), after.as_array()) else {
        return;
    };
    let before: Vec<serde_json::Value> = before.clone();
    let after: Vec<serde_json::Value> = after.clone();
    diff_values(&before, &after, &format!("{pointer}/enum"), findings);
}

/// The classified difference between two sets of legal values.
fn diff_values(
    before: &[serde_json::Value],
    after: &[serde_json::Value],
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    for value in before {
        if !after.contains(value) {
            findings.push(Finding {
                class: Class::Breaking,
                rule: "enum.value.removed",
                pointer: pointer.to_owned(),
                baseline: Some(value.clone()),
                current: None,
                message: format!("{value} is no longer legal; a deployed producer still emits it"),
            });
        }
    }
    for value in after {
        if !before.contains(value) {
            findings.push(Finding {
                class: Class::Breaking,
                rule: "enum.value.added",
                pointer: pointer.to_owned(),
                baseline: None,
                current: Some(value.clone()),
                message: format!(
                    "{value} is new; consumers here match exhaustively and cannot render it"
                ),
            });
        }
    }
}

/// The combinator keyword and the legal values of a node that is a `oneOf`/`anyOf` of `const`
/// branches, i.e. an enumeration spelled out one documented variant at a time.
fn const_enumeration(
    node: &serde_json::Map<String, serde_json::Value>,
) -> Option<(&'static str, Vec<serde_json::Value>)> {
    for keyword in ["oneOf", "anyOf"] {
        let Some(branches) = node.get(keyword).and_then(serde_json::Value::as_array) else {
            continue;
        };
        if branches.is_empty() {
            continue;
        }
        let values: Vec<serde_json::Value> = branches
            .iter()
            .filter_map(|branch| branch.get("const").cloned())
            .collect();
        if values.len() == branches.len() {
            return Some((keyword, values));
        }
    }
    None
}

/// `additionalProperties.relaxed`, `additionalProperties.restricted`.
fn compare_additional_properties(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    let (before, after) = (
        old.get("additionalProperties"),
        new.get("additionalProperties"),
    );
    if before == after {
        return;
    }
    let pointer = format!("{pointer}/additionalProperties");
    // Absent means "true" in JSON Schema, so an absent baseline is an open document.
    match (
        before.and_then(serde_json::Value::as_bool).unwrap_or(true),
        after.and_then(serde_json::Value::as_bool),
    ) {
        (false, Some(true)) => findings.push(Finding {
            class: Class::Compatible,
            rule: "additionalProperties.relaxed",
            pointer,
            baseline: before.cloned(),
            current: after.cloned(),
            message: "unknown members are now accepted".to_owned(),
        }),
        (true, Some(false)) => findings.push(Finding {
            class: Class::Breaking,
            rule: "additionalProperties.restricted",
            pointer,
            baseline: before.cloned(),
            current: after.cloned(),
            message: "unknown members are now rejected; every future additive field breaks"
                .to_owned(),
        }),
        _ => findings.push(unclassified(
            &pointer,
            before.unwrap_or(&serde_json::Value::Null),
            after.unwrap_or(&serde_json::Value::Null),
            "additionalProperties changed in a way the rule table does not model",
        )),
    }
}

/// `unknown_policy.changed`.
fn compare_unknown_policy(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    let keyword = "x-ratatoskr-unknown-policy";
    let (before, after) = (old.get(keyword), new.get(keyword));
    if before == after {
        return;
    }
    let pointer = format!("{pointer}/{keyword}");
    let tightened = before.and_then(serde_json::Value::as_str) == Some("preserve")
        && after.and_then(serde_json::Value::as_str) == Some("reject");
    if tightened {
        findings.push(Finding {
            class: Class::Breaking,
            rule: "unknown_policy.changed",
            pointer,
            baseline: before.cloned(),
            current: after.cloned(),
            message: "the contract stopped preserving unknown members; expand/migrate/contract no \
                      longer works"
                .to_owned(),
        });
    } else {
        findings.push(unclassified(
            &pointer,
            before.unwrap_or(&serde_json::Value::Null),
            after.unwrap_or(&serde_json::Value::Null),
            "the unknown-member policy changed in a way the rule table does not model",
        ));
    }
}

/// `annotation.changed` — documentation only.
fn compare_annotations(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    for keyword in ["title", "description", "examples"] {
        if old.get(keyword) != new.get(keyword) {
            findings.push(Finding {
                class: Class::Informational,
                rule: "annotation.changed",
                pointer: format!("{pointer}/{keyword}"),
                baseline: old.get(keyword).cloned(),
                current: new.get(keyword).cloned(),
                message: format!("{keyword} changed; no consumer behaviour depends on it"),
            });
        }
    }
}

/// Descends into `properties`, `$defs`, `items`, `prefixItems` and each combinator branch.
fn recurse_children(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    let empty = serde_json::Map::new();
    for keyword in ["properties", "$defs"] {
        let before = object_or(old.get(keyword), &empty);
        let after = object_or(new.get(keyword), &empty);
        for (name, baseline) in before {
            match after.get(name) {
                Some(current) => {
                    compare(baseline, current, &child(pointer, keyword, name), findings);
                }
                // A removed property is already `property.removed`; a removed definition is not
                // modelled, and failing closed is the point.
                None if keyword == "$defs" => findings.push(unclassified(
                    &child(pointer, keyword, name),
                    baseline,
                    &serde_json::Value::Null,
                    "a definition disappeared",
                )),
                None => {}
            }
        }
        if keyword == "$defs" {
            for (name, current) in after {
                if !before.contains_key(name) {
                    findings.push(unclassified(
                        &child(pointer, keyword, name),
                        &serde_json::Value::Null,
                        current,
                        "a definition appeared",
                    ));
                }
            }
        }
    }

    if let (Some(before), Some(after)) = (old.get("items"), new.get("items")) {
        compare(before, after, &format!("{pointer}/items"), findings);
    }

    // A `oneOf`/`anyOf` of `const` branches whose value sets differ is an enum change, already
    // classified by `compare_enum`; descending into it would report the same fact twice as a
    // branch-count mismatch.
    let enumeration_changed = match (const_enumeration(old), const_enumeration(new)) {
        (Some((_, before)), Some((_, after))) => before != after,
        _ => false,
    };

    for keyword in ["anyOf", "oneOf", "allOf", "prefixItems"] {
        if enumeration_changed && (keyword == "anyOf" || keyword == "oneOf") {
            continue;
        }
        let (before, after) = (
            old.get(keyword).and_then(serde_json::Value::as_array),
            new.get(keyword).and_then(serde_json::Value::as_array),
        );
        let (Some(before), Some(after)) = (before, after) else {
            continue;
        };
        if before.len() != after.len() {
            findings.push(unclassified(
                &format!("{pointer}/{keyword}"),
                &serde_json::json!(before.len()),
                &serde_json::json!(after.len()),
                "the number of branches changed",
            ));
            continue;
        }
        for (index, (baseline, current)) in before.iter().zip(after).enumerate() {
            compare(
                baseline,
                current,
                &format!("{pointer}/{keyword}/{index}"),
                findings,
            );
        }
    }
}

/// `unclassified` — every keyword the rule table does not model, compared verbatim.
fn compare_residual(
    old: &serde_json::Map<String, serde_json::Value>,
    new: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
    findings: &mut Vec<Finding>,
) {
    let mut keywords: BTreeSet<&str> = BTreeSet::new();
    keywords.extend(old.keys().map(String::as_str));
    keywords.extend(new.keys().map(String::as_str));
    for keyword in keywords {
        if MODELLED.contains(&keyword) {
            continue;
        }
        let (before, after) = (old.get(keyword), new.get(keyword));
        if before != after {
            findings.push(unclassified(
                &format!("{pointer}/{keyword}"),
                before.unwrap_or(&serde_json::Value::Null),
                after.unwrap_or(&serde_json::Value::Null),
                &format!("{keyword:?} is a keyword no compatibility rule models"),
            ));
        }
    }
}

/// Builds the fail-closed finding.
fn unclassified(
    pointer: &str,
    baseline: &serde_json::Value,
    current: &serde_json::Value,
    message: &str,
) -> Finding {
    Finding {
        class: Class::NeedsReview,
        rule: "unclassified",
        pointer: pointer.to_owned(),
        baseline: Some(baseline.clone()),
        current: Some(current.clone()),
        message: format!("{message}; review it by hand"),
    }
}

/// The JSON pointer of a named child, RFC 6901 escaped.
fn child(pointer: &str, keyword: &str, name: &str) -> String {
    format!(
        "{pointer}/{keyword}/{}",
        name.replace('~', "~0").replace('/', "~1")
    )
}

/// A member as an object, or the supplied empty map.
fn object_or<'a>(
    value: Option<&'a serde_json::Value>,
    empty: &'a serde_json::Map<String, serde_json::Value>,
) -> &'a serde_json::Map<String, serde_json::Value> {
    value
        .and_then(serde_json::Value::as_object)
        .unwrap_or(empty)
}

/// A `required` array as a set of names.
fn string_set(value: Option<&serde_json::Value>) -> BTreeSet<&str> {
    value
        .and_then(serde_json::Value::as_array)
        .map(|names| names.iter().filter_map(serde_json::Value::as_str).collect())
        .unwrap_or_default()
}

/// A `type` keyword as a set, whether it is a string or an array of strings.
fn type_set(value: Option<&serde_json::Value>) -> BTreeSet<&str> {
    match value {
        Some(serde_json::Value::String(single)) => BTreeSet::from([single.as_str()]),
        Some(serde_json::Value::Array(many)) => {
            many.iter().filter_map(serde_json::Value::as_str).collect()
        }
        _ => BTreeSet::new(),
    }
}
