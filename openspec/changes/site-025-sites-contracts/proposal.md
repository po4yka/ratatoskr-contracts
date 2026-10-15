## Why

Changeset SITE-025 adds sites that show published notes of a connected Obsidian vault. Every other
repository of the change builds against the wire shapes of this repository, so they come first. The
cross-repository behaviour is defined in the workspace store, change `add-site-publications`: the
specs `site-manifest`, `site-publications`, `site-release-privacy` and `site-content-delivery`, and
design decisions D2, D4, D6 and D9. This change cites them and does not restate them.

## What Changes

- Add the crate `ratatoskr-sites-contracts` (`crates/sites-contracts`).
- Add the site manifest `SiteManifest` (`sites/site-manifest.v1`, the file `ratatoskr-site.toml`)
  with the closed capability set `SiteCapability`.
- Add the release document `Release` (`sites/release.v1`) and the page `Page` (`sites/page.v1`),
  whose body is an mdast-compatible tree with the node types `callout`, `wikiLink` and `embed`, and
  the same body as sanitised HTML.
- Add the command payload `ReleaseBuildRequested` (`sites.release.build_requested.v1`) and the event
  payloads `ReleasePublished` (`sites.release.published.v1`) and `SiteApprovalChanged`
  (`platform.site_approval.changed.v1`).
- Register the three messages in `contracts.toml`, add `ratatoskr-sites` to the known services and
  to the consumers of `notes.file.changed.v1` and `notes.vault.disconnected.v1`.
- Add fixtures, regenerate the JSON Schema and TypeScript, and update the pinned counts.

The change is additive. No existing message, field, schema or API line changes.

## Capabilities

### New Capabilities

- `sites-payloads`: the Rust types of the site manifest, the release, the page and the three site
  messages, and the rules they enforce at parse time.

### Modified Capabilities

<!-- None. -->

## Impact

Touches `Cargo.toml`, `crates/sites-contracts`, `contracts.toml`, `fixtures/`, `schemas/`,
`generated/typescript/`, `tools/contractsc` (registry and tests) and the crate lists in the
documentation. `ratatoskr-notes`, `ratatoskr-sites`, `ratatoskr-platform`, `ratatoskr-site-sdk` and
`ratatoskr-web` pin the resulting commit in their own SITE-025 tasks. Rollback: revert the commits
before any repository pins them.
