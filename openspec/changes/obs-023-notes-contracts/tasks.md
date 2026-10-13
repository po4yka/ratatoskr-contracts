## 1. Notes event contracts (workspace OBS-023 tasks 4.1 and 4.2)

- [x] 1.1 RED: add the fixtures under `fixtures/events/notes.vault.connected.v1`, `notes.vault.disconnected.v1` and `notes.file.changed.v1` (valid: one per change kind and one per channel; invalid: an unknown field for each type, `deleted` with a blob, `renamed` without `previous_path`, `created` without a blob, a blob digest that differs from the revision, an uppercase and a short revision, a `..` path and a zero sequence), declare each invalid fixture in `fixtures/invalid-expectations.toml`, register the three types in `contracts.toml`, and add `tools/contractsc/tests/fixtures.rs::notes_vault_fixtures_refuse_unknown_fields`. It MUST fail with "ratatoskr_notes_contracts::VaultConnected produces no generated output", because the type and its schema do not exist.
- [x] 1.2 GREEN: add the crate `ratatoskr-notes-contracts` (types, `EventPayload`, errors, crate tests), the ids `NotesConnectionId` and `NotesFileId`, the registry entries and the canonical renderers, run `cargo contracts generate` and `cargo contracts api-write`, and update the pinned root and event counts. `notes_vault_fixtures_refuse_unknown_fields`, the `deleted`-with-blob refusal, `cargo test --test live_message_types` and the gate of `DEVELOPMENT.md` pass.

## 2. Documentation

- [x] 2.1 Add the crate to the crate lists in `README.md`, `DEVELOPMENT.md`, `AGENTS.md` and `docs/ARCHITECTURE.md`. This cannot start from a failing test (documentation).
