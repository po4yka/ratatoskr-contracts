## Purpose

Defines the Rust types of `sites/site-event.v1` and `sites.draft.changed.v1`. The cross-repository
meaning of these shapes is in the workspace store, change `add-site-owner-editing-and-live`.

## ADDED Requirements

### Requirement: A public event carries no page list and no source reference

`SiteEvent` SHALL refuse, at the serde layer and at the JSON Schema layer, a `release.published`
payload with any member other than `release_id` and `published_at`.

#### Scenario: A page list

- **WHEN** a `release.published` event carries a list of slugs
- **THEN** the published schema refuses it, verified by `crates/sites-contracts/tests/site_event.rs::release_published_refuses_page_list`

### Requirement: The draft change message is registered

The registry SHALL name `ratatoskr-sites` as the producer and `ratatoskr-platform` as the consumer of
`sites.draft.changed.v1`, and its schema SHALL have no `content` or `path` property.

#### Scenario: The draft change message

- **WHEN** the test reads `contracts.toml` and the generated schema
- **THEN** the registration holds, verified by `tools/contractsc/tests/live_message_types.rs::sites_draft_changed_is_registered`
