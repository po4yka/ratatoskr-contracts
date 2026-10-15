## Purpose

Defines the Rust types of `sites/site-manifest.v1`, `sites/release.v1`, `sites/page.v1`,
`sites.release.build_requested.v1`, `sites.release.published.v1` and
`platform.site_approval.changed.v1`, and the rules they enforce at parse time. The cross-repository
meaning of these shapes is in the workspace store, change `add-site-publications`.

## ADDED Requirements

### Requirement: A site manifest has one closed shape

`SiteManifest` SHALL refuse, at the serde layer and at the JSON Schema layer, an unknown key, an
unknown capability name, a second hostname and a site id outside 3 to 40 lower-case ASCII letters,
digits and `-`. The JSON Schema error SHALL point at the key it refuses.

#### Scenario: The manifest fixtures match their names

- **WHEN** the test reads every `fixtures/sites/site-manifest/*.toml`
- **THEN** each `valid-*` fixture passes both layers and each `invalid-*` fixture fails both layers
  on the key its name states, verified by `tools/contractsc/tests/sites_fixtures.rs::site_manifest_fixtures_match_their_names`

### Requirement: A release and its pages round-trip

`Release` and `Page` SHALL deserialise every valid fixture and serialise it back to the same
canonical JSON.

#### Scenario: The release and page fixtures round-trip

- **WHEN** the test reads the release and page fixtures under `fixtures/sites/`
- **THEN** each one deserialises and serialises back to the same canonical bytes, verified by `tools/contractsc/tests/sites_fixtures.rs::release_and_page_fixtures_round_trip`

### Requirement: The site messages have their producers and consumers

The registry SHALL name `ratatoskr-platform` as the producer and `ratatoskr-sites` as the consumer of
`sites.release.build_requested.v1` and `platform.site_approval.changed.v1`, `ratatoskr-sites` as the
producer of `sites.release.published.v1`, and `ratatoskr-sites` as a consumer of
`notes.file.changed.v1` and `notes.vault.disconnected.v1`. The schema of
`sites.release.published.v1` SHALL have no title, slug or path property.

#### Scenario: The site messages are registered

- **WHEN** the test reads `contracts.toml` and the generated schemas
- **THEN** each message has the stated producers and consumers, verified by `tools/contractsc/tests/live_message_types.rs::sites_message_types_are_registered`
