## 1. Write request (workspace OBS-024 tasks 3.1 and 3.2)

- [x] 1.1 RED: add `crates/notes-contracts/tests/write_request.rs` with `two_operations_are_refused`, `origin_has_two_forms`, `update_requires_base_hash`, `custom_resolution_requires_content`, `paths_stay_inside_the_vault` and `content_is_a_blob_ref`, the invalid fixtures `two-operations.json`, `empty-site-origin.json`, `unknown-origin.json`, `update-without-base-hash.json`, `custom-resolution-without-content.json`, `parent-segment.json`, `absolute-path.json`, `config-folder.json` and `inline-content.json`, and one valid fixture per operation kind under `fixtures/commands/notes.file.write_requested.v1`. Each test asserts that serde and the generated schema both refuse its fixtures. It MUST fail with "notes/write-request.v1 is not found", because the type and its schema do not exist.
- [x] 1.2 GREEN: add `FileWriteRequested` and its operation types, `NotesConflictId`, the row `notes.write_request` (the generator reads it) with its fixture expectations, the registry entries and the canonical renderer, run `cargo contracts generate`, and update the pinned counts. 1.1 passes, and the generated TypeScript matches (`cargo contracts check`).

## 2. Conflict events (workspace OBS-024 tasks 3.3 and 3.4)

- [x] 2.1 RED: add `crates/notes-contracts/tests/conflict.rs::attachment_conflict_with_proposal_is_refused` and `text_conflict_round_trips`, with the fixtures under `fixtures/events/notes.conflict.raised.v1` and `fixtures/events/notes.conflict.resolved.v1`. They MUST fail with "notes/conflict.v1 is not found".
- [x] 2.2 GREEN: add `ConflictRaised` and `ConflictResolved` (hashes only, with `origin`), their rows, expectations, registry entries and renderers, regenerate, and update the pinned counts. 2.1 passes.

## 3. Registry (workspace OBS-024 tasks 3.5 and 3.6)

- [x] 3.1 Add `tools/contractsc/tests/live_message_types.rs::notes_write_request_has_one_producer`, asserting the producer and consumer of the three messages. This cannot start from a failing test: the rows land with the types in 1.2 and 2.2, because `cargo contracts generate` reads them. A second producer on the command row makes it fail and name the list.
- [x] 3.2 Document the three messages in the `README.md` subject table.

## 4. Review corrections (workspace OBS-024 tasks 3.7 to 3.10)

- [x] 4.1 RED: add `crates/notes-contracts/tests/write_request.rs::property_value_is_a_blob_ref`, change `valid/set-property.json` to `{"set": <BlobRef, application/json>}`, and add the invalid fixtures `inline-property-value.json` and `property-value-not-json.json` with their expectations. It MUST fail with "the Rust type accepts invalid/inline-property-value.json", because `property_change.set` takes any JSON value.
- [x] 4.2 GREEN: `property_change` is `{"set": <BlobRef>}` or `"remove"`; the schema requires `media_type` `application/json`, and `validate()` also requires the owner `ratatoskr-notes`. Remove `PropertyValue`. 4.1 passes.
- [x] 4.3 RED: in `crates/notes-contracts/tests/conflict.rs`, make `attachment_conflict_with_proposal_is_refused` assert both layers, add `proposal_requires_three_hashes`, and declare `json_schema` for `attachment-with-proposal.json` and `proposal-without-base.json`. They MUST fail with "the generated schema accepts", because the merge-proposal rules are serde only.
- [x] 4.4 GREEN: the `ConflictRaised` schema is a `oneOf` of three closed shapes (text with a proposal and three hashes, text without a proposal, any other media type without a proposal). 4.3 passes, and the TypeScript projects the union.
- [x] 4.5 Add `crates/notes-contracts/tests/write_request.rs::config_folder_is_refused_in_any_case` with the invalid fixtures `config-folder-uppercase.json` (`.OBSIDIAN/app.json`), `config-folder-mixed-case.json` (`.Obsidian/x`) and `config-folder-bare.json` (`.obsidian`), refused by serde and the schema, and the valid fixture `obsidian-like-folder.json` (`.obsidian-archive/Plan.md`). No failing test first: the rule exists since 1.2. A case-sensitive pattern and a pattern that accepts the bare name each make it fail.
- [ ] 4.6 Correct the `reason` of `events/notes.conflict.raised.v1/invalid/attachment-with-proposal.json`: the schema refuses it through its `oneOf` since 4.4. No test: an expectation text.
