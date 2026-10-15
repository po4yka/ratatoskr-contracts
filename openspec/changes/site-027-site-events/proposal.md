## Why

Changeset SITE-027 lets the owner edit a site's notes from the site and streams live events to the
site. The site SDK reads the stream, and Edge and the sites service exchange a new bus message. The
cross-repository behaviour is defined in the workspace store, change
`add-site-owner-editing-and-live`: the specs `site-live-events` and `site-owner-editing`, and design
decision D5. This change cites them and does not restate them.

## What Changes

- Add to `ratatoskr-sites-contracts` the site event `SiteEvent` (`sites/site-event.v1`): the data of
  one event of `GET /v1/sites/{site}/events`, with a closed `type` and a closed payload per type.
- Add the event payload `DraftChanged` (`sites.draft.changed.v1`) and register it in
  `contracts.toml`: producer `ratatoskr-sites`, consumer `ratatoskr-platform`.
- Add `SourceRef`, the opaque reference of a note in a site's publications.
- Add fixtures, regenerate the JSON Schema and TypeScript, and update the pinned counts.

The change is additive. No existing message, field, schema or API line changes.

## Capabilities

### New Capabilities

- `sites-site-events`: the Rust types of the site event and the draft change message, and the rules
  they enforce at parse time.

### Modified Capabilities

<!-- None. -->

## Impact

Touches `crates/sites-contracts`, `contracts.toml`, `fixtures/`, `schemas/`, `generated/typescript/`
and `tools/contractsc`. `ratatoskr-platform`, `ratatoskr-sites` and `ratatoskr-site-sdk` pin the
resulting commit in their own SITE-027 tasks. Rollback: revert the commits before any repository
pins them.
