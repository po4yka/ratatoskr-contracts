//! Deliberate breaking-change detection — tests B-1 to B-18.
//!
//! `docs/TESTING.md` hard requirement: "A test must demonstrate detection of a deliberate breaking
//! change." Golden-snapshot drift alone proves only that *something* changed, not that it was
//! breaking.
//!
//! Every case loads a **real committed schema**, applies one deliberate mutation in memory and
//! runs the real classifier. No mutation is ever written into the repository, so the cases cost
//! nothing in review noise and cannot rot out of sync with the schemas they mutate.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::path::{Path, PathBuf};

use ratatoskr_contractsc::compat::{self, Class};

/// The event envelope, the contract with the richest field set.
const ENVELOPE: &str = "schemas/json-schema/core/event-envelope.v1.schema.json";

/// The operation snapshot, the contract carrying the closed lifecycle enum.
const SNAPSHOT: &str = "schemas/json-schema/platform/operation-snapshot.v1.schema.json";

/// The repository root, derived from the compiled-in manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

/// One committed schema, parsed.
fn committed(relative: &str) -> serde_json::Value {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is committed: {error}", path.display()));
    serde_json::from_str(&text).expect("a generated schema is JSON")
}

/// A mutable object member, panicking with the pointer when the shape is not what the case
/// assumes — which is how a case that has rotted announces itself.
fn at<'a>(
    document: &'a mut serde_json::Value,
    pointer: &str,
) -> &'a mut serde_json::Map<String, serde_json::Value> {
    document
        .pointer_mut(pointer)
        .and_then(serde_json::Value::as_object_mut)
        .unwrap_or_else(|| panic!("{pointer} is an object in the committed schema"))
}

/// Runs the classifier and asserts exactly one finding matches `(rule, class)`, returning it.
fn expect_one(
    baseline: &serde_json::Value,
    current: &serde_json::Value,
    rule: &str,
    class: Class,
) -> compat::Finding {
    let findings = compat::classify(baseline, current);
    let matched: Vec<&compat::Finding> = findings
        .iter()
        .filter(|finding| finding.rule == rule && finding.class == class)
        .collect();
    assert_eq!(
        matched.len(),
        1,
        "expected exactly one {class} / {rule}; classifier said {findings:#?}"
    );
    assert_eq!(
        findings.len(),
        1,
        "expected the mutation to produce exactly one finding; got {findings:#?}"
    );
    findings.into_iter().next().expect("one finding")
}

/// B-1. A new **required** property breaks every deployed producer, which omits it.
#[test]
fn added_required_property_is_breaking() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/properties").insert(
        "retention_class".to_owned(),
        serde_json::json!({ "type": "string", "description": "How long to keep the record." }),
    );
    current
        .pointer_mut("/required")
        .and_then(serde_json::Value::as_array_mut)
        .expect("required is an array")
        .push(serde_json::json!("retention_class"));

    let finding = expect_one(
        &baseline,
        &current,
        "property.added.required",
        Class::Breaking,
    );
    assert_eq!(finding.pointer, "/properties/retention_class");
}

/// B-2. Making an existing optional property required breaks every deployed producer that omits
/// it.
#[test]
fn newly_required_property_is_breaking() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    require(&mut current, "tenant_id");

    let finding = expect_one(&baseline, &current, "required.added", Class::Breaking);
    assert_eq!(finding.pointer, "/required");
}

/// B-3. A removed property breaks every deployed consumer that reads it.
#[test]
fn removed_property_is_breaking() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/properties")
        .remove("causation_id")
        .expect("causation_id is a committed property");

    let finding = expect_one(&baseline, &current, "property.removed", Class::Breaking);
    assert_eq!(finding.pointer, "/properties/causation_id");
}

/// B-4. A changed JSON type makes every deployed consumer decode the wrong thing.
#[test]
fn changed_type_is_breaking() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/$defs/EnvelopeSchemaVersion")
        .insert("type".to_owned(), serde_json::json!("string"));

    let finding = expect_one(&baseline, &current, "type.changed", Class::Breaking);
    assert_eq!(finding.pointer, "/$defs/EnvelopeSchemaVersion/type");
}

/// B-5. A changed `pattern` is breaking unconditionally: a relaxation is not machine-provable.
#[test]
fn changed_pattern_is_breaking() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/$defs/EntityRef")
        .insert("pattern".to_owned(), serde_json::json!("^[a-z]+:[a-z]+$"));

    let finding = expect_one(&baseline, &current, "pattern.changed", Class::Breaking);
    assert_eq!(finding.pointer, "/$defs/EntityRef/pattern");
}

/// B-6. A tightened bound rejects values a deployed producer still emits.
#[test]
fn tightened_bound_is_breaking() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    let entity_ref = at(&mut current, "/$defs/EntityRef");
    assert_eq!(entity_ref.get("maxLength"), Some(&serde_json::json!(289)));
    entity_ref.insert("maxLength".to_owned(), serde_json::json!(64));

    let finding = expect_one(&baseline, &current, "bound.tightened", Class::Breaking);
    assert_eq!(finding.pointer, "/$defs/EntityRef/maxLength");
}

/// B-7. Closing a document to unknown members breaks every future additive field, which is the
/// whole expand/migrate/contract path.
#[test]
fn restricted_additional_properties_is_breaking() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "").insert("additionalProperties".to_owned(), serde_json::json!(false));

    let finding = expect_one(
        &baseline,
        &current,
        "additionalProperties.restricted",
        Class::Breaking,
    );
    assert_eq!(finding.pointer, "/additionalProperties");
}

/// B-8. Flipping the declared unknown-member policy from `preserve` to `reject` is breaking even
/// though `additionalProperties` did not move: the policy is what a Ratatoskr consumer reads.
#[test]
fn tightened_unknown_policy_is_breaking() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "").insert(
        "x-ratatoskr-unknown-policy".to_owned(),
        serde_json::json!("reject"),
    );

    let finding = expect_one(
        &baseline,
        &current,
        "unknown_policy.changed",
        Class::Breaking,
    );
    assert_eq!(finding.pointer, "/x-ratatoskr-unknown-policy");
}

/// B-9. Removing a lifecycle state breaks every deployed producer that still emits it.
#[test]
fn removed_enum_value_is_breaking() {
    let baseline = committed(SNAPSHOT);
    let mut current = baseline.clone();
    let branches = status_branches(&mut current);
    let before = branches.len();
    branches.retain(|branch| branch.get("const") != Some(&serde_json::json!("cancelled")));
    assert_eq!(
        branches.len(),
        before - 1,
        "cancelled is a committed status"
    );

    let finding = expect_one(&baseline, &current, "enum.value.removed", Class::Breaking);
    assert_eq!(finding.pointer, "/$defs/OperationStatus/oneOf");
    assert_eq!(finding.baseline, Some(serde_json::json!("cancelled")));
}

/// B-10. Adding a lifecycle state is breaking here, because consumers of a closed lifecycle enum
/// match exhaustively and cannot render a state they have never heard of (decision D6).
#[test]
fn added_enum_value_is_breaking() {
    let baseline = committed(SNAPSHOT);
    let mut current = baseline.clone();
    status_branches(&mut current).push(serde_json::json!({
        "description": "The work is suspended.",
        "type": "string",
        "const": "paused",
    }));

    let finding = expect_one(&baseline, &current, "enum.value.added", Class::Breaking);
    assert_eq!(finding.pointer, "/$defs/OperationStatus/oneOf");
    assert_eq!(finding.current, Some(serde_json::json!("paused")));
}

/// B-11. Moving the envelope's version gate is breaking: every deployed producer sends the old
/// constant.
#[test]
fn changed_const_is_breaking() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/$defs/EnvelopeSchemaVersion")
        .insert("const".to_owned(), serde_json::json!(2));

    let finding = expect_one(&baseline, &current, "const.changed", Class::Breaking);
    assert_eq!(finding.pointer, "/$defs/EnvelopeSchemaVersion/const");
}

/// B-12. The fail-closed property: a keyword no rule models is `needs-review`, which the CLI
/// treats as breaking. A classifier that silently passes what it does not understand is worse
/// than none.
#[test]
fn unmodelled_keyword_needs_review() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/properties/payload")
        .insert("contentEncoding".to_owned(), serde_json::json!("base64"));

    let finding = expect_one(&baseline, &current, "unclassified", Class::NeedsReview);
    assert_eq!(finding.pointer, "/properties/payload/contentEncoding");
    assert!(compat::is_blocking(&[finding]), "needs-review must block");
}

/// B-13. A new optional property is the sanctioned additive change.
#[test]
fn added_optional_property_is_compatible() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/properties").insert(
        "trace_id".to_owned(),
        serde_json::json!({ "type": "string", "description": "Distributed trace identity." }),
    );

    let finding = expect_one(
        &baseline,
        &current,
        "property.added.optional",
        Class::Compatible,
    );
    assert_eq!(finding.pointer, "/properties/trace_id");
    assert!(!compat::is_blocking(&[finding]));
}

/// B-14. Dropping a property from `required` is a relaxation: every deployed producer still sends
/// it, and every deployed consumer still reads it.
#[test]
fn removed_requirement_is_compatible() {
    // The baseline is the B-2 world, where `tenant_id` is required; the candidate is the committed
    // schema, where it is not.
    let current = committed(ENVELOPE);
    let mut baseline = current.clone();
    require(&mut baseline, "tenant_id");

    let finding = expect_one(&baseline, &current, "required.removed", Class::Compatible);
    assert_eq!(finding.pointer, "/required");
    assert!(!compat::is_blocking(&[finding]));
}

/// B-15. A relaxed bound accepts strictly more than before.
#[test]
fn relaxed_bound_is_compatible() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/$defs/EntityRef").insert("maxLength".to_owned(), serde_json::json!(512));

    let finding = expect_one(&baseline, &current, "bound.relaxed", Class::Compatible);
    assert_eq!(finding.pointer, "/$defs/EntityRef/maxLength");
    assert!(!compat::is_blocking(&[finding]));
}

/// B-16. A documentation edit changes no consumer behaviour.
#[test]
fn changed_description_is_informational() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "").insert(
        "description".to_owned(),
        serde_json::json!("A reworded but otherwise identical description."),
    );

    let finding = expect_one(
        &baseline,
        &current,
        "annotation.changed",
        Class::Informational,
    );
    assert_eq!(finding.pointer, "/description");
    assert!(!compat::is_blocking(&[finding]));
}

/// B-17. A generator bump is not a contract change: provenance and `$id` are stripped before
/// comparison, so `source_digest` stays the only signal that a contract moved.
#[test]
fn generator_version_bump_is_not_a_contract_change() {
    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/x-ratatoskr-provenance")
        .insert("generator_version".to_owned(), serde_json::json!("9.9.9"));
    assert_ne!(baseline, current, "the mutation landed");

    assert!(compat::classify(&baseline, &current).is_empty());
}

/// B-18. The **gate** fires, not merely the library: the real binary over a real mutated file
/// exits non-zero.
#[test]
fn the_cli_exits_non_zero_on_a_breaking_change() {
    let directory = std::env::temp_dir().join(format!(
        "ratatoskr-contractsc-compat-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("a temporary directory is creatable");

    let baseline = committed(ENVELOPE);
    let mut current = baseline.clone();
    at(&mut current, "/properties")
        .remove("causation_id")
        .expect("causation_id is a committed property");

    let old_path = directory.join("old.schema.json");
    let new_path = directory.join("new.schema.json");
    std::fs::write(
        &old_path,
        serde_json::to_string_pretty(&baseline).expect("serializable"),
    )
    .expect("writable");
    std::fs::write(
        &new_path,
        serde_json::to_string_pretty(&current).expect("serializable"),
    )
    .expect("writable");

    let breaking = std::process::Command::new(env!("CARGO_BIN_EXE_contractsc"))
        .args(["compat"])
        .arg(&old_path)
        .arg(&new_path)
        .output()
        .expect("the generator binary runs");
    assert!(
        !breaking.status.success(),
        "a removed property must fail the gate"
    );
    let report = String::from_utf8_lossy(&breaking.stdout);
    assert!(report.contains("property.removed"), "{report}");
    assert!(report.contains("breaking"), "{report}");

    let unchanged = std::process::Command::new(env!("CARGO_BIN_EXE_contractsc"))
        .args(["compat", "--format", "json"])
        .arg(&old_path)
        .arg(&old_path)
        .output()
        .expect("the generator binary runs");
    assert!(
        unchanged.status.success(),
        "an unchanged contract must pass the gate"
    );
    assert_eq!(String::from_utf8_lossy(&unchanged.stdout).trim(), "[]");

    let _ = std::fs::remove_dir_all(&directory);
}

/// Adds `name` to the document's top-level `required` array, keeping it sorted.
fn require(document: &mut serde_json::Value, name: &str) {
    let required = document
        .pointer_mut("/required")
        .and_then(serde_json::Value::as_array_mut)
        .expect("required is an array");
    assert!(
        !required.contains(&serde_json::json!(name)),
        "{name} is already required in the committed schema"
    );
    required.push(serde_json::json!(name));
    required.sort_by_key(|value| value.as_str().unwrap_or_default().to_owned());
}

/// The `oneOf` branches of the committed `OperationStatus` definition.
fn status_branches(document: &mut serde_json::Value) -> &mut Vec<serde_json::Value> {
    document
        .pointer_mut("/$defs/OperationStatus/oneOf")
        .and_then(serde_json::Value::as_array_mut)
        .expect("OperationStatus is a oneOf of const branches")
}
