## Context

The behaviour is defined by the workspace store (change `add-notes-write-back`, spec
`notes-write-command`, design D1 to D4). The OBS-023 types in `crates/notes-contracts` are the model:
closed vocabularies, validated strings, unknown fields refused, cross-field rules in `validate()`.

## Decisions

- **Where the schemas are.** Rule R3 puts a command schema under `schemas/json-schema/commands/` and
  an event schema under `schemas/events/<event type>.schema.json`. The contract
  `notes/write-request.v1` is the row `notes.write_request`, so its schema is
  `schemas/json-schema/commands/write-request.v1.schema.json`. The contract `notes/conflict.v1` is
  two rows, one for each event type, because one row registers one event type (R9).
- **The envelope carries the owner.** The command envelope has the tenant `user:<id>` and the
  aggregate `operation:<operation_id>`. The conflict events have the aggregate
  `notes_conflict:<uuid>`.
- **Unknown fields are refused** (`unknown_policy = "reject"`). A write must never carry note text in
  a member that the schema does not declare. The payloads have no `extensions` map.
- **One operation.** `operation` is an externally tagged union: an object with exactly one member,
  whose name is the kind (`create`, `update`, `rename`, `move`, `delete`, `set_property`,
  `resolve_conflict`). Two members fail both layers.
- **The content member is `blob`.** The lint bans the property names `content` and `value`, so the
  uploaded content is `blob` (as in `notes.file.changed.v1`) and the frontmatter change is
  `property_change`: `{"set": <BlobRef>}` or `"remove"`.
- **A frontmatter value is a blob.** The workspace keeps the stricter rule that no byte of a note
  enters Edge's outbox or JetStream, so the value of `set_property` is uploaded first as one JSON
  document and named by a `BlobRef` with `media_type` `application/json`. The schema states the
  media type; the owner `ratatoskr-notes` is a rule of the Rust type, as for every blob of a write.
- **A resolution has two shapes.** `resolve_conflict` is `{conflict_id, choice}` for `keep_current`,
  `keep_writer` and `accept_merge`, and `{conflict_id, choice: "custom", blob}` for `custom`. The
  hand-written schema is a `oneOf` of these two objects, so the JSON Schema also refuses `custom`
  without a blob. The workspace spec gives `resolve_conflict` no `base_hash`: notes checks it against
  the revision that the conflict recorded.
- **Write paths.** `VaultWritePath` is `VaultPath` with one more rule: the first segment is not
  `.obsidian` in any ASCII case. A case-insensitive file system (macOS, Windows) maps `.OBSIDIAN` to
  the configuration folder. The rule is in the pattern, so the schema and the parser cannot drift;
  the regular-expression engine has no look-ahead, so the pattern spells the complement of the word.
- **Origin.** `platform` or `site:<site id>`, where a site id has 3 to 40 lower-case ASCII letters,
  digits and `-`, as the workspace spec `site-manifest` defines it.
- **Blobs.** Every `blob` of a write is owned by `ratatoskr-notes` and has a SHA-256 digest. The
  schema cannot compare a member with a constant inside a shared type, so this rule is serde only.
- **Conflict hashes.** `ConflictRaised` carries `base_hash` (absent for `create`), `current_hash`
  (absent when no live file holds the path) and `writer_hash` (absent for `rename`, `move` and
  `delete`, which carry no content). A `merge_proposal` (merged hash, overlapping hunks) is allowed
  only for `text/markdown` and `text/plain`, and only when all three hashes are present. These rules
  are serde only.
- **The operation id.** Both conflict events carry `operation_id`, the write operation that raised
  or resolved the conflict, so that Edge's projection `platform_notes_conflict_projection` can put
  the conflict on that operation.
- **Privacy class** is `user_content`: paths and frontmatter values are written by the owner.

## Risks / Trade-offs

- The JSON Schema accepts a conflict event that breaks a cross-field rule, and a write whose blob
  belongs to another service. A consumer that validates only with the schema must also run the Rust
  type, as for `notes.file.changed.v1`.
- The workspace spec says "a JSON value" for `set_property`. This change carries it as an uploaded
  JSON document instead, so a producer that sends the value inline is refused.

## Migration Plan

Additive. The OBS-024 repositories pin the resulting commit; nothing is removed.
