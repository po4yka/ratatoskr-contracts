# Tasks: add-social-source-removed-event

Test-first pairs throughout; the runner is `cargo nextest run --locked -p ratatoskr-social-contracts`.

## 1. Removed event payload

- [x] 1.1 Failing test `tests/events.rs::removed_payload_round_trips_through_envelope`: construct a `SocialSourceRemoved`, set it into an envelope, read it back through `payload_as`, assert equality and that the envelope event type is `social.source.removed.v1`. Expect failure: type does not exist.
- [x] 1.2 Add `RemovalReason` (closed, snake_case) to `vocabulary.rs`, `SocialSourceRemoved` to `events.rs` with `impl EventPayload` and extensions flatten, export from `lib.rs`.
- [x] 1.3 Failing test `tests/vocabularies.rs::unknown_removal_reason_is_refused`: `"cache_eviction"` fails to parse as `RemovalReason`. Implement with 1.2.

## 2. Optional author

- [x] 2.1 Failing test `tests/snapshot_roundtrip.rs::snapshot_without_author_parses_and_reemits_absent`: parse an authorless snapshot JSON, assert `author.is_none()` and byte-stable re-emission keeps author absent. Expect failure: `author` is required.
- [x] 2.2 Make `author` optional in `snapshot.rs` (public field + wire mirror + move), update `tests/common/mod.rs::snapshot_carrying_every_field` and any snapshot builders to wrap `Some(...)`.

## 3. Registration and generated artifacts

- [x] 3.1 Register `SocialSourceRemoved` in `tools/contractsc/src/registry.rs` (root types map and event payload map) and add the `social.source_removed` contract block to `contracts.toml`; run `cargo contracts generate` so the schema/TypeScript pair appears. No failing test precedes this: generated files and registry wiring are verified by `cargo contracts check` determinism.
- [x] 3.2 Failing test registration `tests/compat_fixtures.rs`: add the removed-family `direction_tests!` entry; expect failure until fixtures exist.
- [x] 3.3 Add fixtures: `fixtures/events/social.source.removed.v1/{valid,compat/*}` sets plus one invalid fixture (`fixtures/events/social.source.removed.v1/invalid/unknown-reason.json`) with its `invalid-expectations.toml` entry; add the author-absent compat fixture under `fixtures/social/social-source-snapshot/compat/old-consumer-new-producer/author-absent.json`.

## 4. Compatibility baseline

- [x] 4.1 Run `cargo contracts api-write`, review the moved `compat/api/ratatoskr-social-contracts.txt` diff line by line, and commit it. No failing test: frozen artifact refresh is proven by the compatibility workflow.

## 5. Documentation

- [x] 5.1 Update the social section of `README.md` and crate-level docs to name the third event and absent-author semantics. Documentation cannot start from a failing test.
