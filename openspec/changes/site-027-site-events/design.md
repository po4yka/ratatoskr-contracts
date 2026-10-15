## Context

The behaviour is defined by the workspace store (change `add-site-owner-editing-and-live`, specs
`site-live-events` and `site-owner-editing`, design D5). The SITE-025 and SITE-026 types in
`crates/sites-contracts` are the model.

## Decisions

- **The schemas are generated.** ADR-0001 makes the Rust type the only source of a schema, so the
  workspace tasks that say "write the schema" add the type and generate the schema from it.
- **Where the files are.** `sites/site-event.v1` is a document of the `sites` family, at
  `schemas/json-schema/sites/site-event.v1.schema.json`. `sites.draft.changed.v1` is a registered
  event, so rule R3 puts its schema at `schemas/events/sites.draft.changed.v1.schema.json`.
- **The event shape.** `{"type": <event type>, "payload": {...}}`, adjacently tagged; `resync` has
  no payload. Each payload refuses unknown members. `release.published` holds only `release_id` and
  `published_at`.
- **A source reference** is opaque: 1 to 128 ASCII letters, digits, `_` and `-`, so it fits a path
  segment of `drafts/sources/{source_ref}`. The sites service mints it.
- **A revision** is the lowercase hex SHA-256 of the file bytes, as `revision` in
  `notes.file.changed.v1`.
