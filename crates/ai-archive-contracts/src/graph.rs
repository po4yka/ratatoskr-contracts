//! [`AiProject`], [`AiConversation`] and [`AiMessage`]: the normalized graph nodes of one
//! import, each stamped with the parser that produced it.
//!
//! None of these types uses `deny_unknown_fields`: every one flattens an `extensions` map, and
//! the tolerant-reader rule (unknown additive members are preserved verbatim) needs the flatten
//! to capture them. Strictness here would break preservation, not strengthen it.

use ratatoskr_error_contracts::WarningEnvelope;
use ratatoskr_identifiers::{
    AiConversationId, AiProjectId, ContentDigest, EntityLocalId, EntityRef, Extensions, TenantRef,
    WireTimestamp, wire_string_newtype,
};

use crate::content_part::AiContentPart;
use crate::tokens::{AiProvider, ParserName, ParserVersion};
use crate::values::{AiText, AiTitle};

wire_string_newtype! {
    /// The model a message was answered with, e.g. `gpt-5.2`, `claude-opus-4-6`.
    ///
    /// **Open on purpose**: model names are provider-owned and change constantly; dots, digits
    /// and hyphens all occur. Consumers treat an unrecognized name as opaque display text.
    pub struct AiModelName {
        pattern  = r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$",
        max_len  = 64,
        examples = ["gpt-5.2", "claude-opus-4-6"],
    }
}

/// Who authored a message.
///
/// **Closed on purpose.** Role drives attribution, indexing and rendering; silently filing an
/// unrecognized role under a default would misattribute statements to their author. Both
/// supported providers' exports normalize into exactly these roles (a Claude export's `human`
/// becomes `user` at parse time in the producer, not here).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AiAuthorRole {
    /// The account owner.
    User,
    /// The assistant.
    Assistant,
    /// A system or custom instruction.
    System,
    /// A tool or connector speaking between calls.
    Tool,
}

/// One normalized project node.
///
/// Projects group conversations. Every field is either Ratatoskr's own identity, an explicit
/// provider reference, or provider-authored content; nothing is inferred.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiProject {
    /// Ratatoskr's own identity for this project, stable across re-imports of the same export
    /// evidence. Bare canonical UUID on the wire; never derived from the provider's id.
    pub ai_project_id: AiProjectId,

    /// The provider the project belongs to. Open token.
    pub provider: AiProvider,

    /// The provider's id for the project, when the export supplies one. Opaque and
    /// case-sensitive; deduplication keys on `provider` plus this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_project_id: Option<EntityLocalId>,

    /// The project's title.
    pub title: AiTitle,

    /// The project description, when the export supplies one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<AiText>,

    /// The project's standing instructions, when the export supplies them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<AiText>,

    /// Instant the provider says the project was created. Provider-authored: present only when
    /// the export supplied it, never inferred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_created_at: Option<WireTimestamp>,

    /// Instant the provider says the project last changed. Provider-authored on the same terms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_updated_at: Option<WireTimestamp>,

    /// Which parser build normalized this node.
    pub parser_name: ParserName,

    /// The build of [`AiProject::parser_name`].
    pub parser_version: ParserVersion,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// One normalized conversation node: the unit Knowledge indexes.
///
/// Messages travel in provider presentation order; the graph structure rides beside the order
/// in each message's optional parent reference, so branches, regenerated answers and edited
/// histories all survive normalization without inventing list positions for them.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiConversation {
    /// Ratatoskr's own identity for this conversation, stable across updates. Bare canonical
    /// UUID on the wire; never derived from the provider's id.
    pub ai_conversation_id: AiConversationId,

    /// The provider the conversation belongs to. Open token.
    pub provider: AiProvider,

    /// The provider's id for the conversation, when the export supplies one. Opaque and
    /// case-sensitive; deduplication keys on `provider` plus this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_conversation_id: Option<EntityLocalId>,

    /// The user whose archive holds this conversation. Every event about this id carries the
    /// same owner.
    pub owner: TenantRef,

    /// The project this conversation belongs to, when any. A pointer to another Ratatoskr
    /// record, so the wire form names the referent kind: `ai_project:<uuid>` (ADR-0007).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_ref: Option<EntityRef>,

    /// The conversation's title, when the export supplies one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<AiTitle>,

    /// Instant the provider says the conversation was created. Provider-authored: present only
    /// when the export supplied it, never inferred from import time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_created_at: Option<WireTimestamp>,

    /// Instant the provider says the conversation last changed. Provider-authored likewise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_updated_at: Option<WireTimestamp>,

    /// The conversation's messages in provider presentation order. Graph edges live in each
    /// message's `parent_message_id`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<AiMessage>,

    /// Digest of the normalized content computed by the producer over the canonical JSON of
    /// `messages`. Consumers treat a mismatch on recomputation as corruption, not change.
    pub content_digest: ContentDigest,

    /// Which parser build normalized this node.
    pub parser_name: ParserName,

    /// The build of [`AiConversation::parser_name`].
    pub parser_version: ParserVersion,

    /// Non-blocking problems recorded while normalizing this conversation. Permitted on any
    /// conversation; archive-level gaps are reported once, in the completeness report.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<WarningEnvelope>,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

/// One normalized message node inside a [`AiConversation`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiMessage {
    /// The provider's id for the message, opaque and case-sensitive. Parent references name
    /// siblings by this value inside the same conversation.
    pub external_message_id: EntityLocalId,

    /// Who authored the message. Closed vocabulary.
    pub author_role: AiAuthorRole,

    /// The authoring message this one answers, regenerates or branches from, named by sibling
    /// external id within the same conversation. Absent on a thread root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_message_id: Option<EntityLocalId>,

    /// The message's content parts in provider order, drawn from one shared grammar.
    /// Unrecognized parts are preserved verbatim ([`AiContentPart::Unknown`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<AiContentPart>,

    /// The model that produced an assistant message, when the export supplies one. Open token;
    /// opaque to consumers beyond display.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<AiModelName>,

    /// Instant the provider says the message was written. Provider-authored: present only when
    /// the export supplied it, never fabricated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_created_at: Option<WireTimestamp>,

    /// Which parser build normalized this node.
    pub parser_name: ParserName,

    /// The build of [`AiMessage::parser_name`].
    pub parser_version: ParserVersion,

    /// Unknown-but-preserved additive fields.
    #[serde(flatten)]
    pub extensions: Extensions,
}

impl AiMessage {
    /// The message's own provider-minted identity.
    #[must_use]
    pub fn external_message_id(&self) -> &EntityLocalId {
        &self.external_message_id
    }
}
