## 1. Write request (workspace OBS-024 tasks 3.1 and 3.2)

- [x] 1.1 RED: add `crates/notes-contracts/tests/write_request.rs` with `two_operations_are_refused`, `origin_has_two_forms`, `update_requires_base_hash`, `custom_resolution_requires_content`, `paths_stay_inside_the_vault` and `content_is_a_blob_ref`, the invalid fixtures `two-operations.json`, `empty-site-origin.json`, `unknown-origin.json`, `update-without-base-hash.json`, `custom-resolution-without-content.json`, `parent-segment.json`, `absolute-path.json`, `config-folder.json` and `inline-content.json`, and one valid fixture per operation kind under `fixtures/commands/notes.file.write_requested.v1`. Each test asserts that serde and the generated schema both refuse its fixtures. It MUST fail with "notes/write-request.v1 is not found", because the type and its schema do not exist.
- [x] 1.2 GREEN: add `FileWriteRequested` and its operation types, `NotesConflictId`, the row `notes.write_request` (the generator reads it) with its fixture expectations, the registry entries and the canonical renderer, run `cargo contracts generate`, and update the pinned counts. 1.1 passes, and the generated TypeScript matches (`cargo contracts check`).

## 2. Conflict events (workspace OBS-024 tasks 3.3 and 3.4)

- [x] 2.1 RED: add `crates/notes-contracts/tests/conflict.rs::attachment_conflict_with_proposal_is_refused` and `text_conflict_round_trips`, with the fixtures under `fixtures/events/notes.conflict.raised.v1` and `fixtures/events/notes.conflict.resolved.v1`. They MUST fail with "notes/conflict.v1 is not found".
- [x] 2.2 GREEN: add `ConflictRaised` and `ConflictResolved` (hashes only, with `origin`), their rows, expectations, registry entries and renderers, regenerate, and update the pinned counts. 2.1 passes.

## 3. Registry (workspace OBS-024 tasks 3.5 and 3.6)

- [ ] 3.1 Add `tools/contractsc/tests/live_message_types.rs::notes_write_request_has_one_producer`, asserting the producer and consumer of the three messages.
- [ ] 3.2 Document the three messages in the `README.md` subject table.
