//! The shared content-part grammar and its unknown-part preservation channel.

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes run on values the test itself built, so an               out-of-bounds or missing member panics the test, which is the reporting mechanism"
)]

mod common;

use ratatoskr_ai_archive_contracts::{
    AiAsset, AiCitation, AiContentPart, AiToolCall, AiToolOutcome, AiToolResult,
};
use ratatoskr_identifiers::BlobRef;

/// Every known part parses from its tagged shape and re-emits byte-identically.
#[test]
fn known_parts_round_trip() {
    let cases: Vec<(AiContentPart, serde_json::Value)> = vec![
        (
            AiContentPart::Text {
                text: common::text("plain answer"),
            },
            serde_json::json!({ "part_kind": "text", "text": "plain answer" }),
        ),
        (
            AiContentPart::Markdown {
                markdown: common::text("# heading\nbody"),
            },
            serde_json::json!({ "part_kind": "markdown", "markdown": "# heading\nbody" }),
        ),
        (
            AiContentPart::Image {
                image: common::blob_ref("image/png"),
            },
            serde_json::json!({
                "part_kind": "image",
                "image": {
                    "owner_service": "ratatoskr-chatgpt",
                    "digest": { "algorithm": "sha256", "hex": common::DIGEST },
                    "media_type": "image/png",
                    "length_bytes": 2048
                }
            }),
        ),
        (
            AiContentPart::Asset {
                asset: AiAsset {
                    asset_kind: common::asset_kind("artifact"),
                    blob: common::blob_ref_for("ratatoskr-claude", "text/html", 4096),
                    file_name: Some(common::file_name("analysis.html")),
                },
            },
            serde_json::json!({
                "part_kind": "asset",
                "asset": {
                    "asset_kind": "artifact",
                    "blob": {
                        "owner_service": "ratatoskr-claude",
                        "digest": { "algorithm": "sha256", "hex": common::DIGEST },
                        "media_type": "text/html",
                        "length_bytes": 4096
                    },
                    "file_name": "analysis.html"
                }
            }),
        ),
    ];

    for (part, wire) in cases {
        let reemitted = common::roundtrip(&wire, &part);
        assert_eq!(reemitted, wire);
    }
}

/// Citation and tool parts keep every optional member present or absent as set.
#[test]
fn citation_and_tool_parts_carry_optional_members() {
    let bare = AiContentPart::Citation {
        citation: AiCitation {
            title: None,
            url: None,
            source_blob: None,
        },
    };
    let wire = serde_json::to_value(&bare).unwrap();
    assert_eq!(
        wire,
        serde_json::json!({ "part_kind": "citation", "citation": {} })
    );

    let titled = AiContentPart::Citation {
        citation: AiCitation {
            title: Some(common::title("A cited page")),
            url: None,
            source_blob: Some(common::blob_ref("text/html")),
        },
    };
    let roundtripped = common::roundtrip_value(&titled);
    assert_eq!(roundtripped["citation"]["title"], "A cited page");
    assert_eq!(
        roundtripped["citation"]["source_blob"]["media_type"],
        "text/html"
    );
    assert!(roundtripped["citation"].get("url").is_none());

    let call = AiContentPart::ToolCall {
        tool_call: AiToolCall {
            tool_call_id: Some(common::local_id("call_abc123")),
            tool_name: common::tool_name("web_search"),
        },
    };
    let outcome = AiContentPart::ToolResult {
        tool_result: AiToolResult {
            tool_call_id: Some(common::local_id("call_abc123")),
            outcome: AiToolOutcome::Succeeded,
            output_text: Some(common::text("3 results")),
        },
    };
    assert_eq!(
        common::roundtrip_value(&call)["tool_call"]["tool_call_id"],
        "call_abc123"
    );
    assert_eq!(
        common::roundtrip_value(&outcome)["tool_result"]["outcome"],
        "succeeded"
    );
}

/// An unrecognized part kind is preserved verbatim — the defining preservation case.
#[test]
fn unknown_part_kinds_survive_verbatim() {
    let exotic = serde_json::json!({
        "part_kind": "hologram_projection",
        "holo_data": { "depth_map_ref": "blob-42" },
        "notes": ["not", "typed"]
    });
    let parsed: AiContentPart = serde_json::from_value(exotic.clone()).expect("preserved");
    match parsed {
        AiContentPart::Unknown(_) => {}
        other => panic!("expected the unknown channel, got {other:?}"),
    }
    assert_eq!(serde_json::to_value(&parsed).unwrap(), exotic);

    // A non-object part is equally preserved.
    let scalar = serde_json::json!("a plain string part");
    let parsed: AiContentPart = serde_json::from_value(scalar.clone()).expect("preserved");
    assert!(matches!(parsed, AiContentPart::Unknown(_)));
    assert_eq!(serde_json::to_value(&parsed).unwrap(), scalar);
}

/// A recognized discriminator with an invalid body fails loudly instead of being demoted to
/// the unknown channel.
#[test]
fn malformed_known_part_is_not_quietly_demoted() {
    let missing_text = serde_json::json!({
        "part_kind": "text",
        "body": "wrong member name"
    });
    let error = serde_json::from_value::<AiContentPart>(missing_text)
        .expect_err("a text part without text is malformed");
    assert!(
        error.to_string().contains("AiContentPart"),
        "the error must name the type: {error}"
    );

    let wrong_type = serde_json::json!({
        "part_kind": "image",
        "image": 2048
    });
    assert!(serde_json::from_value::<AiContentPart>(wrong_type).is_err());
}

/// A `BlobRef`-carrying part never carries bytes; nothing here accepts base64 blobs.
#[test]
fn image_part_rejects_inline_bytes() {
    let inline = serde_json::json!({
        "part_kind": "image",
        "image": "iVBORw0KGgoAAAANSUhEUg=="
    });
    assert!(serde_json::from_value::<AiContentPart>(inline).is_err());

    let blob: BlobRef = serde_json::from_value(serde_json::json!({
        "owner_service": "ratatoskr-chatgpt",
        "digest": { "algorithm": "sha256", "hex": common::DIGEST },
        "media_type": "image/png",
        "length_bytes": 1
    }))
    .expect("a reference is the only carrier");
    assert_eq!(blob.length_bytes, 1);
}
