//! Project/conversation/message graph nodes.

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes run on values the test itself built, so an               out-of-bounds or missing member panics the test, which is the reporting mechanism"
)]

mod common;

use ratatoskr_ai_archive_contracts::{
    AiAssetKind, AiAuthorRole, AiContentPart, AiConversation, AiMessage, AiModelName, AiProject,
};
use ratatoskr_identifiers::{AiProjectId, EntityLocalId, Extensions, TenantRef, WireTimestamp};

/// A project carrying every field round-trips; optional members absent serialize as absent.
#[test]
fn project_round_trips_with_every_field() {
    let mut extensions = Extensions::new();
    extensions.insert("provider_project_url_path", serde_json::json!("/g/p-1"));

    let project = AiProject {
        ai_project_id: common::project_id(),
        provider: common::provider("claude"),
        external_project_id: Some(common::local_id("prd_01j9")),
        title: common::title("Contract work"),
        description: Some(common::text("Everything about the wire contracts.")),
        instructions: Some(common::text("Answer tersely.\nCite sources.")),
        provider_created_at: Some(common::instant("2026-07-01T08:00:00Z")),
        provider_updated_at: Some(common::instant("2026-08-01T09:30:00Z")),
        parser_name: common::parser_name("claude_export"),
        parser_version: common::parser_version("1.4.2"),
        extensions,
    };

    let wire = serde_json::to_value(&project).unwrap();
    assert_eq!(wire["ai_project_id"], common::PROJECT_UUID);
    assert_eq!(wire["external_project_id"], "prd_01j9");
    assert_eq!(wire["instructions"], "Answer tersely.\nCite sources.");
    assert_eq!(wire["provider_updated_at"], "2026-08-01T09:30:00Z");
    let decoded: AiProject = serde_json::from_value(wire).expect("and deserializes");
    assert_eq!(decoded, project);

    // The minimal shape: every optional member omitted.
    let minimal = AiProject {
        external_project_id: None,
        description: None,
        instructions: None,
        provider_created_at: None,
        provider_updated_at: None,
        ..project.clone()
    };
    let wire = serde_json::to_value(&minimal).unwrap();
    assert!(wire.get("external_project_id").is_none());
    assert!(wire.get("provider_created_at").is_none());
    let decoded: AiProject = serde_json::from_value(wire).expect("the minimal shape parses");
    assert_eq!(decoded, minimal);
}

/// A conversation carries its messages in presentation order plus the graph edges.
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "a test building a three-message branched conversation field by field"
)]
fn conversation_carries_order_and_graph_edges() {
    let prompt = AiMessage {
        external_message_id: common::local_id("msg-0001"),
        author_role: AiAuthorRole::User,
        parent_message_id: None,
        parts: vec![AiContentPart::Text {
            text: common::text("Explain E0597."),
        }],
        model: None,
        provider_created_at: Some(common::instant("2026-08-02T10:00:00Z")),
        parser_name: common::parser_name("chatgpt_export"),
        parser_version: common::parser_version("2026.08.1"),
        extensions: Extensions::new(),
    };
    let answer_a = AiMessage {
        external_message_id: common::local_id("msg-0002"),
        author_role: AiAuthorRole::Assistant,
        parent_message_id: Some(common::local_id("msg-0001")),
        parts: vec![
            AiContentPart::Markdown {
                markdown: common::text("Lifetimes borrow from **scopes**."),
            },
            AiContentPart::Image {
                image: common::blob_ref("image/png"),
            },
        ],
        model: Some(AiModelName::parse("gpt-5.2").expect("a legal model name")),
        provider_created_at: None,
        parser_name: common::parser_name("chatgpt_export"),
        parser_version: common::parser_version("2026.08.1"),
        extensions: Extensions::new(),
    };
    let answer_b = AiMessage {
        external_message_id: common::local_id("msg-0003"),
        author_role: AiAuthorRole::Assistant,
        parent_message_id: Some(common::local_id("msg-0001")),
        parts: vec![AiContentPart::Markdown {
            markdown: common::text("A different regeneration."),
        }],
        model: None,
        provider_created_at: None,
        parser_name: common::parser_name("chatgpt_export"),
        parser_version: common::parser_version("2026.08.1"),
        extensions: Extensions::new(),
    };

    let conversation = AiConversation {
        ai_conversation_id: common::conversation_id(),
        provider: common::provider("chatgpt"),
        external_conversation_id: Some(common::local_id("conv-77")),
        owner: TenantRef::parse(common::OWNER).expect("a legal owner"),
        project_ref: None,
        title: Some(common::title("Borrow checker deep dive")),
        provider_created_at: Some(common::instant("2026-08-02T09:59:00Z")),
        provider_updated_at: Some(common::instant("2026-08-02T10:05:00Z")),
        messages: vec![prompt, answer_a.clone(), answer_b.clone()],
        content_digest: common::digest(),
        parser_name: common::parser_name("chatgpt_export"),
        parser_version: common::parser_version("2026.08.1"),
        warnings: Vec::new(),
        extensions: Extensions::new(),
    };

    let wire = serde_json::to_value(&conversation).unwrap();
    assert_eq!(
        wire["messages"]
            .as_array()
            .expect("messages are an array")
            .len(),
        3,
        "presentation order is preserved"
    );
    // Both regenerated answers point at the same prompt.
    for index in [1usize, 2] {
        assert_eq!(wire["messages"][index]["parent_message_id"], "msg-0001");
    }

    let decoded: AiConversation = serde_json::from_value(wire).expect("deserializes");
    assert_eq!(decoded, conversation);
    let parents: Vec<Option<&EntityLocalId>> = decoded
        .messages
        .iter()
        .map(|message| message.parent_message_id.as_ref())
        .collect();
    assert_eq!(
        parents,
        vec![
            None,
            Some(decoded.messages[0].external_message_id()),
            Some(decoded.messages[0].external_message_id())
        ]
    );

    // A projectless conversation without any graph edge still parses.
    let linear = AiConversation {
        external_conversation_id: None,
        project_ref: None,
        title: None,
        provider_created_at: None,
        provider_updated_at: None,
        messages: vec![
            AiMessage {
                parent_message_id: None,
                ..answer_a
            },
            AiMessage {
                parent_message_id: None,
                ..answer_b
            },
        ],
        ..conversation.clone()
    };
    let decoded: AiConversation =
        serde_json::from_value(serde_json::to_value(linear).unwrap()).expect("parses");
    assert!(decoded.project_ref.is_none());
}

/// A conversation may reference its project by kinded reference.
#[test]
fn conversation_references_its_project() {
    let mut conversation = common::minimal_conversation();
    conversation.project_ref = Some(common::project_ref());

    let wire = serde_json::to_value(&conversation).unwrap();
    assert_eq!(wire["project_ref"], common::PROJECT_REF);
    let decoded: AiConversation = serde_json::from_value(wire).expect("deserializes");
    assert_eq!(decoded, conversation);

    let narrowed = AiProjectId::try_from(decoded.project_ref.as_ref().expect("set"));
    assert_eq!(narrowed.expect("the kind matches"), common::project_id());
}

/// Provider-authored timestamps are instants only in canonical UTC spelling.
#[test]
fn provider_timestamps_must_be_canonical() {
    let mut message = common::minimal_message();
    message.provider_created_at = Some(WireTimestamp::parse("2026-08-02T10:00:00Z").expect("ok"));

    let wire = serde_json::to_value(&message).unwrap();
    let decoded: AiMessage = serde_json::from_value(wire).expect("canonical parses");
    assert_eq!(decoded.provider_created_at, message.provider_created_at);

    let offset = serde_json::json!({
        "external_message_id": "m-1",
        "author_role": "user",
        "parts": [],
        "provider_created_at": "2026-08-02T12:00:00+02:00",
        "parser_name": "x",
        "parser_version": "1"
    });
    let error = serde_json::from_value::<AiMessage>(offset)
        .expect_err("an offset instant is not canonical");
    assert!(
        error.to_string().contains("not in canonical form"),
        "{error}"
    );
}

/// Author roles and model names validate.
#[test]
fn roles_and_model_names_validate() {
    for role in ["user", "assistant", "system", "tool"] {
        let parsed: AiAuthorRole = serde_json::from_str(&format!("\"{role}\"")).expect(role);
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            format!("\"{role}\"")
        );
    }
    assert!(serde_json::from_str::<AiAuthorRole>("\"shadow_user\"").is_err());

    let model = AiModelName::parse("claude-opus-4-6").expect("a legal model name");
    assert_eq!(model.as_str(), "claude-opus-4-6");
    assert!(AiModelName::parse("-leading").is_err());
    assert!(AiModelName::parse("").is_err());
}

/// Asset kinds stay open tokens.
#[test]
fn asset_kinds_stay_open() {
    assert_eq!(
        AiAssetKind::parse("canvas").expect("known").as_str(),
        "canvas"
    );
    assert_eq!(
        AiAssetKind::parse("whiteboard_v9").expect("open").as_str(),
        "whiteboard_v9"
    );
    assert!(AiAssetKind::parse("Canvas").is_err());
}
