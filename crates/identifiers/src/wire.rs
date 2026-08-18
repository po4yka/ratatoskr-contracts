//! Tolerant-envelope support: preserved unknown fields, loss detection, canonical JSON.

use std::collections::BTreeMap;

/// Additive fields this build does not know about, preserved verbatim so a consumer built against
/// an older contract re-emits everything a newer producer sent.
///
/// `README.md` Compatibility policy: "consumers ignore unknown additive fields".
/// `DOMAIN.md` invariant 6, "preserved" branch.
///
/// Flattening this into a struct makes the generated schema advertise `additionalProperties: true`
/// and `x-ratatoskr-unknown-policy: "preserve"` on the parent. `BTreeMap`, not `IndexMap`, so
/// preserved keys re-serialize in a deterministic sorted order.
///
/// A producer that wants strict rejection of its own typos asserts `extensions.is_empty()` in its
/// own test. There is no strict wire mode in m1–m4.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Extensions(BTreeMap<String, serde_json::Value>);

impl Extensions {
    /// An empty extension map, which serializes to nothing at all.
    #[must_use]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// `true` when the producer sent nothing this build did not understand.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The number of preserved keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The preserved keys, in the sorted order they re-serialize in.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }

    /// The preserved value for `key`, if any.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    /// Adds or replaces a preserved key, returning the value it displaced.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: serde_json::Value,
    ) -> Option<serde_json::Value> {
        self.0.insert(key.into(), value)
    }

    /// The whole preserved map.
    #[must_use]
    pub fn as_map(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.0
    }
}

impl schemars::JsonSchema for Extensions {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Extensions")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::Extensions"))
    }

    // Emits no title/description: schemars merges those into the flattening parent.
    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "additionalProperties": true,
            "x-ratatoskr-unknown-policy": "preserve",
        })
    }
}

/// JSON pointers present in `input` but absent from `roundtripped`.
///
/// Empty means the parse was lossless. Used by `contractsc` for every fixture and by consumers
/// that want a hard "did I drop anything?" assertion.
#[must_use]
pub fn dropped_field_pointers(
    input: &serde_json::Value,
    roundtripped: &serde_json::Value,
) -> Vec<String> {
    let mut dropped = Vec::new();
    collect_dropped("", input, roundtripped, &mut dropped);
    dropped
}

/// RFC 6901 escaping for one JSON pointer segment.
fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn collect_dropped(
    prefix: &str,
    input: &serde_json::Value,
    roundtripped: &serde_json::Value,
    dropped: &mut Vec<String>,
) {
    match input {
        serde_json::Value::Object(members) => {
            let survivors = roundtripped.as_object();
            for (key, value) in members {
                let pointer = format!("{prefix}/{}", escape_pointer_segment(key));
                match survivors.and_then(|object| object.get(key)) {
                    Some(survivor) => collect_dropped(&pointer, value, survivor, dropped),
                    None => dropped.push(pointer),
                }
            }
        }
        serde_json::Value::Array(items) => {
            let survivors = roundtripped.as_array();
            for (index, value) in items.iter().enumerate() {
                let pointer = format!("{prefix}/{index}");
                match survivors.and_then(|array| array.get(index)) {
                    Some(survivor) => collect_dropped(&pointer, value, survivor, dropped),
                    None => dropped.push(pointer),
                }
            }
        }
        _ => {}
    }
}

/// The repository's canonical JSON rendering for **fixtures and payload bytes**:
/// `serde_json::to_string_pretty` (two-space indent) plus exactly one trailing `\n`, UTF-8, no BOM.
///
/// Serializes the *typed value*, so struct fields appear in declaration order — which is the
/// documented wire order. Preserved `extensions` keys follow, in `BTreeMap` order.
///
/// # Errors
///
/// Propagates a `serde_json` serialization failure.
pub fn canonical_json<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let mut rendered = serde_json::to_string_pretty(value)?;
    rendered.push('\n');
    Ok(rendered)
}
