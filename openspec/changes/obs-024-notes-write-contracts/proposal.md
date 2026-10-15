## Why

Changeset OBS-024 lets Ratatoskr change files in a connected Obsidian vault. Edge sends a write
command to `ratatoskr-notes`, and `ratatoskr-notes` tells Edge when a write ends in a conflict and
when the owner resolves it. No wire contract for these three messages exists. The cross-repository
behaviour is defined in the workspace store, change `add-notes-write-back`: the spec
`notes-write-command` and design decisions D1 to D4. This change cites them and does not restate
them.

## What Changes

- Add to `ratatoskr-notes-contracts` the command payload `FileWriteRequested`
  (`notes.file.write_requested.v1`, contract name `notes/write-request.v1`) with `origin`, the seven
  operations and the content as a `BlobRef` owned by `ratatoskr-notes`.
- Add the event payloads `ConflictRaised` (`notes.conflict.raised.v1`) and `ConflictResolved`
  (`notes.conflict.resolved.v1`), together the contract `notes/conflict.v1`. They carry hashes,
  never file content.
- Add the typed id `NotesConflictId` to `ratatoskr-identifiers`.
- Register the three messages in `contracts.toml`: the command with producer `ratatoskr-platform` and
  consumer `ratatoskr-notes`, the two events with producer `ratatoskr-notes` and consumer
  `ratatoskr-platform`. Add `notes_conflict` to the known entity kinds.
- Add fixtures, regenerate the JSON Schema and TypeScript, and update the pinned counts.

The change is additive. No existing message, field, schema or API line changes.

## Capabilities

### New Capabilities

- `notes-write-payloads`: the Rust payload types of the write command and the two conflict events,
  and the rules they enforce at parse time.

### Modified Capabilities

<!-- None. -->

## Impact

Touches `crates/identifiers`, `crates/notes-contracts`, `contracts.toml`, `fixtures/`,
`schemas/json-schema/commands/`, `schemas/events/`, `generated/typescript/`, `tools/contractsc`
(registry and tests) and `README.md`. `ratatoskr-notes`, `ratatoskr-platform`,
`ratatoskr-obsidian-plugin`, `ratatoskr-export-agent` and `ratatoskr-web` pin the resulting commit
in their own OBS-024 tasks. Rollback: revert the commits before any repository pins them.
