# Ratatoskr Contracts

`ratatoskr-contracts` is the wire-contract repository for Ratatoskr. It defines the versioned structures exchanged between independently deployed services and the public API artifacts consumed by Ratatoskr clients.

> **Status:** milestones 1–5 implemented. Shared identifiers, the event envelope, error contracts, operation contracts, Document IR and the deterministic generator (`cargo contracts`) exist, together with generated JSON Schema and the fixture suite. Social-source and AI-archive contracts, OpenAPI, generated TypeScript, a frozen compatibility baseline and package publication do **not** exist yet. Repository CI runs the documented gate.

> [!IMPORTANT]
> **Ratatoskr is in development.** No database holds data that has to survive a schema change.
> While this status holds, these two rules replace what the documents below plan:
>
> - the API and the database keep their first version. There is no `v2` and no later major
>   version.
> - the database has no migrations. One schema definition exists, and a schema change edits it in
>   place.
>
> Only the repository owner changes this status.

## Role in Ratatoskr

Ratatoskr is a multi-repository system built around bounded contexts. Services own their domain models and persistence independently; this repository contains only the stable boundary types required for interoperability.

It is intended to provide:

- event envelopes and versioned event payload schemas;
- command and operation contracts;
- public OpenAPI specifications;
- shared opaque identifiers and timestamps used on the wire;
- standard error and pagination envelopes;
- document, social-source, and AI-archive interchange contracts;
- generated client/server artifacts for supported languages;
- compatibility checks used by child repositories and the workspace CI.

It is **not** a shared domain-model repository, ORM layer, utility dumping ground, or replacement for service-owned entities.

## Contract families

The full contract surface is expected to include the tree below. `(now)` marks what exists today; every other entry is a later milestone and is deliberately absent, because this repository does not create speculative schemas.

```text
crates/
├── identifiers/            (now)
├── event-envelope/         (now)
├── operation-contracts/    (now)
├── error-contracts/        (now)
├── document-contracts/
├── social-contracts/
└── ai-archive-contracts/

schemas/
├── events/                 (now, generated)
├── json-schema/            (now, generated)
└── openapi/

fixtures/                   (now)
generated/
tools/                      (now: contractsc)
```

### Event envelope

All asynchronous events use a common envelope with explicit identity, causality, ownership, and schema versioning:

```json
{
  "event_id": "018f...",
  "event_type": "social.source.upserted.v1",
  "occurred_at": "2026-08-17T10:00:00Z",
  "producer": "ratatoskr-x",
  "aggregate_id": "x-post:123",
  "correlation_id": "operation:018f...",
  "causation_id": "event:018e...",
  "tenant_id": "user:018d...",
  "schema_version": 1,
  "payload": {}
}
```

Event names follow:

```text
<bounded_context>.<aggregate>.<action>.v<major>
```

Examples:

```text
content.document.extracted.v1
github.repository.observed.v1
vault.snapshot.verified.v1
x.bookmark.observed.v1
social.source.upserted.v1
chatgpt.export.ingested.v1
claude.project.upserted.v1
knowledge.analysis.completed.v1
platform.operation.progressed.v1
telegram.interaction.received.v1
```

## Ownership boundaries

Contracts contain wire-level representations only. Each service remains responsible for:

- its internal aggregates and state machines;
- database schema and migrations;
- provider-specific credentials and metadata;
- validation beyond the public boundary;
- projections and backward-compatible interpretation.

A contract type must not import a service implementation crate or expose database implementation details. Cross-service foreign keys are represented as opaque identifiers, not shared ORM objects.

## Compatibility policy

Contract changes follow an **expand / migrate / contract** process:

1. Add an optional field, new event version, or parallel endpoint.
2. Deploy tolerant consumers.
3. Deploy producers that emit the expanded shape.
4. Migrate persisted state and projections.
5. Remove the legacy shape only after all supported consumers have moved.

Rules:

- consumers ignore unknown additive fields;
- producers do not silently change the meaning of existing fields;
- required-field additions require a new major contract version;
- enum evolution includes an explicit unknown/other representation where forward compatibility is required;
- provider-specific raw data remains opaque and lossless when the common model cannot represent it;
- generated artifacts are reproducible and never hand-edited.

## Delivery assumptions

Ratatoskr uses at-least-once event delivery. Contracts therefore carry the information needed for deterministic idempotency and tracing:

- globally unique `event_id`;
- stable aggregate and tenant identifiers;
- correlation and causation identifiers;
- explicit occurrence time;
- producer and schema version;
- command idempotency keys where applicable.

Exactly-once processing is not promised by the schema. Producers and consumers implement transactional outbox/inbox patterns and repeatable state transitions in their own repositories.

## Principal shared models

### Operations

Long-running work is represented by an operation contract rather than a synchronous HTTP response. The common model supports:

- the seven states `accepted`, `queued`, `running`, `succeeded`, `partially_succeeded`, `failed` and `cancelled`, spelled exactly as the wire spells them;
- progress phases and user-safe messages;
- typed result references;
- retryability and structured errors;
- correlation with Telegram messages, web views, and mobile notifications.

### Document interchange

The extractor publishes a typed Document IR rather than treating Markdown as the canonical format. Version one carries identity, source address, content digest, optional title and language, ordered heading and paragraph blocks, and block-addressable provenance through blob references.

### Social sources

X, Instagram, and Threads preserve different levels of authority over saved state. The common contract distinguishes official platform observations, explicit user captures, data-export observations, and legacy imports instead of reducing them to one ambiguous `is_saved` flag.

### AI archives

ChatGPT and Claude exports share normalized project, conversation, message-graph, content-part, attachment, snapshot, and completeness-report contracts. Provider-specific records remain available as opaque JSON or blob references.

## Proposed development workflow

The repository will publish versioned Rust crates and generated artifacts. A typical contract change will:

1. introduce or update the source schema;
2. regenerate Rust and client representations;
3. run schema and serialization tests;
4. run backward-compatibility checks against the previous published version;
5. update fixtures;
6. publish a tagged release;
7. update the corresponding Ratatoskr workspace changeset.

Repository-local CI is expected to include formatting, linting, unit tests, JSON Schema validation, OpenAPI validation, generated-file drift checks, and compatibility tests.

## Workspace integration

`ratatoskr-workspace` pins a compatible contracts commit together with the corresponding producer and consumer commits. Child repositories must remain buildable outside the workspace and must not commit relative path dependencies such as `../contracts` for production builds. The workspace may apply temporary local dependency overrides while validating a cross-repository changeset.

Until milestone 10 publishes the first tagged packages, a consumer depends on a contract crate as a Cargo git dependency pinned to a full commit SHA:

```toml
ratatoskr-event-envelope = { git = "https://github.com/po4yka/ratatoskr-contracts.git", rev = "216924f1420c179ad2e87ffda6cf2135befb461e" }
```

This is the only sanctioned interim form. It satisfies the rule above because it is not a path dependency and it resolves identically from a clean checkout. A branch or a tag reference is not sufficient, because neither pins: a branch moves, and a tag can be moved. When milestone 10 lands, the `git`/`rev` pair is replaced by a version requirement against the published package, and the workspace lock records the version instead of the SHA.

The pin is the `rev`, not the transport. Use `https://` while this repository is public: it resolves anonymously, so a consumer needs no credential locally and its CI needs no deploy key. A consumer that switches to `ssh://` gains a long-lived private key in CI and protects nothing that is already world-readable. If this repository is ever made private, `ssh://` becomes correct and every consumer needs the key — a consumer must not mix the two forms in one manifest, because two URLs for one source are two distinct packages.

## Non-goals

- Shared persistence entities or migrations.
- Shared business logic that belongs to one bounded context.
- Provider SDK wrappers.
- LLM prompts or analysis schemas that never cross a service boundary.
- A requirement that every internal service type be exported publicly.
- Synchronized breaking releases across all repositories.

## Initial milestones

1. Establish naming, identifier, timestamp, error, and event-envelope conventions.
2. Define operation contracts used by Platform and clients.
3. Define Document IR and extraction event schemas.
4. Define GitHub/Vault desired-state and verification events.
5. Define social-source contracts.
6. Define AI-archive snapshot and completeness contracts.
7. Generate the first public Edge OpenAPI artifacts.
8. Add compatibility and generated-drift CI.

## Project status

Milestones 1 through 5 of `docs/IMPLEMENTATION_PLAN.md` are implemented: contract metadata, shared identifiers, the event envelope, error and operation contracts, Document IR, the deterministic generator and gate, generated JSON Schema, and the fixture suite. The gate also runs in CI. Milestones 6 through 10 — social sources, AI archives, cross-language generation, the frozen compatibility baseline, package CI and publishing — are still target architecture.

Two decisions are recorded and accepted: [ADR-0001](docs/adr/0001-canonical-schema-source-format.md) (Rust-first canonical source) and [ADR-0002](docs/adr/0002-event-naming-and-major-version-strategy.md) (event naming and the two version axes). Nothing is published, so every contract here remains subject to further ADRs and contract-focused review.
