## Context

The behaviour is defined by the workspace store (change `add-notes-vault-connections`, specs
`notes-vault-connections` and `notes-file-changes`, design D7). `crates/backup-contracts` and
`crates/notification-contracts` are the models for a crate with event payloads.
`PolicyAcknowledged` is the model for cross-field rules in a hand-written `Deserialize`.

## Decisions

- **The envelope carries the owner.** The envelope's `tenant_id` is `user:<id>`. The payloads do not
  repeat it. The aggregate is `notes_connection:<uuid>` for the vault events and `notes_file:<uuid>`
  for `notes.file.changed.v1`.
- **Unknown fields are refused** (`unknown_policy = "reject"`, `deny_unknown_fields`). The workspace
  spec requires that a fixture with any field outside the schema fails. This is a privacy rule: the
  events must never carry vault content. The payloads therefore have no `extensions` map.
- **Closed enums.** `VaultChannel` (`sync`, `plugin`, `cli`) and `FileChangeKind` (`created`,
  `updated`, `renamed`, `deleted`) are closed and `#[non_exhaustive]`, like `PolicyOutcome`.
- **Paths.** `VaultPath` is a validated string: not empty, at most 1024 UTF-8 bytes, segments
  separated by `/`, no leading or trailing `/`, no empty, `.` or `..` segment, no backslash and no C0
  or DEL control character. The workspace has no Unicode normalisation crate, so NFC is the
  producer's duty and is documented on the type, not checked.
- **Revision and blob.** `revision` is a `DigestHex` (lowercase hex, 64 characters). `blob` is the
  existing `BlobRef`. When it is present, its SHA-256 digest equals `revision` and its owner is
  `ratatoskr-notes`. `media_type` reuses `MediaType`.
- **Cross-field rules** run in `validate()`, which the hand-written `Deserialize` calls:
  `previous_path` is present exactly when the kind is `renamed`; `blob` is present exactly when the
  kind is not `deleted`; the blob digest equals the revision; `connection_sequence` is at least 1.
- **Schema layer.** The schema publishes what it can state without a conditional keyword:
  `additionalProperties: false`, the path and digest patterns, and `minimum: 1` on the sequence. The
  TypeScript projection refuses `if`/`then` at an object root, so the cross-field rules are serde
  only, and `fixtures/invalid-expectations.toml` says so for each fixture.
- **`connection_sequence`.** The field lint requires a unit suffix on every integer. `_sequence` is
  added to the vocabulary, with the same kind of justification as `revision`: a gapless
  per-connection counter whose unit is the event itself.
- **Privacy class** is `boundary_metadata`, as for the other events of this kind.

## Risks / Trade-offs

- The JSON Schema accepts a `notes.file.changed.v1` that breaks a cross-field rule. A consumer that
  validates only with the schema must also run the Rust type, or repeat the rules.
- `docs/NOTES_AND_SITES.md` lists `ratatoskr-sites` as a later consumer. SITE-025 adds it to the
  registry rows; this change declares only `ratatoskr-knowledge`.

## Migration Plan

Additive. `ratatoskr-notes` and `ratatoskr-knowledge` pin the resulting commit; nothing is removed.
