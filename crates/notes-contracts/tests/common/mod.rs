//! Shared envelopes and payload builders for the notes-contracts integration tests.

#![allow(
    dead_code,
    reason = "each test binary consumes a different subset of this module"
)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::{
    BlobOwner, BlobRef, ContentDigest, DigestAlgorithm, DigestHex, MediaType, NotesConnectionId,
    NotesFileId,
};
use ratatoskr_notes_contracts::{
    FileChangeKind, FileChanged, VaultChannel, VaultConnected, VaultDisplayName, VaultPath,
};

/// The revision every builder uses; the blob digest equals it.
pub(crate) const REVISION: &str =
    "0a1b2c3d4e5f60710a1b2c3d4e5f60710a1b2c3d4e5f60710a1b2c3d4e5f6071";

/// A legal envelope of one vault connection. The owner is the tenant, and the aggregate is the
/// connection itself; `set_payload` replaces `event_type` and body.
pub(crate) const VAULT_ENVELOPE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000c01",
  "event_type": "notes.vault.connected.v1",
  "occurred_at": "2026-10-13T09:00:00Z",
  "producer": "ratatoskr-notes",
  "aggregate_id": "notes_connection:018f0000-0000-7000-8000-000000000b01",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000c02",
  "tenant_id": "user:018f0000-0000-7000-8000-000000000005",
  "schema_version": 1,
  "payload": {}
}
"#;

/// A legal envelope of one file change. The aggregate is the file.
pub(crate) const FILE_ENVELOPE: &str = r#"{
  "event_id": "018f0000-0000-7000-8000-000000000c03",
  "event_type": "notes.file.changed.v1",
  "occurred_at": "2026-10-13T09:00:01Z",
  "producer": "ratatoskr-notes",
  "aggregate_id": "notes_file:018f0000-0000-7000-8000-000000000b11",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000c02",
  "tenant_id": "user:018f0000-0000-7000-8000-000000000005",
  "schema_version": 1,
  "payload": {}
}
"#;

pub(crate) fn connection_id() -> NotesConnectionId {
    NotesConnectionId::parse("018f0000-0000-7000-8000-000000000b01").unwrap()
}

pub(crate) fn connected(channel: VaultChannel) -> VaultConnected {
    VaultConnected {
        connection_id: connection_id(),
        channel,
        vault_display_name: VaultDisplayName::parse("Personal").unwrap(),
    }
}

pub(crate) fn blob(revision: &str) -> BlobRef {
    BlobRef {
        owner_service: BlobOwner::parse("ratatoskr-notes").unwrap(),
        digest: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            hex: DigestHex::parse(revision).unwrap(),
        },
        media_type: MediaType::parse("text/markdown").unwrap(),
        length_bytes: 2048,
    }
}

/// A rename, which carries **every** member: the drift-guard partner of the private wire mirror.
/// A field added to [`FileChanged`] and not to the mirror is refused as unknown on parse, and the
/// composition test fails the moment that happens.
pub(crate) fn rename_carrying_every_field() -> FileChanged {
    FileChanged {
        connection_id: connection_id(),
        file_id: NotesFileId::parse("018f0000-0000-7000-8000-000000000b11").unwrap(),
        connection_sequence: 3,
        change_kind: FileChangeKind::Renamed,
        path: VaultPath::parse("Archive/Café Plan.md").unwrap(),
        previous_path: Some(VaultPath::parse("Projects/Plan.md").unwrap()),
        revision: DigestHex::parse(REVISION).unwrap(),
        media_type: MediaType::parse("text/markdown").unwrap(),
        blob: Some(blob(REVISION)),
    }
}
