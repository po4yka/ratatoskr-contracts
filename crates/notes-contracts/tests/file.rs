//! The file change event, its cross-field rules and its envelope composition.

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes run on values the test itself built, so a \
              missing member panics the test, which is the reporting mechanism"
)]

mod common;

use ratatoskr_event_envelope::{EventEnvelope, EventPayload, EventType};
use ratatoskr_identifiers::dropped_field_pointers;
use ratatoskr_notes_contracts::{FileChangeKind, FileChanged, NotesContractError, VaultPath};

use common::{FILE_ENVELOPE, REVISION, blob, rename_carrying_every_field};

const OTHER_REVISION: &str = "1f2e3d4c5b6a70811f2e3d4c5b6a70811f2e3d4c5b6a70811f2e3d4c5b6a7081";

#[test]
fn event_type_constant_is_the_registered_name() {
    let parsed = EventType::parse(FileChanged::EVENT_TYPE).expect("the registered name parses");
    assert_eq!(parsed.bounded_context(), "notes");
    assert_eq!(parsed.aggregate(), "file");
    assert_eq!(parsed.action(), "changed");
    assert_eq!(parsed.major(), 1);
    assert!(parsed.action_looks_past_tense());
}

/// A rename carrying every member travels inside a real envelope, aggregated on the file, and
/// comes back typed and whole.
#[test]
fn rename_travels_inside_a_real_envelope() {
    let payload = rename_carrying_every_field();

    let mut envelope =
        EventEnvelope::from_json(FILE_ENVELOPE.as_bytes()).expect("the envelope parses");
    envelope.set_payload(&payload).expect("a JSON object body");
    assert_eq!(envelope.event_type, FileChanged::event_type());
    assert_eq!(
        envelope.aggregate_id.to_wire(),
        "notes_file:018f0000-0000-7000-8000-000000000b11"
    );

    let wire = envelope.to_canonical_json().expect("re-serializes");
    let reparsed = EventEnvelope::from_json(wire.as_bytes()).expect("round trips");
    assert_eq!(reparsed, envelope);
    let sent: serde_json::Value = serde_json::from_str(&wire).expect("JSON");
    let received = serde_json::to_value(&reparsed).expect("re-serialize");
    assert_eq!(
        dropped_field_pointers(&sent, &received),
        Vec::<String>::new()
    );
    assert_eq!(
        reparsed.payload_as::<FileChanged>().expect("typed"),
        payload
    );
}

/// Every consistent kind round trips: a deletion without a blob, the others with one.
#[test]
fn every_consistent_kind_round_trips() {
    for kind in [
        FileChangeKind::Created,
        FileChangeKind::Updated,
        FileChangeKind::Deleted,
    ] {
        let mut payload = rename_carrying_every_field();
        payload.change_kind = kind;
        payload.previous_path = None;
        if kind == FileChangeKind::Deleted {
            payload.blob = None;
        }
        payload.validate().expect("consistent");
        let text = serde_json::to_string(&payload).expect("serializes");
        let back: FileChanged = serde_json::from_str(&text).expect("parses");
        assert_eq!(back, payload);
    }
}

/// Every inconsistency is refused at parse time with the error `validate()` names.
#[test]
fn inconsistent_file_changes_are_refused_at_parse() {
    let base = rename_carrying_every_field();
    let mut cases: Vec<(&str, FileChanged, NotesContractError)> = Vec::new();

    let mut deleted_with_blob = base.clone();
    deleted_with_blob.change_kind = FileChangeKind::Deleted;
    deleted_with_blob.previous_path = None;
    cases.push((
        "deleted with blob",
        deleted_with_blob,
        NotesContractError::BlobOnDeletedFile,
    ));

    let mut rename_without_path = base.clone();
    rename_without_path.previous_path = None;
    cases.push((
        "renamed without previous_path",
        rename_without_path,
        NotesContractError::RenameWithoutPreviousPath,
    ));

    let mut update_with_path = base.clone();
    update_with_path.change_kind = FileChangeKind::Updated;
    cases.push((
        "updated with previous_path",
        update_with_path,
        NotesContractError::PreviousPathWithoutRename {
            change_kind: "updated",
        },
    ));

    let mut created_without_blob = base.clone();
    created_without_blob.change_kind = FileChangeKind::Created;
    created_without_blob.previous_path = None;
    created_without_blob.blob = None;
    cases.push((
        "created without blob",
        created_without_blob,
        NotesContractError::MissingBlob {
            change_kind: "created",
        },
    ));

    let mut digest_mismatch = base.clone();
    digest_mismatch.blob = Some(blob(OTHER_REVISION));
    cases.push((
        "blob digest differs",
        digest_mismatch,
        NotesContractError::BlobDigestMismatch,
    ));

    let mut foreign_owner = base.clone();
    let mut foreign_blob = blob(REVISION);
    foreign_blob.owner_service = "ratatoskr-vault".parse().expect("a blob owner");
    foreign_owner.blob = Some(foreign_blob);
    cases.push((
        "foreign blob owner",
        foreign_owner,
        NotesContractError::ForeignBlobOwner {
            owner_service: "ratatoskr-vault".to_owned(),
        },
    ));

    let mut zero_sequence = base;
    zero_sequence.connection_sequence = 0;
    cases.push((
        "zero sequence",
        zero_sequence,
        NotesContractError::ConnectionSequenceZero,
    ));

    for (label, payload, expected) in cases {
        assert_eq!(payload.validate(), Err(expected.clone()), "case `{label}`");
        let text = serde_json::to_string(&payload).expect("serializes");
        let error = serde_json::from_str::<FileChanged>(&text).expect_err(label);
        assert_eq!(error.to_string(), expected.to_string(), "case `{label}`");
    }
}

/// A member outside the schema is refused, so the file text cannot ride along.
#[test]
fn unknown_members_are_refused() {
    let mut value = serde_json::to_value(rename_carrying_every_field()).expect("serializes");
    value["text"] = serde_json::json!("# Plan");
    let error = serde_json::from_value::<FileChanged>(value).expect_err("extra member");
    assert!(
        error.to_string().contains("unknown field `text`"),
        "{error}"
    );
}

/// A vault path is relative and stays inside the vault.
#[test]
fn vault_paths_stay_inside_the_vault() {
    for accepted in [
        "Plan.md",
        "Projects/Plan.md",
        ".obsidian-like/file.md",
        "...md",
        "a/..b/c.",
        "Café.md",
    ] {
        assert!(
            VaultPath::parse(accepted).is_ok(),
            "{accepted:?} is a legal path"
        );
    }
    for refused in [
        "",
        "/Plan.md",
        "Plan.md/",
        "Projects//Plan.md",
        ".",
        "..",
        "./Plan.md",
        "Projects/../Plan.md",
        "Projects/..",
        "Projects\\Plan.md",
        "Plan\u{0}.md",
        "Plan\n.md",
        "Plan\u{7f}.md",
    ] {
        assert!(
            VaultPath::parse(refused).is_err(),
            "{refused:?} must be refused"
        );
    }
    assert!(VaultPath::parse(&"a".repeat(1024)).is_ok());
    assert!(VaultPath::parse(&"a".repeat(1025)).is_err());
}
