//! The file change event, its change kinds and the vault path grammar.

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{
    BlobRef, DigestAlgorithm, DigestHex, MediaType, NotesConnectionId, NotesFileId,
    wire_string_newtype,
};

use crate::error::NotesContractError;

/// The only service that stores note bytes, and so the only legal `blob.owner_service`.
pub(crate) const NOTES_BLOB_OWNER: &str = "ratatoskr-notes";

wire_string_newtype! {
    /// A file path relative to the vault root, segments separated by `/`.
    ///
    /// Not empty, at most 1024 UTF-8 bytes, no leading or trailing `/`, no empty, `.` or `..`
    /// segment, no backslash and no C0 or DEL control character, so a path can never name a file
    /// outside the vault.
    ///
    /// The path is in Unicode NFC. This contract does not check NFC: the producer normalises
    /// every path before it publishes one, and a consumer may rely on it.
    pub struct VaultPath {
        pattern  = r"^(?:[^./\\\x00-\x1f\x7f]|\.[^./\\\x00-\x1f\x7f]|\.\.[^/\\\x00-\x1f\x7f])[^/\\\x00-\x1f\x7f]*(?:/(?:[^./\\\x00-\x1f\x7f]|\.[^./\\\x00-\x1f\x7f]|\.\.[^/\\\x00-\x1f\x7f])[^/\\\x00-\x1f\x7f]*)*$",
        max_len  = 1024,
        examples = ["Projects/Plan.md", "Daily/2026-10-13.md"],
    }
}

/// What happened to the file.
///
/// **Closed on purpose**: a consumer branches on the kind to index, move or forget a file, and an
/// unknown kind must stop processing instead of being read as one of them.
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
pub enum FileChangeKind {
    /// The file appeared in the vault.
    Created,
    /// The bytes of the file changed under the same path.
    Updated,
    /// The file moved from `previous_path` to `path`.
    Renamed,
    /// The file left the vault, or became excluded.
    Deleted,
}

impl FileChangeKind {
    /// The wire word for this kind.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Renamed => "renamed",
            Self::Deleted => "deleted",
        }
    }
}

/// Payload of `notes.file.changed.v1`: one file of a connected vault was created, updated,
/// renamed or deleted.
///
/// A fact. The enclosing envelope carries the owner as `tenant_id` (`user:<id>`) and the
/// aggregate `notes_file:<file_id>`; the payload does not repeat the owner. It carries a
/// [`BlobRef`] for the bytes, never the bytes or any part of the file text, and it refuses every
/// member it does not declare.
///
/// `Deserialize` is hand-written because the rules are cross-field: `previous_path` is present
/// exactly when the kind is `renamed`, `blob` is present exactly when the kind is not `deleted`,
/// the blob digest equals `revision` and the blob belongs to `ratatoskr-notes`, and
/// `connection_sequence` is at least 1. It parses a private mirror struct and then calls
/// [`FileChanged::validate`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileChanged {
    /// The connection the file belongs to.
    pub connection_id: NotesConnectionId,

    /// The file, stable across updates and renames.
    pub file_id: NotesFileId,

    /// The position of this event among the published events of its connection. Starts at 1 for
    /// each connection and grows by exactly one per published event, so a consumer can detect a
    /// missing event.
    #[schemars(range(min = 1))]
    pub connection_sequence: u64,

    /// What happened to the file.
    pub change_kind: FileChangeKind,

    /// The path of the file after the change, relative to the vault root, in NFC. For `deleted`,
    /// the path the file had when it left the vault.
    pub path: VaultPath,

    /// The path before the change. Present exactly when `change_kind` is `renamed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_path: Option<VaultPath>,

    /// The lowercase hex SHA-256 of the file bytes. For `deleted`, the revision the file had when
    /// it left the vault.
    pub revision: DigestHex,

    /// The media type of the file bytes.
    pub media_type: MediaType,

    /// The stored bytes of this revision, owned by `ratatoskr-notes`; its SHA-256 digest equals
    /// `revision` and its media type equals `media_type`. Present exactly when `change_kind` is
    /// not `deleted`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<BlobRef>,
}

impl EventPayload for FileChanged {
    const EVENT_TYPE: &'static str = "notes.file.changed.v1";
}

impl FileChanged {
    /// Checks every cross-field rule. `Deserialize` calls this; a producer that builds or mutates
    /// a payload calls it before it publishes.
    ///
    /// # Errors
    ///
    /// [`NotesContractError::ConnectionSequenceZero`],
    /// [`NotesContractError::RenameWithoutPreviousPath`],
    /// [`NotesContractError::PreviousPathWithoutRename`],
    /// [`NotesContractError::BlobOnDeletedFile`], [`NotesContractError::MissingBlob`],
    /// [`NotesContractError::ForeignBlobOwner`], [`NotesContractError::BlobDigestMismatch`],
    /// [`NotesContractError::BlobMediaTypeMismatch`].
    pub fn validate(&self) -> Result<(), NotesContractError> {
        if self.connection_sequence == 0 {
            return Err(NotesContractError::ConnectionSequenceZero);
        }

        let change_kind = self.change_kind.as_str();
        match (self.change_kind, &self.previous_path) {
            (FileChangeKind::Renamed, None) => {
                return Err(NotesContractError::RenameWithoutPreviousPath);
            }
            (FileChangeKind::Renamed, Some(_)) | (_, None) => {}
            (_, Some(_)) => {
                return Err(NotesContractError::PreviousPathWithoutRename { change_kind });
            }
        }

        match (self.change_kind, &self.blob) {
            (FileChangeKind::Deleted, Some(_)) => Err(NotesContractError::BlobOnDeletedFile),
            (FileChangeKind::Deleted, None) => Ok(()),
            (_, None) => Err(NotesContractError::MissingBlob { change_kind }),
            (_, Some(blob)) => self.check_blob(blob),
        }
    }

    /// The blob belongs to `ratatoskr-notes`, names exactly the bytes of `revision` and has the
    /// media type of the file.
    fn check_blob(&self, blob: &BlobRef) -> Result<(), NotesContractError> {
        if blob.owner_service.as_str() != NOTES_BLOB_OWNER {
            return Err(NotesContractError::ForeignBlobOwner {
                owner_service: blob.owner_service.as_str().to_owned(),
            });
        }
        if blob.digest.algorithm != DigestAlgorithm::Sha256 || blob.digest.hex != self.revision {
            return Err(NotesContractError::BlobDigestMismatch);
        }
        if blob.media_type != self.media_type {
            return Err(NotesContractError::BlobMediaTypeMismatch);
        }
        Ok(())
    }
}

/// The wire mirror of [`FileChanged`], parsed before the cross-field rules run.
///
/// **Drift guard:** a field added to [`FileChanged`] and not to this mirror would be refused as
/// unknown on parse; the round-trip test of a payload carrying every field fails the moment that
/// happens. Do not add a field to one of these structs alone.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileChangedWire {
    connection_id: NotesConnectionId,
    file_id: NotesFileId,
    connection_sequence: u64,
    change_kind: FileChangeKind,
    path: VaultPath,
    #[serde(default)]
    previous_path: Option<VaultPath>,
    revision: DigestHex,
    media_type: MediaType,
    #[serde(default)]
    blob: Option<BlobRef>,
}

impl<'de> serde::Deserialize<'de> for FileChanged {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = FileChangedWire::deserialize(deserializer)?;
        let payload = Self {
            connection_id: wire.connection_id,
            file_id: wire.file_id,
            connection_sequence: wire.connection_sequence,
            change_kind: wire.change_kind,
            path: wire.path,
            previous_path: wire.previous_path,
            revision: wire.revision,
            media_type: wire.media_type,
            blob: wire.blob,
        };
        payload.validate().map_err(serde::de::Error::custom)?;
        Ok(payload)
    }
}
