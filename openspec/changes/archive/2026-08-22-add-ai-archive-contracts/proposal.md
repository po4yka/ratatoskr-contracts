# Add AI archive contracts

## Why

Milestone 7 of `docs/IMPLEMENTATION_PLAN.md` is the last contract family named by the repository's purpose that has no canonical types: `ratatoskr-chatgpt` and `ratatoskr-claude` will import official account exports as immutable evidence and publish normalized conversation graphs, and `ratatoskr-knowledge` cannot index those graphs until a shared wire form exists. `docs/ARCHITECTURE.md` S8 sketches the required semantics (export evidence, conversation graphs with parent links, unknown content parts preserved, evidence-based completeness) but no canonical Rust type, schema, or fixture exists.

## What Changes

- New workspace crate `crates/ai-archive-contracts` (`ratatoskr-ai-archive-contracts`) defining, as canonical Rust types:
  - `AiProvider` as an open validated token (`chatgpt`, `claude`, ...); `ParserName` / `ParserVersion` stamps carried by every graph node so consumers can tell which parser build normalized a record;
  - `AiProject`, `AiConversation`, `AiMessage` graph nodes: conversations reference their optional project, messages carry an optional parent message id (branches and regenerated answers survive), and every node carries provider external ids where the export supplies them;
  - `AiContentPart`: one shared part grammar (text, markdown, image, asset, citation, tool call, tool result) plus an unknown-part channel that preserves unrecognized provider records verbatim through normalization and re-export;
  - `AiAsset` (asset references by `BlobRef`, never bytes) and `AiCitation`;
  - `AiArchiveImport`: archive identity, owner, the immutable raw-export `BlobRef`, import time, parser stamp, completeness report and warnings;
  - `AiCompletenessReport`: closed completeness vocabulary per `docs/ARCHITECTURE.md` S8.3, verifiable counts, structured gaps;
  - `AiArchiveSnapshot`: the whole normalized tree in one contract;
  - event payloads on the existing envelope: `ai_archive.archive.imported.v1`, `ai_archive.conversation.added.v1`, `ai_archive.conversation.updated.v1`.
- `crates/identifiers` gains `AiArchiveId`, `AiProjectId` and `AiConversationId` via the existing `uuid_newtype!` macro (ADR-0007 clause 1); no new identifier grammar.
- `tools/contractsc/src/registry.rs` registers seven root types; `contracts.toml` declares four JSON Schema contracts and three event contracts, adds `ratatoskr-chatgpt` and `ratatoskr-claude` to `[services].known`, adds three entity kinds to `[entity_kinds].known`, and extends `[lint].timestamp_property_names` with `imported_at`, `provider_created_at` and `provider_updated_at`.
- Generated JSON Schema under `schemas/json-schema/ai_archive/` and `schemas/events/`; fixtures under `fixtures/ai_archive/` and `fixtures/events/ai_archive.*.v1/`; entries in `fixtures/invalid-expectations.toml`.
- Docs: `docs/ARCHITECTURE.md` S8 rewritten around the implemented crate; `README.md`, `DEVELOPMENT.md` and `docs/IMPLEMENTATION_PLAN.md` move milestone 7 to implemented.

No breaking changes: nothing consumed today is altered.

## Capabilities

### New Capabilities

- `ai-archive-contracts`: the wire behaviour of the normalized AI-archive graph and its imported/conversation-added/conversation-updated events — evidence identity and authority semantics, graph structure, content-part preservation, parser stamps, completeness reporting, envelope compatibility.

### Modified Capabilities

- None. `openspec/specs/` holds only `social-source-contracts`; cross-repository facts cited here live in the `ratatoskr-workspace` store (`blob-references` covers `BlobRef` resolution, which the raw-export evidence and every asset reference rely on).

## Impact

- Producers: `ratatoskr-chatgpt` and `ratatoskr-claude` (all three events). Consumers: `ratatoskr-knowledge` (indexing/analysis).
- Code: new crate; three newtypes in `ratatoskr-identifiers`; registry + metadata + lint-vocabulary entries in `tools/contractsc`/`contracts.toml`; one new match arm per new root type in the canonical renderer of `tools/contractsc/tests/fixtures.rs`; regenerated artifacts; doc status updates.
- Out of scope, unchanged: export parsers, BlobStore code, Compliance adapters, Knowledge-side consumption code.
