//! The write command, its operations and the vocabulary they use.

use std::borrow::Cow;

use ratatoskr_event_envelope::CommandPayload;
use ratatoskr_identifiers::{
    BlobRef, ContentDigest, DigestHex, NotesConflictId, NotesConnectionId, OperationId,
    wire_string_newtype,
};

use crate::error::NotesContractError;
use crate::file::NOTES_BLOB_OWNER;

wire_string_newtype! {
    /// Who asked for a write: `platform` for the Ratatoskr API, or `site:<site id>` for a site in
    /// owner mode.
    ///
    /// A site id has 3 to 40 lower-case ASCII letters, digits and `-`. Every other value is
    /// refused. `ratatoskr-notes` records the origin with the write and repeats it in a conflict
    /// that the write raises.
    pub struct WriteOrigin {
        pattern  = r"^(?:platform|site:[a-z0-9-]{3,40})$",
        max_len  = 45,
        examples = ["platform", "site:docs"],
    }
}

wire_string_newtype! {
    /// A path that a write may change: a [`crate::VaultPath`] whose first segment is not
    /// `.obsidian` in any ASCII case.
    ///
    /// Not empty, at most 1024 UTF-8 bytes, segments separated by `/`, no leading or trailing
    /// `/`, no empty, `.` or `..` segment, no backslash and no C0 or DEL control character, so a
    /// path can never name a file outside the vault. `.obsidian/` is the vault configuration, and
    /// Ratatoskr never writes it; a case-insensitive file system maps `.OBSIDIAN/` to the same
    /// folder, so every case is refused. The pattern spells the first segment without
    /// look-ahead: any segment, except one that starts with `.o` and continues with exactly
    /// `bsidian`.
    ///
    /// The path is in Unicode NFC. This contract does not check NFC: the producer normalises
    /// every path before it sends one.
    pub struct VaultWritePath {
        pattern  = concat!(
            "^(?:",
            // A first segment that does not start with a dot.
            r"[^./\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*",
            // `.` and a character other than `.`, `o` or `O`.
            r"|\.[^./oO\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*",
            // `.o` and anything except exactly `bsidian` in any case.
            r"|\.[oO](?:|[^bB/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*|[bB]",
            r"(?:|[^sS/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*|[sS]",
            r"(?:|[^iI/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*|[iI]",
            r"(?:|[^dD/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*|[dD]",
            r"(?:|[^iI/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*|[iI]",
            r"(?:|[^aA/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*|[aA]",
            r"(?:|[^nN/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*|[nN][^/\\\x00-\x1f\x7f]+)))))))",
            // `..` and at least one more character.
            r"|\.\.[^/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*",
            ")",
            // Every later segment follows the `VaultPath` grammar.
            r"(?:/(?:[^./\\\x00-\x1f\x7f]|\.[^./\\\x00-\x1f\x7f]|\.\.[^/\\\x00-\x1f\x7f])",
            r"[^/\\\x00-\x1f\x7f]*)*$",
        ),
        max_len  = 1024,
        examples = ["Projects/Plan.md", "Archive/2026/Plan.md"],
    }
}

wire_string_newtype! {
    /// The name of one frontmatter property of a Markdown file, as the owner wrote it.
    ///
    /// 1 to 255 characters with no C0 or DEL control character. The pattern carries the
    /// 255-character bound; the length guard is 1020 UTF-8 bytes, the most 255 characters can
    /// take.
    pub struct PropertyKey {
        pattern  = r"^[^\x00-\x1f\x7f]{1,255}$",
        max_len  = 1020,
        examples = ["tags", "status"],
    }
}

/// The media type of the uploaded bytes of a frontmatter value: one JSON value.
const PROPERTY_VALUE_MEDIA_TYPE: &str = "application/json";

/// What a `set_property` operation does to its property.
///
/// A frontmatter value is note content, so it never travels in the command. The caller uploads
/// the value as one JSON document (`application/json`) to `ratatoskr-notes` first, and `set`
/// names the resulting [`BlobRef`]; `ratatoskr-notes` writes the value as YAML.
///
/// **Closed on purpose**: a consumer that does not know a change must refuse the write rather
/// than guess.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PropertyChange {
    /// Set the property to the uploaded JSON value, and add the property when the file has none.
    Set(BlobRef),
    /// Remove the property from the frontmatter.
    Remove,
}

impl schemars::JsonSchema for PropertyChange {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("PropertyChange")
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::PropertyChange"))
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // The derived shape, plus the media type of the blob, which the shared `BlobRef` schema
        // cannot state for one use. The blob owner stays a rule of the Rust type.
        let blob = generator.subschema_for::<BlobRef>();
        schemars::json_schema!({
            "description": "What a `set_property` operation does to its property. A frontmatter \
                            value is note content, so the caller uploads it as one JSON document \
                            (`application/json`) to ratatoskr-notes first, and `set` names the \
                            resulting BlobRef.",
            "oneOf": [
                {
                    "type": "object",
                    "description": "Set the property to the uploaded JSON value, and add the \
                                    property when the file has none.",
                    "properties": {
                        "set": {
                            "allOf": [
                                blob,
                                {
                                    "type": "object",
                                    "properties": {
                                        "media_type": {
                                            "type": "string",
                                            "const": PROPERTY_VALUE_MEDIA_TYPE,
                                        },
                                    },
                                    "required": ["media_type"],
                                },
                            ],
                        },
                    },
                    "required": ["set"],
                    "additionalProperties": false,
                },
                {
                    "type": "string",
                    "const": "remove",
                    "description": "Remove the property from the frontmatter.",
                },
            ],
        })
    }
}

/// How the owner resolves a conflict.
///
/// **Closed on purpose**: each choice selects different bytes, and an unknown choice must stop
/// the write instead of being read as one of them.
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
pub enum ResolutionChoice {
    /// Keep the current revision of the vault.
    KeepCurrent,
    /// Write the writer's content of the conflict.
    KeepWriter,
    /// Write the merge proposal of the conflict. Notes refuses it for a conflict without one.
    AcceptMerge,
    /// Write content that the owner uploaded for this resolution, named by `blob`.
    Custom,
}

impl ResolutionChoice {
    /// The wire word for this choice.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeepCurrent => "keep_current",
            Self::KeepWriter => "keep_writer",
            Self::AcceptMerge => "accept_merge",
            Self::Custom => "custom",
        }
    }
}

/// `create`: add a file at a path that no live file holds.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct CreateFile {
    /// The path of the new file, relative to the vault root, in NFC.
    pub path: VaultWritePath,

    /// The content of the new file, uploaded first to `ratatoskr-notes`.
    pub blob: BlobRef,
}

/// `update`: replace the bytes of a file.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct UpdateFile {
    /// The path of the file, relative to the vault root, in NFC.
    pub path: VaultWritePath,

    /// The lowercase hex SHA-256 of the revision that the writer read and changed.
    pub base_hash: DigestHex,

    /// The new content of the file, uploaded first to `ratatoskr-notes`.
    pub blob: BlobRef,
}

/// `rename` and `move`: give a file a new path. Links to the file are rewritten.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct RelocateFile {
    /// The path of the file before the operation, relative to the vault root, in NFC.
    pub path: VaultWritePath,

    /// The lowercase hex SHA-256 of the revision that the writer read.
    pub base_hash: DigestHex,

    /// The path of the file after the operation, relative to the vault root, in NFC.
    pub target_path: VaultWritePath,
}

/// `delete`: move a file to the vault trash.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct DeleteFile {
    /// The path of the file, relative to the vault root, in NFC.
    pub path: VaultWritePath,

    /// The lowercase hex SHA-256 of the revision that the writer read.
    pub base_hash: DigestHex,
}

/// `set_property`: set or remove one frontmatter property of a Markdown file.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SetFileProperty {
    /// The path of the file, relative to the vault root, in NFC.
    pub path: VaultWritePath,

    /// The lowercase hex SHA-256 of the revision that the writer read.
    pub base_hash: DigestHex,

    /// The frontmatter property to change.
    pub property_key: PropertyKey,

    /// `{"set": <BlobRef of an application/json value>}` to set the property, or `"remove"` to
    /// remove it.
    pub property_change: PropertyChange,
}

/// `resolve_conflict`: end an open conflict with one choice.
///
/// `blob` is present exactly when `choice` is `custom`. The published schema states this as a
/// `oneOf` of the two shapes. There is no `base_hash`: `ratatoskr-notes` checks the resolution
/// against the current revision that the conflict recorded.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolveConflict {
    /// The conflict to resolve.
    pub conflict_id: NotesConflictId,

    /// How the owner resolves it.
    pub choice: ResolutionChoice,

    /// The content to write, uploaded first to `ratatoskr-notes`. Present exactly when `choice`
    /// is `custom`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<BlobRef>,
}

impl ResolveConflict {
    /// Checks that `blob` is present exactly when `choice` is `custom`, and that it belongs to
    /// `ratatoskr-notes`.
    ///
    /// # Errors
    ///
    /// [`NotesContractError::CustomResolutionWithoutBlob`],
    /// [`NotesContractError::BlobWithoutCustomResolution`],
    /// [`NotesContractError::ForeignBlobOwner`].
    pub fn validate(&self) -> Result<(), NotesContractError> {
        match (self.choice, &self.blob) {
            (ResolutionChoice::Custom, None) => {
                Err(NotesContractError::CustomResolutionWithoutBlob)
            }
            (ResolutionChoice::Custom, Some(blob)) => check_write_blob(blob),
            (choice, Some(_)) => Err(NotesContractError::BlobWithoutCustomResolution {
                choice: choice.as_str(),
            }),
            (_, None) => Ok(()),
        }
    }
}

impl schemars::JsonSchema for ResolveConflict {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("ResolveConflict")
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::ResolveConflict"))
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // Two closed shapes, so the schema refuses `custom` without a blob and a blob on any
        // other choice. A `oneOf` of objects projects to a TypeScript union.
        let conflict_id = generator.subschema_for::<NotesConflictId>();
        let blob = generator.subschema_for::<BlobRef>();
        schemars::json_schema!({
            "description": "`resolve_conflict`: end an open conflict with one choice. `blob` is \
                            present exactly when `choice` is `custom`. There is no `base_hash`: \
                            ratatoskr-notes checks the resolution against the current revision \
                            that the conflict recorded.",
            "oneOf": [
                {
                    "type": "object",
                    "description": "Keep the current revision, write the writer's content, or \
                                    write the merge proposal.",
                    "properties": {
                        "conflict_id": conflict_id,
                        "choice": {
                            "type": "string",
                            "enum": ["keep_current", "keep_writer", "accept_merge"],
                        },
                    },
                    "required": ["conflict_id", "choice"],
                    "additionalProperties": false,
                },
                {
                    "type": "object",
                    "description": "Write content that the owner uploaded for this resolution.",
                    "properties": {
                        "conflict_id": conflict_id,
                        "choice": { "type": "string", "const": "custom" },
                        "blob": blob,
                    },
                    "required": ["conflict_id", "choice", "blob"],
                    "additionalProperties": false,
                },
            ],
        })
    }
}

/// The one change that a write asks for. On the wire it is an object with exactly one member,
/// whose name is the operation kind.
///
/// **Closed on purpose**: a consumer that does not know an operation must refuse the write rather
/// than guess what it changes.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WriteOperation {
    /// Add a file.
    Create(CreateFile),
    /// Replace the bytes of a file.
    Update(UpdateFile),
    /// Give a file a new name.
    Rename(RelocateFile),
    /// Give a file a new folder.
    Move(RelocateFile),
    /// Move a file to the vault trash.
    Delete(DeleteFile),
    /// Set or remove one frontmatter property.
    SetProperty(SetFileProperty),
    /// End an open conflict.
    ResolveConflict(ResolveConflict),
}

impl WriteOperation {
    /// The wire word for this operation kind.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Rename(_) => "rename",
            Self::Move(_) => "move",
            Self::Delete(_) => "delete",
            Self::SetProperty(_) => "set_property",
            Self::ResolveConflict(_) => "resolve_conflict",
        }
    }
}

/// Payload of `notes.file.write_requested.v1` (contract `notes/write-request.v1`): change one
/// file of a connected vault.
///
/// A command. Edge is its only producer and `ratatoskr-notes` its only consumer. The enclosing
/// envelope carries the owner as `tenant_id` (`user:<id>`) and the aggregate
/// `operation:<operation_id>`.
///
/// Content never travels in the command. The caller uploads it first to
/// `/v1/notes/connections/{connection_id}/blobs`, and the operation names the resulting
/// [`BlobRef`], owned by `ratatoskr-notes`. Every member that the schema does not declare is
/// refused, so no note text can ride along.
///
/// `Deserialize` is hand-written: it parses a private mirror struct and then calls
/// [`FileWriteRequested::validate`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileWriteRequested {
    /// The Platform operation of kind `notes.file.write` that this write reports to.
    pub operation_id: OperationId,

    /// SHA-256 of the caller's `Idempotency-Key`. A second command with the same key and
    /// connection does not queue its operation again.
    pub idempotency_key: ContentDigest,

    /// The connection whose vault the write changes.
    pub connection_id: NotesConnectionId,

    /// Who asked for the write: `platform` or `site:<site id>`.
    pub origin: WriteOrigin,

    /// The one change that the write asks for.
    pub operation: WriteOperation,
}

impl CommandPayload for FileWriteRequested {
    const COMMAND_TYPE: &'static str = "notes.file.write_requested.v1";
}

impl FileWriteRequested {
    /// Checks every cross-field rule. `Deserialize` calls this; a producer that builds or mutates
    /// a payload calls it before it publishes.
    ///
    /// # Errors
    ///
    /// [`NotesContractError::ForeignBlobOwner`] when a blob does not belong to
    /// `ratatoskr-notes`, [`NotesContractError::PropertyBlobNotJson`] when the blob of a
    /// frontmatter value is not `application/json`, and the errors of
    /// [`ResolveConflict::validate`].
    pub fn validate(&self) -> Result<(), NotesContractError> {
        match &self.operation {
            WriteOperation::Create(create) => check_write_blob(&create.blob),
            WriteOperation::Update(update) => check_write_blob(&update.blob),
            WriteOperation::SetProperty(property) => match &property.property_change {
                PropertyChange::Set(blob) => check_property_blob(blob),
                PropertyChange::Remove => Ok(()),
            },
            WriteOperation::ResolveConflict(resolution) => resolution.validate(),
            WriteOperation::Rename(_) | WriteOperation::Move(_) | WriteOperation::Delete(_) => {
                Ok(())
            }
        }
    }
}

/// Write content is uploaded to `ratatoskr-notes`, so every blob of a write belongs to it.
fn check_write_blob(blob: &BlobRef) -> Result<(), NotesContractError> {
    if blob.owner_service.as_str() == NOTES_BLOB_OWNER {
        Ok(())
    } else {
        Err(NotesContractError::ForeignBlobOwner {
            owner_service: blob.owner_service.as_str().to_owned(),
        })
    }
}

/// The blob of a frontmatter value belongs to `ratatoskr-notes` and holds one JSON value.
fn check_property_blob(blob: &BlobRef) -> Result<(), NotesContractError> {
    check_write_blob(blob)?;
    if blob.media_type.as_str() == PROPERTY_VALUE_MEDIA_TYPE {
        Ok(())
    } else {
        Err(NotesContractError::PropertyBlobNotJson {
            media_type: blob.media_type.as_str().to_owned(),
        })
    }
}

/// The wire mirror of [`FileWriteRequested`], parsed before the cross-field rules run.
///
/// **Drift guard:** a field added to [`FileWriteRequested`] and not to this mirror would be
/// refused as unknown on parse; the round trip of the valid fixtures fails the moment that
/// happens. Do not add a field to one of these structs alone.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileWriteRequestedWire {
    operation_id: OperationId,
    idempotency_key: ContentDigest,
    connection_id: NotesConnectionId,
    origin: WriteOrigin,
    operation: WriteOperation,
}

impl<'de> serde::Deserialize<'de> for FileWriteRequested {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = FileWriteRequestedWire::deserialize(deserializer)?;
        let payload = Self {
            operation_id: wire.operation_id,
            idempotency_key: wire.idempotency_key,
            connection_id: wire.connection_id,
            origin: wire.origin,
            operation: wire.operation,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}
