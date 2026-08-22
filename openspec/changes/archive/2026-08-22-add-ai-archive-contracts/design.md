# Design: add-ai-archive-contracts

## Context

The repository generates JSON Schema from canonical Rust types via `contractsc` (ADR-0001). A new contract family therefore means: a crate, registry entries, `contracts.toml` declarations, fixtures with expectations, and regenerated artifacts — all under a gate that lints field names, timestamp vocabulary, and fixture secrecy. `docs/ARCHITECTURE.md` S8 sketches the semantics; `AGENTS.md` adds the AI-archive rules (conversation graphs, unknown records preserved, conservative deletion state, product exports never equated with API surfaces). Producers are `ratatoskr-chatgpt` and `ratatoskr-claude`; the consumer is Knowledge.

One gate fact shapes the type layout: the field lint's L4 timestamp governance keys `[[contract.field]]` entries to the **first root type** of a contract (`TypeName#/properties/name`). A `format: date-time` property on any type that is not some contract's first root is therefore ungovernable and fails the gate. Every type carrying a timestamp must be a root type of its own contract.

## Goals / Non-Goals

Goals: one crate holding every AI-archive wire type; one shared grammar for both providers with explicit extension points; unknown provider records preserved verbatim; full gate compliance; fixtures that prove both compatibility directions.

Non-Goals: export parsers, BlobStore code, Compliance adapters, Knowledge consumers, deletion/retention workflows (only the observed state travels), embeddings or LLM summaries (AGENTS.md forbids them here), provider-specific payload mirrors.

## Decisions

### D1: No new identifier grammar; three new UUID identities

`AiArchiveId` (`kind = "ai_archive"`), `AiProjectId` (`ai_project`) and `AiConversationId` (`ai_conversation`) come from the existing `uuid_newtype!` macro, exactly like `SocialSourceId` (ADR-0007 clause 1). Provider message/conversation/project ids reuse `EntityLocalId` — opaque, case-sensitive, provider-minted. Cross-node pointers use the vocabulary that already exists: a conversation's project link is an `EntityRef` (`ai_project:<uuid>`, clause 2 — a pointer whose referent kind must be readable), while a message's parent link is a plain `EntityLocalId` because it names a sibling by provider id inside the same conversation payload, not a Ratatoskr record.

### D2: One grammar; extension points, not provider branches

`AiProvider` is an open validated token (the event-type segment grammar), like `Platform`. Provider differences live in: opaque external ids; the open `asset_kind` and `gap_kind` tokens; the unknown-part channel; and the `extensions` preservation map on every node. No type name, enum variant, or schema mentions a provider. Alternative considered — provider-tagged variants (`ChatGptMessage` / `ClaudeMessage`) — rejected: it is exactly the two-divergent-schemas outcome the scope forbids and it forces every consumer to grow a provider branch.

### D3: Timestamps only on root types; four JSON Schema roots

Because of L4 governance (Context), `AiArchiveSnapshot`, `AiProject`, `AiConversation` and `AiMessage` each become the first root type of their own JSON Schema contract. This mirrors the `core.blob_ref` precedent (a reusable published shape gets its own schema) and gives Knowledge a stable schema for exactly the unit it indexes (the conversation). `AiArchiveImport` becomes the payload type of the imported event, which both makes its `imported_at` governable and avoids a summary/head duplicate struct. Timestamp vocabulary: `imported_at` (authority `observed` — the producer's clock), `provider_created_at` / `provider_updated_at` (authority `provider_authored`, absent when the export supplies none — never fabricated, the social `published_at` rule).

### D4: The snapshot composes; events carry slices

`AiArchiveSnapshot` = `AiArchiveImport` (head) + projects + conversations. The imported event carries the head alone; the added/updated events carry `ai_archive_id` + one whole conversation. Rationale: a whole-archive event payload would be megabytes and redundant with per-conversation events; per-conversation payloads keep at-least-once redelivery idempotent and replay convergent (state-carried transfer, the social precedent). The snapshot stays a published contract because it is the canonical normalized tree — the shape a bulk load or a re-parse verification consumes. Drift between the snapshot's head and the event payload is structurally impossible: they are the same Rust type.

### D5: Unknown content parts via a hand-written enum

`AiContentPart` is an internally tagged enum (`part_kind`) with known variants — `text`, `markdown`, `image`, `asset`, `citation`, `tool_call`, `tool_result` — plus an unknown channel. `Serialize` and `Deserialize` are hand-written: parsing first tries the known discriminators (each variant denying unknown members), and anything else — including a known discriminator with an invalid body? No: a known discriminator with an invalid body is a *malformed known part* and fails loudly (spec: not quietly demoted); only an unrecognized discriminator (or a non-object) lands in `Unknown(serde_json::Value)` verbatim. `JsonSchema` is hand-written to match: `oneOf` of the known branches plus an always-valid unknown branch marked `x-ratatoskr-unknown-policy: "preserve"`. This follows the `Extensions` precedent for hand-written schema emission and satisfies AGENTS.md "archive imports must not discard unrecognized records".

### D6: Completeness is closed, counted, and cross-checked

`AiArchiveCompleteness` is a closed enum spelling ARCHITECTURE S8.3's six states verbatim; an unknown state stops processing (retention and indexing decisions hang off it). `AiCompletenessReport` carries `conversation_count`, `message_count`, `asset_count`, `gap_count` (all `_count`-suffixed per L2) and `gaps: Vec<AiGap>` with open `gap_kind` tokens. Two cross-field invariants, enforced in a hand-written `Deserialize` on the snapshot and on the import head: (a) every state other than `complete` requires at least one gap; (b) `conversation_count` and `gap_count` must equal the counts computable from the carried nodes — a producer cannot report counts its own payload disproves. `message_count` and `asset_count` are producer-asserted totals for whole-archive reports (the imported event's head has no tree to check them against), documented as such. Like the social S1 rule, a `complete` import may still carry warnings.

### D7: Parser stamps are two opaque tokens

`ParserName` (which parser, snake_case token) and `ParserVersion` (which build, bounded printable ASCII without whitespace: `1.4.2`, `2026.08.1`, git shas all fit) on every node. Alternatives: a single semver-typed field (rejects date-based and sha-based versions), or stamps only at import level (loses the mixed-build seam when only part of an archive is re-parsed). Consumers compare for staleness; nothing parses them.

### D8: Governance metadata choices

- `[services].known` gains `ratatoskr-chatgpt` and `ratatoskr-claude`; `[entity_kinds].known` gains `ai_archive`, `ai_project`, `ai_conversation`.
- Contract owner is `ratatoskr-chatgpt`: the metadata model allows one accountable owner, and joint governance across the two producer repos runs through the workspace changeset either way (the social precedent picked its origin producer the same way).
- Privacy class is `user_content` for all seven contracts (conversation bodies are user content); classification `internal`.
- Fixture URLs: the scanner bans `https://` under `fixtures/**`, so citation `url` coverage comes from the Rust round-trip drift-guard test, exactly as social's `permalink` did.
- `tools/contractsc/tests/fixtures.rs` `canonical()` gains one match arm per new root type — that test file is the canonical renderer registry, so a new root type without an arm fails F-4.

### D9: Compatibility fixtures without a fake history

Same shape as social D7: born at v1, there is no older major to freeze. `new-consumer-old-producer/` holds the minimal first-day shape (every optional member omitted); `old-consumer-new-producer/` holds payloads carrying unknown additive members — including an unknown content part, which is this family's defining preservation case.

## Risks / Trade-offs

- [Four JSON Schema roots instead of one] → Each is a genuinely published shape (the conversation is what Knowledge indexes); the alternative — ungovernable timestamps or none — is worse.
- [Unknown-part channel can preserve garbage] → It preserves exactly what arrived, which is the requirement; known-discriminator parts are strictly validated, so only genuinely novel records land there.
- [Hand-written serde/schema on `AiContentPart` can drift three ways] → The drift-guard round-trip test constructs a message carrying every part kind plus an unknown one and asserts byte stability and losslessness; the invalid fixture for a malformed known part pins the fail-loudly branch.
- [Whole-conversation event payloads are large for long conversations] → State-carried transfer is the repository's replay-safety stance; chunking is a producer-side concern the contract does not preclude (a producer may emit one added event per conversation, which is the designed granularity).
- [`message_count`/`asset_count` unverifiable on the head-only event] → Documented as producer-asserted; the snapshot contract verifies what its tree can check.

## Migration Plan

Purely additive: new crate, new schemas, new fixtures; no existing type or artifact changes except registry/metadata/doc status. Rollback is reverting the commit; nothing consumes these contracts yet. No database exists behind them (development status), so there is no data migration.

## Open Questions

None.
