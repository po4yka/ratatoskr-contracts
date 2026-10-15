//! The conflict events: a write met a revision that its writer did not see, and the owner
//! resolved it.

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{
    DigestHex, MediaType, NotesConflictId, NotesConnectionId, OperationId,
};

use crate::error::NotesContractError;
use crate::file::VaultPath;
use crate::write::WriteOrigin;

/// The media types for which `ratatoskr-notes` computes a line-based three-way merge.
const MERGEABLE_MEDIA_TYPES: &[&str] = &["text/markdown", "text/plain"];

/// A line-based three-way merge of the base, the current revision and the writer's content.
///
/// A proposal only: `ratatoskr-notes` never applies it on its own. The owner reads the merged
/// bytes through `GET /v1/notes/conflicts/{conflict_id}` and accepts them with the resolution
/// choice `accept_merge`.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct MergeProposal {
    /// The lowercase hex SHA-256 of the merged bytes.
    pub merged_hash: DigestHex,

    /// `true` when at least one hunk changed on both sides, so the merged bytes need the owner's
    /// review; `false` when the two sides changed different lines.
    pub has_overlapping_hunks: bool,
}

/// Payload of `notes.conflict.raised.v1` (contract `notes/conflict.v1`): a write reached the head
/// of its file's queue and its base revision was not the current revision, or a `create` met a
/// live file with the same case-folded path. The current revision did not change.
///
/// A fact. The enclosing envelope carries the owner as `tenant_id` (`user:<id>`) and the
/// aggregate `notes_conflict:<conflict_id>`. The payload names the three sides by their hashes and
/// never carries file content; a client reads the sides and the proposal through
/// `GET /v1/notes/conflicts/{conflict_id}`. Every member it does not declare is refused.
///
/// `Deserialize` is hand-written because the rules are cross-field: a `merge_proposal` is present
/// only for `text/markdown` and `text/plain`, and only when all three hashes are present. It
/// parses a private mirror struct and then calls [`ConflictRaised::validate`]. The published
/// schema states the same rules as a `oneOf` of three closed shapes (see
/// [`merge_rule_branches`]), so a schema-only consumer refuses the same payloads.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(transform = merge_rule_branches)]
pub struct ConflictRaised {
    /// The conflict. It stays open until a `resolve_conflict` write names it.
    pub conflict_id: NotesConflictId,

    /// The Platform operation of kind `notes.file.write` that ended in this conflict.
    pub operation_id: OperationId,

    /// The connection whose vault holds the file.
    pub connection_id: NotesConnectionId,

    /// The path that the write named, relative to the vault root, in NFC.
    pub path: VaultPath,

    /// Who asked for the write, repeated from the write: `platform` or `site:<site id>`.
    pub origin: WriteOrigin,

    /// The media type of the file at `path`.
    pub media_type: MediaType,

    /// The lowercase hex SHA-256 of the revision that the writer read. Absent for `create`,
    /// which names no base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_hash: Option<DigestHex>,

    /// The lowercase hex SHA-256 of the current revision when the conflict was raised. Absent
    /// when no live file holds the path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<DigestHex>,

    /// The lowercase hex SHA-256 of the content that the writer sent. Absent for `rename`, `move`
    /// and `delete`, which carry no content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writer_hash: Option<DigestHex>,

    /// The three-way merge proposal. Present only for `text/markdown` and `text/plain` when the
    /// base revision is still stored, and so only when all three hashes are present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_proposal: Option<MergeProposal>,
}

impl EventPayload for ConflictRaised {
    const EVENT_TYPE: &'static str = "notes.conflict.raised.v1";
}

impl ConflictRaised {
    /// Checks every cross-field rule. `Deserialize` calls this; a producer that builds or mutates
    /// a payload calls it before it publishes.
    ///
    /// # Errors
    ///
    /// [`NotesContractError::MergeProposalOnBinary`] and
    /// [`NotesContractError::MergeProposalWithoutThreeSides`].
    pub fn validate(&self) -> Result<(), NotesContractError> {
        if self.merge_proposal.is_none() {
            return Ok(());
        }
        if !MERGEABLE_MEDIA_TYPES.contains(&self.media_type.as_str()) {
            return Err(NotesContractError::MergeProposalOnBinary {
                media_type: self.media_type.as_str().to_owned(),
            });
        }
        if self.base_hash.is_none() || self.current_hash.is_none() || self.writer_hash.is_none() {
            return Err(NotesContractError::MergeProposalWithoutThreeSides);
        }
        Ok(())
    }
}

/// Rewrites the derived object schema of [`ConflictRaised`] as a `oneOf` of three closed shapes:
///
/// 1. `media_type` is mergeable text, `merge_proposal` is present, and the three hashes are
///    present and not `null`;
/// 2. `media_type` is mergeable text and `merge_proposal` is absent or `null`;
/// 3. `media_type` is any other type and `merge_proposal` is absent or `null`.
///
/// A conditional keyword (`if`/`then`) would say the same thing, but the TypeScript projection
/// refuses one at an object root; a `oneOf` of objects projects to a union. The shapes are
/// exclusive: 1 and 2 differ on `merge_proposal`, and 3 differs from both on `media_type`.
fn merge_rule_branches(schema: &mut schemars::Schema) {
    let Some(root) = schema.as_object_mut() else {
        return;
    };
    let Some(serde_json::Value::Object(properties)) = root.remove("properties") else {
        return;
    };
    let required = root
        .remove("required")
        .unwrap_or_else(|| serde_json::json!([]));
    let closed = root.remove("additionalProperties");
    root.remove("type");

    let mergeable: Vec<serde_json::Value> = MERGEABLE_MEDIA_TYPES
        .iter()
        .map(|media_type| serde_json::Value::from(*media_type))
        .collect();
    let restricted = |rule: serde_json::Value| {
        let mut media_type = properties.get("media_type").cloned().unwrap_or_default();
        let description = media_type
            .as_object_mut()
            .and_then(|member| member.remove("description"));
        let mut member = serde_json::json!({ "allOf": [media_type, rule] });
        if let (Some(text), Some(object)) = (description, member.as_object_mut()) {
            object.insert("description".to_owned(), text);
        }
        member
    };
    let branch = |media_type: serde_json::Value, with_proposal: bool| {
        let mut members = properties.clone();
        members.insert("media_type".to_owned(), media_type);
        let mut needed = required.clone();
        if with_proposal {
            for name in ["base_hash", "current_hash", "writer_hash", "merge_proposal"] {
                if let Some(member) = members.get_mut(name) {
                    drop_null_branch(member);
                }
                if let Some(list) = needed.as_array_mut() {
                    list.push(serde_json::Value::from(name));
                }
            }
        } else {
            members.insert(
                "merge_proposal".to_owned(),
                serde_json::json!({
                    "description": "No merge proposal: the shape for a conflict without one.",
                    "type": "null",
                }),
            );
        }
        serde_json::json!({
            "type": "object",
            "properties": members,
            "required": needed,
            "additionalProperties": closed.clone().unwrap_or(serde_json::Value::Bool(false)),
        })
    };
    let text = restricted(serde_json::json!({ "enum": mergeable }));
    let other = restricted(serde_json::json!({ "not": { "enum": mergeable } }));
    root.insert(
        "oneOf".to_owned(),
        serde_json::json!([
            branch(text.clone(), true),
            branch(text, false),
            branch(other, false)
        ]),
    );
}

/// Turns `{"anyOf": [<schema>, {"type": "null"}]}` into `<schema>`, keeping the description.
fn drop_null_branch(member: &mut serde_json::Value) {
    let Some(object) = member.as_object_mut() else {
        return;
    };
    let Some(serde_json::Value::Array(branches)) = object.remove("anyOf") else {
        return;
    };
    for branch in branches {
        if branch.get("type").and_then(serde_json::Value::as_str) == Some("null") {
            continue;
        }
        if let serde_json::Value::Object(inner) = branch {
            object.extend(inner);
        }
    }
}

/// The wire mirror of [`ConflictRaised`], parsed before the cross-field rules run.
///
/// **Drift guard:** a field added to [`ConflictRaised`] and not to this mirror would be refused as
/// unknown on parse; the round trip of the valid fixtures fails the moment that happens. Do not
/// add a field to one of these structs alone.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ConflictRaisedWire {
    conflict_id: NotesConflictId,
    operation_id: OperationId,
    connection_id: NotesConnectionId,
    path: VaultPath,
    origin: WriteOrigin,
    media_type: MediaType,
    #[serde(default)]
    base_hash: Option<DigestHex>,
    #[serde(default)]
    current_hash: Option<DigestHex>,
    #[serde(default)]
    writer_hash: Option<DigestHex>,
    #[serde(default)]
    merge_proposal: Option<MergeProposal>,
}

impl<'de> serde::Deserialize<'de> for ConflictRaised {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ConflictRaisedWire::deserialize(deserializer)?;
        let payload = Self {
            conflict_id: wire.conflict_id,
            operation_id: wire.operation_id,
            connection_id: wire.connection_id,
            path: wire.path,
            origin: wire.origin,
            media_type: wire.media_type,
            base_hash: wire.base_hash,
            current_hash: wire.current_hash,
            writer_hash: wire.writer_hash,
            merge_proposal: wire.merge_proposal,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}

/// Payload of `notes.conflict.resolved.v1` (contract `notes/conflict.v1`): the channel
/// acknowledged the `resolve_conflict` write that names this conflict, and the conflict is closed.
///
/// A fact. The enclosing envelope carries the owner as `tenant_id` (`user:<id>`) and the
/// aggregate `notes_conflict:<conflict_id>`. The payload carries the resulting revision hash and
/// never file content. Every member it does not declare is refused.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ConflictResolved {
    /// The conflict that is closed.
    pub conflict_id: NotesConflictId,

    /// The Platform operation of kind `notes.file.write` of the `resolve_conflict` write.
    pub operation_id: OperationId,

    /// The connection whose vault holds the file.
    pub connection_id: NotesConnectionId,

    /// The path of the file, relative to the vault root, in NFC.
    pub path: VaultPath,

    /// Who asked for the resolution: `platform` or `site:<site id>`.
    pub origin: WriteOrigin,

    /// The lowercase hex SHA-256 of the revision that is current after the resolution. For
    /// `keep_current` it is the revision that was current when the conflict was raised.
    pub revision: DigestHex,
}

impl EventPayload for ConflictResolved {
    const EVENT_TYPE: &'static str = "notes.conflict.resolved.v1";
}
