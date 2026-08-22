# Tasks: add-social-source-contracts

## 1. Identifiers

- [x] 1.1 Failing test: add `SocialSourceId` cases to `crates/identifiers/tests/typed_ids.rs` — parses a canonical UUID, rejects an uppercase spelling with `PatternMismatch`, `as_entity_ref()` carries kind `social_source`, and `TryFrom<&EntityRef>` round-trips. Run `cargo test -p ratatoskr-identifiers --test typed_ids` and confirm it fails to compile because the type does not exist.
- [x] 1.2 Implement `SocialSourceId` with the existing `uuid_newtype!` macro (`kind = "social_source"`) in `crates/identifiers/src/uuid_ids.rs` and export it from `lib.rs`; the tests from 1.1 pass.

## 2. Crate skeleton

- [x] 2.1 Create `crates/social-contracts` (Cargo.toml mirroring `ratatoskr-operation-contracts`, empty module files, lib docs) and add it to the workspace members plus `[workspace.dependencies]`. This is configuration, so it cannot start from a failing test; verification is `cargo check -p ratatoskr-social-contracts --locked` succeeding on an empty lib.

## 3. Open token types

- [x] 3.1 Failing test: `crates/social-contracts/tests/platform.rs` — `Platform::parse` accepts `x`, `instagram`, `threads`, preserves an unknown-but-grammatical token verbatim via `as_str`, rejects uppercase and empty input; same shape for `SocialMediaKind` (`image`, `video`, unknown preserved).
- [x] 3.2 Implement `Platform` and `SocialMediaKind` via `wire_string_newtype!` (`^[a-z][a-z0-9_]{0,31}$`); tests from 3.1 pass.

## 4. Closed vocabularies

- [x] 4.1 Failing test: `crates/social-contracts/tests/vocabularies.rs` — serde accepts every spelled variant of `AcquisitionMethod`, `SavedAuthority`, `CaptureCompleteness`, `UpstreamAvailability` and rejects `"carrier_pigeon"` / `"shadowbanned"` / `"half"` / `"platform_says_so"` with `unknown variant` errors. The generated-schema half of the closed-vocabulary rule is asserted later by the `json_schema`-declared invalid fixtures of task 9.3.
- [x] 4.2 Implement the four closed enums (derive serde + schemars, `#[non_exhaustive]`, rustdoc stating the closed-parse rule); tests from 4.1 pass.

## 5. Value types

- [x] 5.1 Failing test: `crates/social-contracts/tests/value_types.rs` — `ProviderHandle` accepts `example_user` and rejects `@example_user` and control characters; `DisplayName` rejects control characters; `PostText` preserves internal newlines through a round trip while rejecting other C0 controls; `PostPermalink` requires the HTTPS form and rejects whitespace; `SocialAuthor`, `SocialMediaItem`, `SocialRelation`, `SocialFolderMembership` round-trip with all fields set and with optional fields absent serializing as absent (never null).
- [x] 5.2 Implement the five newtypes and four structs per design D1/D5; tests from 5.1 pass.

## 6. Snapshot

- [x] 6.1 Failing test: `crates/social-contracts/tests/snapshot_invariants.rs` — a snapshot with `completeness: partial` and no warnings is rejected naming the invariant; the same payload with one warning parses; a checkpoint containing a newline is rejected; `deleted_upstream` with complete capture and full text/media parses.
- [x] 6.2 Failing test: `crates/social-contracts/tests/snapshot_roundtrip.rs` — the O-2-style drift guard constructs a snapshot carrying **every** field (permalink included), asserts byte-stable canonical round trip, losslessness via `dropped_field_pointers`, and that every documented member name appears in the rendering.
- [x] 6.3 Implement `SocialSourceSnapshot` with derived `Serialize`, private wire-mirror `Deserialize` calling `validate()`, and the partial-requires-warnings invariant; update the now-stale "only hand-written `Deserialize`" claim in `crates/operation-contracts/src/snapshot.rs`; tests from 6.1–6.2 pass.

## 7. Events

- [x] 7.1 Failing test: `crates/social-contracts/tests/events.rs` — `EVENT_TYPE` constants are `social.source.captured.v1` / `social.source.updated.v1`, parse, carry major 1; a real minimal envelope accepts `set_payload` for each and returns the typed payload via `payload_as` unchanged; requesting a social payload from a `platform.operation.progressed.v1` envelope fails with `PayloadType`.
- [x] 7.2 Implement `SocialSourceCaptured` and `SocialSourceUpdated` implementing `EventPayload`, carrying the whole snapshot; tests from 7.1 pass.

## 8. Registry, metadata, artifacts

- [x] 8.1 Register the three root types and two event payloads in `tools/contractsc/src/registry.rs`. Configuration; verification is `cargo check -p ratatoskr-contractsc --locked`.
- [x] 8.2 Declare the three contracts in `contracts.toml`: `social.social_source_snapshot` plus the two event contracts, with `[[contract.field]]` governance for `published_at` (authority `provider_authored`) and `captured_at` (authority `observed`), `[lint].timestamp_property_names` extended with both names, and `ratatoskr-instagram` / `ratatoskr-threads` added to `[services].known`. Configuration; verification is R2/R9 passing inside `cargo contracts check` once artifacts exist.
- [x] 8.3 Run `cargo contracts generate` and commit the three new generated schemas. Generated files cannot start from a failing test; verification is the drift step of `cargo contracts check` reporting them current.

## 9. Fixtures

- [x] 9.1 Valid fixtures under `fixtures/social/social-source-snapshot/valid/` covering all five acquisition methods across X/Instagram/Threads shapes (bookmark-with-folders authoritative, explicit captures, public resolution, data export, legacy import, deleted-upstream-complete, partial-with-warning). Verification: `cargo contracts check` fixture step accepts them all.
- [x] 9.2 Payload fixtures under `fixtures/events/social.source.captured.v1/valid/` and `fixtures/events/social.source.updated.v1/valid/`. Verification: `cargo contracts check` accepts.
- [x] 9.3 Invalid fixtures with `fixtures/invalid-expectations.toml` entries: unknown acquisition method, unknown saved authority, partial-without-warning (serde-only, cross-field), non-canonical `captured_at` (serde-only), missing `social_source_id`, unknown upstream availability, missing media blob, and one per event family (missing `source`). Each entry declares `rejected_by` honestly per ADR-0001. Verification: `cargo contracts check` reports every layer agreement green.
- [x] 9.4 Compatibility fixtures per design D7: `old-consumer-new-producer/future-optional-member.json` and `new-consumer-old-producer/minimal-first-shape.json` for the snapshot family and both event families. Verification: `cargo contracts check` accepts all six.
- [x] 9.5 Failing test: `crates/social-contracts/tests/compat_fixtures.rs` asserts both directions exist and re-emit losslessly for all three families. Written after 9.4's files exist on disk it should pass immediately; confirm it fails if any directory is emptied (delete-and-run once, then restore).

## 10. Documentation

- [x] 10.1 Rewrite `docs/ARCHITECTURE.md` S7 around the implemented crate (types, closed/open vocabulary split, snapshot semantics, events, URL-scanner fixture note) and correct its enum sketch to include `public_resolution`. Documentation cannot start from a failing test.
- [x] 10.2 Update status references: `docs/IMPLEMENTATION_PLAN.md` milestone 6 marked implemented, `README.md` status/tree lines, `DEVELOPMENT.md` present/absent lists and five-crate counts.

## 11. Gate

- [x] 11.1 Run the full gate from `DEVELOPMENT.md` in order (fetch, deny, fmt, clippy, file-length, `cargo contracts check`, test workspace) until every step is green with no diff left behind.
