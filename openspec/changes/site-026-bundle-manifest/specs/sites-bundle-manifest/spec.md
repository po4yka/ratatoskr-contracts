## Purpose

Defines the Rust type of `sites/bundle-manifest.v1` and the rules it enforces at parse time. The
cross-repository meaning of the manifest is in the workspace store, change
`add-site-gateway-hosting`, spec `site-bundles`.

## ADDED Requirements

### Requirement: A bundle manifest lists confined files with their digests

`BundleManifest` SHALL refuse, at the serde layer and at the JSON Schema layer, a file without a
SHA-256, a path with a `..` segment or a leading `/`, and a path under `_r/`. It SHALL accept a path
under `.ratatoskr/`.

#### Scenario: A path under the forwarding prefix

- **WHEN** the manifest lists `_r/v1/sites/x/content`
- **THEN** the published schema refuses it, verified by `crates/sites-contracts/tests/bundle_manifest.rs::a_path_under_the_forwarding_prefix_is_refused`

#### Scenario: The SDK pages

- **WHEN** the manifest lists `.ratatoskr/sign-in/index.html`
- **THEN** the published schema accepts it, verified by `crates/sites-contracts/tests/bundle_manifest.rs::a_manifest_with_the_reserved_sdk_path_is_valid`
