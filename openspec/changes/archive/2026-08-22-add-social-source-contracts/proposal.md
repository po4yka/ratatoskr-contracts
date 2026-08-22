# Add social-source contracts

## Why

Milestone 6 of `docs/IMPLEMENTATION_PLAN.md` is the last contract family blocking the social vertical: `ratatoskr-x`, `ratatoskr-instagram` and `ratatoskr-threads` have no shared way to publish normalized post/author/media records, and Knowledge cannot consume captures until they do. `docs/ARCHITECTURE.md` S7 sketches the required semantics (acquisition method, saved-state authority, no collapsed `is_saved` flag) but no canonical Rust type, schema, or fixture exists.

## What Changes

- New workspace crate `crates/social-contracts` (`ratatoskr-social-contracts`) defining, as canonical Rust types:
  - `Platform` and `SocialMediaKind` as open validated strings; `AcquisitionMethod`, `SavedAuthority`, `CaptureCompleteness` and `UpstreamAvailability` as closed enums that reject unknown values at parse;
  - `SocialAuthor` (inline provider identity), `SocialMediaItem` (media metadata with a `BlobRef`, never bytes), `SocialRelation` (quote/reply/repost), `SocialFolderMembership` (native provider folders);
  - `SocialSourceSnapshot`: the normalized record plus capture semantics — completeness (complete vs partial), a provider sync checkpoint reference, warnings, and the raw-blob/content-digest pair;
  - event payloads `SocialSourceCaptured` (`social.source.captured.v1`) and `SocialSourceUpdated` (`social.source.updated.v1`) implementing `EventPayload`.
- `crates/identifiers` gains `SocialSourceId`, a typed UUID identity produced by the existing `uuid_newtype!` macro (ADR-0007 clause 1); no new identifier grammar.
- `tools/contractsc/src/registry.rs` registers the three root types and two event payloads; `contracts.toml` declares the three contracts, adds `ratatoskr-instagram` and `ratatoskr-threads` to `[services].known`, and extends `[lint].timestamp_property_names` with `published_at` and `captured_at`.
- Generated JSON Schema under `schemas/json-schema/social/` and `schemas/events/`; fixtures under `fixtures/social/` and `fixtures/events/social.source.{captured,updated}.v1/`; entries in `fixtures/invalid-expectations.toml`.
- Docs: `docs/ARCHITECTURE.md` S7 describes the implemented crate; `README.md`, `DEVELOPMENT.md` and `docs/IMPLEMENTATION_PLAN.md` move milestone 6 to implemented.

No breaking changes: nothing consumed today is altered.

## Capabilities

### New Capabilities

- `social-source-contracts`: the wire behaviour of the social-source snapshot and its captured/updated events — identity and authority semantics, media-by-reference, folder membership, snapshot completeness and checkpoints, envelope compatibility.

### Modified Capabilities

- None. `openspec/specs/` is empty by policy; cross-repository facts cited here live in the `ratatoskr-workspace` store (`blob-references` covers `BlobRef` resolution, which the media and raw-blob fields rely on).

## Impact

- Producers: `ratatoskr-x`, `ratatoskr-instagram`, `ratatoskr-threads` (both events). Consumers: `ratatoskr-knowledge` (indexing/analysis); `ratatoskr-platform` consumes nothing new but remains the operation envelope carrier.
- Code: new crate; one newtype in `ratatoskr-identifiers`; registry + metadata + lint-vocabulary entries in `tools/contractsc`/`contracts.toml`; regenerated artifacts; doc status updates.
- Out of scope, unchanged: provider clients, OAuth, HTTP, Knowledge-side consumption code.
