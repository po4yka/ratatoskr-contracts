//! The two normalization passes that run between schema construction and rendering.

use crate::GenError;

/// Recursively sorts every `required` array lexicographically.
///
/// Required, not cosmetic: schemars emits `required` in **struct declaration order**, so without
/// this pass a pure field reorder in Rust produces a schema diff and a moved `source_digest`.
/// Test `G-4` asserts the result over every committed artifact.
pub fn sort_required(schema: &mut schemars::Schema) {
    sort_required_object(schema.ensure_object());
}

/// Rejects any number in the schema that is not an integer.
///
/// Floating-point formatting is the one remaining way two machines can render the same schema
/// differently, and `AGENTS.md` principle 6 has no use for a float in a wire contract anyway.
///
/// # Errors
///
/// [`GenError::NonIntegerNumber`] naming the JSON pointer of the first offending number.
pub fn assert_no_floats(schema: &schemars::Schema) -> Result<(), GenError> {
    assert_no_floats_value("", schema.as_value())
}

/// `sort_required` over one JSON object.
fn sort_required_object(members: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(serde_json::Value::Array(names)) = members.get_mut("required") {
        names.sort_by(|left, right| match (left.as_str(), right.as_str()) {
            (Some(left), Some(right)) => left.cmp(right),
            _ => core::cmp::Ordering::Equal,
        });
    }
    for member in members.values_mut() {
        sort_required_value(member);
    }
}

/// `sort_required` over a bare `serde_json::Value`.
fn sort_required_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(members) => sort_required_object(members),
        serde_json::Value::Array(items) => {
            for item in items {
                sort_required_value(item);
            }
        }
        _ => {}
    }
}

/// `assert_no_floats` over a bare `serde_json::Value`, tracking the JSON pointer.
fn assert_no_floats_value(pointer: &str, value: &serde_json::Value) -> Result<(), GenError> {
    match value {
        serde_json::Value::Number(number) => {
            if number.is_f64() {
                return Err(GenError::NonIntegerNumber {
                    pointer: pointer.to_owned(),
                    value: number.to_string(),
                });
            }
            Ok(())
        }
        serde_json::Value::Object(members) => {
            for (key, member) in members {
                assert_no_floats_value(&format!("{pointer}/{}", escape(key)), member)?;
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                assert_no_floats_value(&format!("{pointer}/{index}"), item)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// RFC 6901 escaping for one JSON pointer segment.
fn escape(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}
