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

/// One notes contract under test: the Rust type and the generated schema of one message type.
pub(crate) struct ContractUnderTest {
    /// The contract name in the workspace store, e.g. `notes/write-request.v1`.
    pub(crate) contract: &'static str,
    /// The message type that the payload travels as.
    pub(crate) message_type: &'static str,
    /// The generated JSON Schema, relative to the repository root.
    pub(crate) schema: &'static str,
    /// The fixture directory, relative to the repository root.
    pub(crate) fixtures: &'static str,
    /// Parses a payload with the Rust type and serializes the typed value again.
    pub(crate) roundtrip: fn(&serde_json::Value) -> Result<serde_json::Value, String>,
}

/// Every notes contract whose Rust type and generated schema exist.
pub(crate) const CONTRACTS: &[ContractUnderTest] = &[
    ContractUnderTest {
        contract: "notes/write-request.v1",
        message_type: "notes.file.write_requested.v1",
        schema: "schemas/json-schema/commands/write-request.v1.schema.json",
        fixtures: "fixtures/commands/notes.file.write_requested.v1",
        roundtrip: roundtrip_as::<ratatoskr_notes_contracts::FileWriteRequested>,
    },
    ContractUnderTest {
        contract: "notes/conflict.v1",
        message_type: "notes.conflict.raised.v1",
        schema: "schemas/events/notes.conflict.raised.v1.schema.json",
        fixtures: "fixtures/events/notes.conflict.raised.v1",
        roundtrip: roundtrip_as::<ratatoskr_notes_contracts::ConflictRaised>,
    },
    ContractUnderTest {
        contract: "notes/conflict.v1",
        message_type: "notes.conflict.resolved.v1",
        schema: "schemas/events/notes.conflict.resolved.v1.schema.json",
        fixtures: "fixtures/events/notes.conflict.resolved.v1",
        roundtrip: roundtrip_as::<ratatoskr_notes_contracts::ConflictResolved>,
    },
];

/// Parses `value` as `T` and serializes the typed value again.
fn roundtrip_as<T>(value: &serde_json::Value) -> Result<serde_json::Value, String>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let typed: T = serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
    serde_json::to_value(&typed).map_err(|error| error.to_string())
}

/// The contract `contract` of the message type `message_type`. Panics with both names when the
/// Rust type or its generated schema does not exist yet.
pub(crate) fn contract(contract: &str, message_type: &str) -> &'static ContractUnderTest {
    CONTRACTS
        .iter()
        .find(|entry| entry.contract == contract && entry.message_type == message_type)
        .unwrap_or_else(|| {
            panic!("{contract} ({message_type}) is not found: no Rust type and no generated schema")
        })
}

/// The repository root: `crates/notes-contracts` is two levels below it.
fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the crate sits two levels below the repository root")
        .to_path_buf()
}

impl ContractUnderTest {
    /// The fixture `tail` (for example `invalid/two-operations.json`) as JSON.
    pub(crate) fn fixture(&self, tail: &str) -> serde_json::Value {
        let path = repo_root().join(self.fixtures).join(tail);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        serde_json::from_str(&text).unwrap_or_else(|error| panic!("{tail} is not JSON: {error}"))
    }

    /// The generated schema, compiled as draft 2020-12 with format validation.
    pub(crate) fn validator(&self) -> jsonschema::Validator {
        let path = repo_root().join(self.schema);
        let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "{}: no generated schema at {}: {error}",
                self.contract,
                path.display()
            )
        });
        let schema: serde_json::Value = serde_json::from_str(&text).expect("a schema is JSON");
        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema)
            .expect("the generated schema compiles")
    }

    /// Asserts that the Rust type and the generated schema both refuse `invalid/<name>`.
    pub(crate) fn assert_refused_by_both(&self, name: &str) {
        let tail = format!("invalid/{name}");
        let payload = self.fixture(&tail);
        assert!(
            (self.roundtrip)(&payload).is_err(),
            "{}: the Rust type accepts {tail}",
            self.contract
        );
        assert!(
            self.validator().validate(&payload).is_err(),
            "{}: the generated schema accepts {tail}",
            self.contract
        );
    }

    /// Asserts that the Rust type refuses `invalid/<name>` with an error that contains `reason`.
    /// The schema layer is not asked: a cross-field rule is serde only.
    pub(crate) fn assert_refused_by_type(&self, name: &str, reason: &str) {
        let tail = format!("invalid/{name}");
        let payload = self.fixture(&tail);
        match (self.roundtrip)(&payload) {
            Ok(_) => panic!("{}: the Rust type accepts {tail}", self.contract),
            Err(error) => assert!(
                error.contains(reason),
                "{}: {tail} is refused for another reason: {error}",
                self.contract
            ),
        }
    }

    /// Asserts that the Rust type and the generated schema both accept `valid/<name>`, and that
    /// the Rust type gives back every member.
    pub(crate) fn assert_accepted_by_both(&self, name: &str) {
        let tail = format!("valid/{name}");
        let payload = self.fixture(&tail);
        let reemitted = (self.roundtrip)(&payload).unwrap_or_else(|error| {
            panic!("{}: the Rust type refuses {tail}: {error}", self.contract)
        });
        assert_eq!(
            reemitted, payload,
            "{}: {tail} does not round-trip",
            self.contract
        );
        if let Err(error) = self.validator().validate(&payload) {
            panic!(
                "{}: the generated schema refuses {tail}: {error}",
                self.contract
            );
        }
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
