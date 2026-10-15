## Purpose

Defines the Rust payload types of `notes.file.write_requested.v1`, `notes.conflict.raised.v1` and
`notes.conflict.resolved.v1`, and the rules they enforce at parse time. The cross-repository meaning
of these messages is in the workspace store, change `add-notes-write-back`, spec
`notes-write-command`.

## ADDED Requirements

### Requirement: A write request carries exactly one valid operation

`FileWriteRequested` SHALL refuse, at the serde layer and at the JSON Schema layer, a payload with
two operations, an origin outside `platform` and `site:<site id>`, an `update` without `base_hash`, a
`custom` resolution without a blob, a path that leaves the vault or names `.obsidian/`, and content
that is not a `BlobRef`.

#### Scenario: A payload carries two operations

- **WHEN** the operation object holds both `rename` and `delete`
- **THEN** serde and the published schema both refuse it, verified by `crates/notes-contracts/tests/write_request.rs::two_operations_are_refused`

#### Scenario: A path names the configuration folder

- **WHEN** a path is `.obsidian/app.json`
- **THEN** serde and the published schema both refuse it, verified by `crates/notes-contracts/tests/write_request.rs::paths_stay_inside_the_vault`

### Requirement: A conflict proposes a merge only for text

`ConflictRaised::validate` SHALL refuse a `merge_proposal` when `media_type` is not `text/markdown`
or `text/plain`. Deserialization SHALL run it.

#### Scenario: An attachment conflict carries a proposal

- **WHEN** a conflict for `image/png` carries a `merge_proposal`
- **THEN** deserialization fails, verified by `crates/notes-contracts/tests/conflict.rs::attachment_conflict_with_proposal_is_refused`
