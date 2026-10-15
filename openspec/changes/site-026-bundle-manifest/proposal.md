## Why

Changeset SITE-026 hosts each site's static bundle on the host. A site repository publishes a bundle
with a manifest, `ratatoskr-bundle.json`, and the sites service, the gateway and the deploy command
of the site SDK all read it. The cross-repository behaviour is defined in the workspace store,
change `add-site-gateway-hosting`: the spec `site-bundles` and design decision D4. This change cites
them and does not restate them.

## What Changes

- Add to `ratatoskr-sites-contracts` the bundle manifest `BundleManifest`
  (`sites/bundle-manifest.v1`): the site id, the repository, the commit, the entry document, the
  entry fallback and every file with its path, size, SHA-256 and `hashed` flag.
- Register it in `contracts.toml`, add fixtures, regenerate the JSON Schema and TypeScript, and
  update the pinned counts.

The change is additive. No existing message, field, schema or API line changes.

## Capabilities

### New Capabilities

- `sites-bundle-manifest`: the Rust type of the bundle manifest and the rules it enforces at parse
  time.

### Modified Capabilities

<!-- None. -->

## Impact

Touches `crates/sites-contracts`, `contracts.toml`, `fixtures/`, `schemas/json-schema/sites/`,
`generated/typescript/` and `tools/contractsc`. `ratatoskr-sites` and `ratatoskr-site-sdk` pin the
resulting commit in their own SITE-026 tasks. Rollback: revert the commits before any repository
pins them.
