//! The write command `notes.file.write_requested.v1` (contract `notes/write-request.v1`, workspace
//! spec `notes-write-command`). Every refusal is checked at both layers: the Rust type and the
//! generated JSON Schema. Every refusal test also checks a valid fixture of the same kind, so a
//! layer that refuses everything cannot pass.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "assertions in a test binary"
)]

mod common;

use common::ContractUnderTest;

fn write_request() -> &'static ContractUnderTest {
    common::contract("notes/write-request.v1", "notes.file.write_requested.v1")
}

/// A payload carries exactly one operation.
#[test]
fn two_operations_are_refused() {
    let contract = write_request();
    contract.assert_accepted_by_both("rename.json");
    contract.assert_accepted_by_both("delete.json");
    contract.assert_refused_by_both("two-operations.json");
}

/// `origin` is `platform` or `site:<site id>`, and nothing else.
#[test]
fn origin_has_two_forms() {
    let contract = write_request();
    contract.assert_accepted_by_both("create.json");
    contract.assert_accepted_by_both("update.json");
    contract.assert_refused_by_both("empty-site-origin.json");
    contract.assert_refused_by_both("unknown-origin.json");
}

/// Every operation except `create` and `resolve_conflict` names the revision it changes.
#[test]
fn update_requires_base_hash() {
    let contract = write_request();
    contract.assert_accepted_by_both("update.json");
    contract.assert_refused_by_both("update-without-base-hash.json");
}

/// A `custom` resolution names the uploaded content; the other choices name none.
#[test]
fn custom_resolution_requires_content() {
    let contract = write_request();
    contract.assert_accepted_by_both("resolve-conflict.json");
    contract.assert_accepted_by_both("resolve-conflict-custom.json");
    contract.assert_refused_by_both("custom-resolution-without-content.json");
    contract.assert_refused_by_both("blob-without-custom-resolution.json");
}

/// A path is relative to the vault root, has no `..` segment and is outside `.obsidian/`.
#[test]
fn paths_stay_inside_the_vault() {
    let contract = write_request();
    contract.assert_accepted_by_both("move.json");
    contract.assert_refused_by_both("parent-segment.json");
    contract.assert_refused_by_both("absolute-path.json");
    contract.assert_refused_by_both("config-folder.json");
}

/// `.obsidian` is refused as the first segment in any ASCII case and as a bare name, and a
/// folder whose name only starts with `.obsidian` stays writable. A pin of the `VaultWritePath`
/// pattern of task 3.2.
#[test]
fn config_folder_is_refused_in_any_case() {
    let contract = write_request();
    contract.assert_accepted_by_both("obsidian-like-folder.json");
    contract.assert_refused_by_both("config-folder-uppercase.json");
    contract.assert_refused_by_both("config-folder-mixed-case.json");
    contract.assert_refused_by_both("config-folder-bare.json");
}

/// The content of a write is a `BlobRef` that the caller uploaded first, never inline text.
#[test]
fn content_is_a_blob_ref() {
    let contract = write_request();
    contract.assert_accepted_by_both("update.json");
    contract.assert_accepted_by_both("set-property.json");
    contract.assert_accepted_by_both("remove-property.json");
    contract.assert_refused_by_both("inline-content.json");
}

/// A frontmatter value is note content, so it travels like file content: uploaded to notes
/// first as `application/json`, and named by a `BlobRef`. An inline JSON value is refused.
#[test]
fn property_value_is_a_blob_ref() {
    let contract = write_request();
    contract.assert_accepted_by_both("set-property.json");
    contract.assert_accepted_by_both("remove-property.json");
    contract.assert_refused_by_both("inline-property-value.json");
    contract.assert_refused_by_both("property-value-not-json.json");
}

/// Vault adapters cannot distinguish a final dot or space on some file systems.
#[test]
fn segment_that_ends_in_a_dot_or_a_space_is_refused() {
    let contract = write_request();
    contract.assert_accepted_by_both("rename.json");
    contract.assert_accepted_by_both("obsidian-like-folder.json");
    for name in [
        "segment-ending-in-a-dot.json",
        "config-folder-ending-in-a-dot.json",
        "segment-ending-in-a-space.json",
    ] {
        assert_unsafe_path_at_both_positions(contract, name);
    }
    for path in ["Notes/x.", "Notes/x ", " . /x.md", "x/ ./y.md"] {
        assert_path_layers(contract, path, false);
    }
    for path in [
        "Notes 2026/Plan.md",
        "Notes.v1/Plan.md",
        ".obsidian-archive/Plan.md",
        ".o/Plan.md",
        ".obsidia/Plan.md",
        "..archive/Plan.md",
        "x/.obsidian/Plan.md",
        "x/ Plan.md",
        "x/.hidden",
        "x/..hidden",
        "x/a.b",
    ] {
        assert_path_layers(contract, path, true);
    }
}

/// A colon can select an alternate data stream on a supported file system.
#[test]
fn colon_is_refused() {
    let contract = write_request();
    contract.assert_accepted_by_both("rename.json");
    assert_unsafe_path_at_both_positions(contract, "colon.json");
    assert_path_layers(contract, "Notes/B:C.md", false);
}

fn assert_unsafe_path_at_both_positions(contract: &ContractUnderTest, name: &str) {
    let payload = contract.fixture(&format!("invalid/{name}"));
    let path = payload
        .pointer("/operation/rename/path")
        .and_then(serde_json::Value::as_str)
        .expect("the fixture names a rename source");
    assert_path_layers(contract, path, false);
}

fn assert_path_layers(contract: &ContractUnderTest, path: &str, accepted: bool) {
    let validator = contract.validator();
    for field in ["path", "target_path"] {
        let mut payload = contract.fixture("valid/rename.json");
        *payload
            .pointer_mut(&format!("/operation/rename/{field}"))
            .expect("the fixture has both rename paths") = serde_json::json!(path);
        let serde_accepts = (contract.roundtrip)(&payload).is_ok();
        let schema_accepts = validator.is_valid(&payload);
        assert_eq!(
            (serde_accepts, schema_accepts),
            (accepted, accepted),
            "{field} {path:?}: (serde, schema) acceptance"
        );
    }
}
