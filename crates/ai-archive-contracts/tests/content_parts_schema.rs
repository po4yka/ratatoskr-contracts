//! The hand-written JSON Schema of [`AiContentPart`] matches the hand-written serde.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use schemars::Schema;
use schemars::generate::SchemaGenerator;

/// Builds a root schema document for `AiContentPart` the way `contractsc` does.
fn root_schema() -> Schema {
    let mut generator = SchemaGenerator::default();
    generator.root_schema_for::<ratatoskr_ai_archive_contracts::AiContentPart>()
}

#[test]
fn compiles_as_draft_2020_12() {
    let schema = root_schema();
    let document = serde_json::to_value(&schema).unwrap();
    jsonschema::meta::validate(&document)
        .unwrap_or_else(|error| panic!("the generated schema is not valid draft 2020-12: {error}"));
}

#[test]
fn accepts_every_known_tagged_shape() {
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&serde_json::to_value(root_schema()).unwrap())
        .expect("the schema compiles");

    for accepted in [
        serde_json::json!({ "part_kind": "text", "text": "plain answer" }),
        serde_json::json!({ "part_kind": "markdown", "markdown": "# heading" }),
        serde_json::json!({
            "part_kind": "image",
            "image": {
                "owner_service": "ratatoskr-chatgpt",
                "digest": { "algorithm": "sha256",
                            "hex": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" },
                "media_type": "image/png",
                "length_bytes": 2048
            }
        }),
        serde_json::json!({
            "part_kind": "asset",
            "asset": {
                "asset_kind": "artifact",
                "blob": {
                    "owner_service": "ratatoskr-claude",
                    "digest": { "algorithm": "sha256",
                                "hex": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" },
                    "media_type": "text/html",
                    "length_bytes": 4096
                }
            }
        }),
        serde_json::json!({ "part_kind": "citation", "citation": {} }),
        serde_json::json!({
            "part_kind": "tool_call",
            "tool_call": { "tool_name": "web_search" }
        }),
        serde_json::json!({
            "part_kind": "tool_result",
            "tool_result": { "outcome": "succeeded" }
        }),
    ] {
        validator
            .validate(&accepted)
            .unwrap_or_else(|error| panic!("{accepted} must validate: {error}"));
    }
}

#[test]
fn unknown_branch_is_open_and_marked_preserve() {
    let document = serde_json::to_value(root_schema()).unwrap();
    let branches = document
        .pointer("/oneOf")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("the part schema is a oneOf: {document}"));

    let marked = branches
        .iter()
        .find(|branch| branch.get("x-ratatoskr-unknown-policy").is_some())
        .unwrap_or_else(|| panic!("one branch must be the marked unknown channel: {branches:?}"));
    assert_eq!(
        marked
            .get("x-ratatoskr-unknown-policy")
            .and_then(serde_json::Value::as_str),
        Some("preserve")
    );

    // The open branch accepts an arbitrary future record...
    let exotic = serde_json::json!({
        "part_kind": "hologram_projection",
        "holo_data": { "depth_map_ref": "blob-42" }
    });
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&document)
        .expect("compiles");
    validator.validate(&exotic).unwrap_or_else(|error| {
        panic!("an unknown record must pass the published schema: {error}")
    });

    // ...and a non-object record too...
    validator
        .validate(&serde_json::json!("a plain string part"))
        .unwrap_or_else(|error| panic!("a non-object record must pass the schema: {error}"));

    // ...while a malformed known part matches no branch, mirroring the Rust layer.
    assert!(
        validator
            .validate(&serde_json::json!({ "part_kind": "text", "body": "wrong" }))
            .is_err(),
        "a malformed known part must fail the published schema"
    );
}

#[test]
fn known_branches_stay_exhaustive_over_the_shared_vocabulary() {
    let document = serde_json::to_value(root_schema()).unwrap();
    let branches = document
        .pointer("/oneOf")
        .and_then(serde_json::Value::as_array)
        .expect("oneOf");
    for kind in [
        "text",
        "markdown",
        "image",
        "asset",
        "citation",
        "tool_call",
        "tool_result",
    ] {
        assert!(
            branches.iter().any(|branch| branch
                .pointer("/properties/part_kind/const")
                .and_then(serde_json::Value::as_str)
                == Some(kind)),
            "{kind} must have its own exact branch"
        );
    }
}
