## Purpose

Defines the Rust payload types of `notes.vault.connected.v1`, `notes.vault.disconnected.v1` and
`notes.file.changed.v1`, and the rules they enforce at parse time. The cross-repository meaning of
these events is in the workspace store, change `add-notes-vault-connections`.

## ADDED Requirements

### Requirement: A notes event payload refuses every unknown field

The three payload types SHALL refuse a member that their schema does not declare, at the serde layer
and at the JSON Schema layer.

#### Scenario: A fixture carries an extra field

- **WHEN** an invalid fixture of any of the three types carries a member outside the schema
- **THEN** serde and the published schema both refuse it, verified by `tools/contractsc/tests/fixtures.rs::notes_vault_fixtures_refuse_unknown_fields`

### Requirement: A file change is consistent with its kind

`FileChanged::validate` SHALL require `previous_path` exactly when `change_kind` is `renamed`, `blob`
exactly when `change_kind` is not `deleted`, a blob digest equal to `revision` and owned by
`ratatoskr-notes`, and a `connection_sequence` of at least 1. Deserialization SHALL run it.

#### Scenario: A deleted file carries a blob

- **WHEN** a payload has `change_kind` `deleted` and a `blob`
- **THEN** deserialization fails with `BlobOnDeletedFile`, verified by `crates/notes-contracts/tests/file.rs::inconsistent_file_changes_are_refused_at_parse`

#### Scenario: A rename has no previous path

- **WHEN** a payload has `change_kind` `renamed` and no `previous_path`
- **THEN** deserialization fails with `RenameWithoutPreviousPath`, verified by `crates/notes-contracts/tests/file.rs::inconsistent_file_changes_are_refused_at_parse`

#### Scenario: A path leaves the vault

- **WHEN** `path` holds a `..` segment, starts with `/`, holds a backslash or a control character
- **THEN** deserialization fails, verified by `crates/notes-contracts/tests/file.rs::vault_paths_stay_inside_the_vault`
