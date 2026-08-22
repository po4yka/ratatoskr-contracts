//! Asset references: stored files, artifacts and canvas-like objects by [`BlobRef`], plus
//! citations and tool activity — never bytes.

use ratatoskr_identifiers::{BlobRef, EntityLocalId, wire_string_newtype};

use crate::values::AiTitle;

wire_string_newtype! {
    /// What an [`AiAsset`] holds, e.g. `file`, `artifact`, `canvas`.
    ///
    /// **Open on purpose.** Providers draw the file/artifact/canvas line differently and redraw
    /// it over time; a validated token keeps new kinds from breaking a running consumer, and a
    /// consumer renders or skips an unrecognized kind generically while keeping the record.
    pub struct AiAssetKind {
        pattern  = r"^[a-z][a-z0-9_]{0,31}$",
        max_len  = 32,
        examples = ["file", "artifact", "canvas"],
    }
}

wire_string_newtype! {
    /// The name a provider gave one tool, e.g. `web_search`, `code_interpreter`.
    ///
    /// **Open on purpose**, like every provider-owned vocabulary here: dots and hyphens occur in
    /// real tool names, so the token alphabet is wider than the snake_case segment grammar.
    pub struct AiToolName {
        pattern  = r"^[a-z0-9][a-z0-9._-]{0,62}$",
        max_len  = 64,
        examples = ["web_search", "code_interpreter", "dalle.v2"],
    }
}

wire_string_newtype! {
    /// The canonical HTTPS address of a cited source.
    ///
    /// A deliberate lower bound: absolute `https://`, no whitespace, no control characters.
    /// Full URL syntax validation belongs to the producer that minted the link; this contract
    /// only guarantees the value is unambiguous to render and store. `http://` is refused —
    /// both providers serve citations over TLS.
    pub struct AiSourceUrl {
        pattern  = r"^https://[!-~]{1,2000}$",
        max_len  = 2048,
        examples = ["https://example.com/source"],
    }
}

/// How a tool invocation ended.
///
/// **Closed on purpose**: a consumer must not guess whether an unclassifiable outcome means
/// the assistant's answer rests on a failed call. Adding a variant is an additive wire
/// change consumers adopt by upgrading; until then an unknown outcome stops processing.
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
pub enum AiToolOutcome {
    /// The tool ran and returned normally; the result may still be wrong.
    Succeeded,
    /// The tool failed, timed out or was refused.
    Failed,
}

/// One stored asset of an AI conversation: a file, artifact or canvas-like object.
///
/// A reference, never bytes. `asset_kind` says what the provider called it; the [`BlobRef`]
/// names where the bytes live and lets a reader verify them.
///
/// [`BlobRef`]: ratatoskr_identifiers::BlobRef
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AiAsset {
    /// What kind of object this is. Open token.
    pub asset_kind: AiAssetKind,

    /// Content-addressed bytes owned by the storing service.
    pub blob: BlobRef,

    /// The display name the export gave the file, when it named it at all. Never a path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<crate::values::AiFileName>,
}

/// One citation inside a message: what was cited and where its evidence lives.
///
/// Every member is optional because providers cite differently — a URL, a stored passage, a
/// bare title, or any combination. A citation with no resolvable member is still honest: it
/// records that the model attributed its statement to something.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AiCitation {
    /// The cited work's title, as the export rendered it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<AiTitle>,

    /// Canonical HTTPS address of the cited source, when the export supplied one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<AiSourceUrl>,

    /// The cited passage as stored bytes, when the importing service kept it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_blob: Option<BlobRef>,
}

/// One side of a tool invocation: the call itself.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AiToolCall {
    /// The provider's identifier for this invocation, when it supplied one. Pairs with the
    /// matching [`AiToolResult::tool_call_id`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<EntityLocalId>,

    /// Which tool the model invoked. Open token.
    pub tool_name: AiToolName,
}

/// The other side of a tool invocation: how it ended.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AiToolResult {
    /// The provider's identifier for this invocation, when it supplied one. Pairs with the
    /// matching [`AiToolCall::tool_call_id`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<EntityLocalId>,

    /// How the invocation ended. Closed vocabulary.
    pub outcome: AiToolOutcome,

    /// Bounded text output of the tool, when there is any worth carrying inline. Larger outputs
    /// belong in an [`AiAsset`], not here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_text: Option<crate::values::AiText>,
}
