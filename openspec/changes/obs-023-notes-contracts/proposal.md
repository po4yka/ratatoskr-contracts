## Why

Changeset OBS-023 connects Obsidian vaults to Ratatoskr. `ratatoskr-notes` tells Knowledge when a
vault connection becomes active, when it is removed, and when a file in it changes. No wire contract
for these three facts exists. The cross-repository behaviour is defined in the workspace store, in the
change `add-notes-vault-connections`: the spec `notes-vault-connections` (requirement "Connection
events name the connection and never its secrets") and the spec `notes-file-changes`, with design
decision D7 and `docs/NOTES_AND_SITES.md` section 7. This change cites them and does not restate them.

## What Changes

- Add the crate `ratatoskr-notes-contracts` with the event payloads `VaultConnected`
  (`notes.vault.connected.v1`), `VaultDisconnected` (`notes.vault.disconnected.v1`) and
  `FileChanged` (`notes.file.changed.v1`).
- Add the typed ids `NotesConnectionId` and `NotesFileId` to `ratatoskr-identifiers`.
- Register the three events in `contracts.toml` with owner and producer `ratatoskr-notes` and consumer
  `ratatoskr-knowledge`. Add `ratatoskr-notes` to the known services, `notes_connection` and
  `notes_file` to the known entity kinds, and `_sequence` to the numeric-suffix vocabulary.
- Add fixtures, regenerate the JSON Schema and TypeScript, and bless the API baselines.

The change is additive. No existing message, field, schema or API line changes.

## Capabilities

### New Capabilities

- `notes-event-payloads`: the Rust payload types of the three notes events and the rules they enforce
  at parse time.

### Modified Capabilities

<!-- None. -->

## Impact

Touches `Cargo.toml`, `Cargo.lock`, `crates/identifiers`, the new `crates/notes-contracts`,
`contracts.toml`, `fixtures/`, `schemas/events/`, `generated/typescript/events/`, `compat/api/`,
`tools/contractsc` (registry and tests) and the crate lists in the documentation. `ratatoskr-notes`
and `ratatoskr-knowledge` pin the resulting commit in their own OBS-023 tasks. Rollback: revert the
commits before any repository pins them.
