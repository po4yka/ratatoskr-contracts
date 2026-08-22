# Tasks: add-ai-archive-contracts

## 1. Identifiers

- [x] 1.1 Failing test: add `AiArchiveId`, `AiProjectId`, `AiConversationId` cases to `crates/identifiers/tests/typed_ids.rs` — each parses a canonical UUID, rejects an uppercase spelling with `PatternMismatch`, carries its own kind in `as_entity_ref()`, and round-trips through `TryFrom<&EntityRef>`. Run `cargo test -p ratatoskr-identifiers --test typed_ids` and confirm it fails to compile because the types do not exist.
- [x] 1.2 Implement the three newtypes with the existing `uuid_newtype!` macro in `crates/identifiers/src/uuid_ids.rs` and export them from `lib.rs`; the tests from 1.1 pass.

## 2. Crate skeleton

- [x] 2.1 Create `crates/ai-archive-contracts` (Cargo.toml mirroring `ratatoskr-social-contracts`, empty module files, lib docs) and add it to workspace members plus `[workspace.dependencies]`. This is configuration, so it cannot start from a failing test; verification is `cargo check -p ratatoskr-ai-archive-contracts --locked` succeeding on an empty lib.

## 3. Tokens, stamps, values

- [x] 3.1 Failing test: `crates/ai-archive-contracts/tests/tokens.rs` — `AiProvider::parse` accepts `chatgpt` and `claude`, preserves an unknown-but-grammatical token verbatim via `as_str`, rejects uppercase and empty input; same shape for `ParserName`; `ParserVersion` accepts `1.4.2` and a git-sha-like token and rejects whitespace-bearing input; `AiTitle` rejects control characters; `AiText` preserves internal newlines through a round trip while rejecting other C0 controls.
- [x] 3.2 Implement `AiProvider`, `ParserName`, `ParserVersion`, `AiTitle`, `AiText` via `wire_string_newtype!`; tests from 3.1 pass.

## 4. Content parts

- [x] 4.1 Failing test: `crates/ai-archive-contracts/tests/content_parts.rs` — every known part kind parses from its tagged JSON shape; an image part carries a `BlobRef`; an asset part carries an asset-kind token plus `BlobRef` plus optional file name; citation, tool call and tool result round-trip with all optional members set; a part whose discriminator this build does not know parses into the unknown channel and re-emits byte-identically; a part declaring the text discriminator without a text member is rejected naming the type (not demoted to unknown); a non-object part is preserved as unknown.
- [x] 4.2 Failing test: `crates/ai-archive-contracts/tests/content_parts_schema.rs` — the hand-written `JsonSchema` for `AiContentPart` compiles as draft 2020-12, accepts each known tagged shape and an arbitrary unknown object, and marks the unknown branch with the preserve policy.
- [x] 4.3 Implement `AiContentPart` (internally tagged enum, hand-written `Serialize`/`Deserialize`/`JsonSchema`), `AiAsset`, `AiCitation`, `AiToolCall`, `AiToolResult` per design D5; tests from 4.1–4.2 pass.

## 5. Graph nodes

- [x] 5.1 Failing test: `crates/ai-archive-contracts/tests/graph_nodes.rs` — a project/conversation/message triple carrying every field (provider external ids, project ref, parent refs, model name, parser stamps, provider-authored timestamps where present) round-trips losslessly with optional members absent serializing as absent; a message with two answers sharing one parent parses; provider timestamps are rejected when non-canonical.
- [x] 5.2 Implement `AiProject`, `AiConversation`, `AiMessage` with derived serde + schemars; tests from 5.1 pass.

## 6. Import head, report, snapshot

- [x] 6.1 Failing test: `crates/ai-archive-contracts/tests/completeness.rs` — all six completeness states parse by their wire tokens; an unknown state is rejected with `unknown variant`; a head or snapshot declaring any state other than `complete` with no gaps is rejected naming the invariant; `conversation_count` or `gap_count` disagreeing with the carried nodes is rejected with a count-mismatch error; a complete import may carry warnings and zero gaps and parses.
- [x] 6.2 Failing test: `crates/ai-archive-contracts/tests/snapshot_roundtrip.rs` — the drift guard constructs an `AiArchiveSnapshot` carrying every field of every node (unknown part, citations with URLs, assets), asserts byte-stable canonical round trip, losslessness via `dropped_field_pointers`, and that every documented member name appears in the rendering; the import head alone round-trips identically.
- [x] 6.3 Implement `AiArchiveImport`, `AiCompletenessReport`, `AiGap`, `AiArchiveCompleteness` (closed enum), and `AiArchiveSnapshot` with hand-written `Deserialize` calling `validate()` on both head and snapshot; tests from 6.1–6.2 pass.

## 7. Events

- [x] 7.1 Failing test: `crates/ai-archive-contracts/tests/events.rs` — `EVENT_TYPE` constants are `ai_archive.archive.imported.v1`, `ai_archive.conversation.added.v1`, `ai_archive.conversation.updated.v1`, each parsing with major 1 and past-tense action; each real payload accepts `set_payload` into a minimal envelope and returns typed and unchanged via `payload_as`; requesting an archive payload from a `platform.operation.progressed.v1` envelope fails with `PayloadType`.
- [x] 7.2 Implement `EventPayload` for `AiArchiveImport` and for `AiConversationAdded` / `AiConversationUpdated` (`ai_archive_id` + whole conversation); tests from 7.1 pass.

## 8. Registry, metadata, artifacts

- [x] 8.1 Register the seven root types in `tools/contractsc/src/registry.rs` (sorted by rust_path) and add their event-type declarations; add the seven match arms to `canonical()` in `tools/contractsc/tests/fixtures.rs`. Configuration; verification is `cargo check -p ratatoskr-contractsc --locked` and `cargo test -p ratatoskr-contractsc --test fixtures`.
- [x] 8.2 Declare four JSON Schema contracts and three event contracts in `contracts.toml`: `ai_archive.archive_snapshot`, `ai_archive.project`, `ai_archive.conversation`, `ai_archive.message` under family `ai_archive`, and the three events under family `events`; add `ratatoskr-chatgpt` / `ratatoskr-claude` to `[services].known`, `ai_archive` / `ai_project` / `ai_conversation` to `[entity_kinds].known`, and `imported_at` / `provider_created_at` / `provider_updated_at` to `[lint].timestamp_property_names` with `[[contract.field]]` governance on each root that declares them. Configuration; verification is R2/R9 passing inside `cargo contracts check` once artifacts exist.
- [x] 8.3 Run `cargo contracts generate` and commit the seven new generated schemas. Generated files cannot start from a failing test; verification is the drift step of `cargo contracts check` reporting them current.

## 9. Fixtures

- [x] 9.1 Valid fixtures under `fixtures/ai_archive/{project,conversation,message,archive-snapshot}/valid/` covering both providers across shapes: ChatGPT-style conversation with text/tool parts, Claude-style conversation with markdown/artifact/citation parts, projectless conversation, branched messages, mixed parser stamps, complete-with-warning, structurally-partial-with-gaps, conversations-complete, failed-validation. Verification: `cargo contracts check` fixture step accepts them all.
- [x] 9.2 Payload fixtures under `fixtures/events/ai_archive.archive.imported.v1/valid/`, `fixtures/events/ai_archive.conversation.added.v1/valid/` and `fixtures/events/ai_archive.conversation.updated.v1/valid/`. Verification: `cargo contracts check` accepts.
- [x] 9.3 Invalid fixtures with `fixtures/invalid-expectations.toml` entries: unknown completeness state, incomplete-without-gap (serde-only, cross-field), count mismatch (serde-only, cross-field), non-canonical `imported_at` (serde-only), malformed known content part, unknown provider token that violates the token grammar, missing owner, message parent pointing outside the conversation grammar, and one per event family (missing required member). Each entry declares `rejected_by` honestly per ADR-0001. Verification: `cargo contracts check` reports every layer agreement green.
- [x] 9.4 Compatibility fixtures per design D9 for the snapshot, conversation, message, imported and added families: `old-consumer-new-producer/` payloads carrying unknown additive members including an unknown content part, `new-consumer-old-producer/` minimal first-day shapes. Verification: `cargo contracts check` accepts them all.
- [x] 9.5 Failing test: `crates/ai-archive-contracts/tests/compat_fixtures.rs` asserts both directions exist and re-emit losslessly for all seven fixture families. Written after 9.4's files exist on disk it should pass immediately; confirm it fails if any directory is emptied (delete-and-run once, then restore).

## 10. Documentation

- [x] 10.1 Rewrite `docs/ARCHITECTURE.md` S8 around the implemented crate (types, closed/open vocabulary split, graph semantics, part preservation, counts, events, URL-scanner fixture note) and correct its `ContentPart` sketch to the implemented grammar. Documentation cannot start from a failing test.
- [x] 10.2 Update status references: `docs/IMPLEMENTATION_PLAN.md` milestone 7 marked implemented, `README.md` status/tree lines, `DEVELOPMENT.md` present/absent lists and crate counts.

## 11. Gate

- [x] 11.1 Run the full gate from `DEVELOPMENT.md` in order (fetch, deny, fmt, clippy, file-length, `cargo contracts check`, test workspace) until every step is green with no diff left behind.
