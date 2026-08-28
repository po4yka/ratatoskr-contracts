# Ratatoskr Contracts

`ratatoskr-contracts` is the wire-contract repository for Ratatoskr. It defines the versioned structures exchanged between independently deployed services and the public API artifacts consumed by Ratatoskr clients.

> **Status:** milestones 1–9 implemented. Shared identifiers, the event envelope, error contracts, operation contracts, Document IR, the social-source contracts, the AI-archive contracts, the backup-policy contracts, the notification contracts, the channel-digest contracts, the blob-transfer protocol contracts and the deterministic generator (`cargo contracts`) exist, together with generated JSON Schema, the matching TypeScript declarations under `generated/typescript/`, the fixture suite, frozen public-API baselines under `compat/api/`, and CI that runs the documented gate plus the compatibility, determinism and packaging jobs of `.github/workflows/contracts.yml`. OpenAPI and package publication do **not** exist yet.

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
- document, social-source, AI-archive, backup-policy, user-facing-notification, and public-channel-digest interchange contracts, plus the shared upload transfer protocol;
- generated client/server artifacts for supported languages;
- compatibility checks used by child repositories and the workspace CI.

It is **not** a shared domain-model repository, ORM layer, utility dumping ground, or replacement for service-owned entities.

## Contract families

The full contract surface is expected to include the tree below. `(now)` marks what exists today; every other entry is a later milestone and is deliberately absent, because this repository does not create speculative schemas.

```text
crates/
├── identifiers/               (now)
├── event-envelope/            (now)
├── channel-digest-contracts/  (now)
├── operation-contracts/       (now)
├── error-contracts/           (now)
├── document-contracts/        (now)
├── social-contracts/          (now)
├── ai-archive-contracts/      (now)
├── backup-contracts/          (now)
├── notification-contracts/    (now)
└── blob-transfer-contracts/   (now)

schemas/
├── events/                 (now, generated)
├── json-schema/            (now, generated)
└── openapi/

fixtures/                   (now)
compat/api/                 (now: frozen public-API baselines)
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
vault.backup_policy.acknowledged.v1
platform.notification.raised.v1
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

X, Instagram, and Threads preserve different levels of authority over saved state. The common contract (`ratatoskr-social-contracts`) distinguishes official platform observations, explicit user captures, data-export observations, and legacy imports instead of reducing them to one ambiguous `is_saved` flag. A `SocialSourceSnapshot` carries the normalized record — identity, optional author (absent means authorship was unobservable, never that the source has no author), text, media by `BlobRef`, quote/reply/repost relations, provider-native folder memberships — beside the facts of one capture of it: acquisition method, saved authority, completeness with explanatory warnings, an opaque sync checkpoint, and observed upstream availability. The `social.source.captured.v1` and `social.source.updated.v1` events carry the whole snapshot inside the common envelope, and `social.source.removed.v1` states that a user's library stopped holding a source — a fact about the library that never claims anything about upstream state.

### AI archives

ChatGPT and Claude exports share one normalized grammar in `ratatoskr-ai-archive-contracts`: an import head names the immutable export evidence by `BlobRef` beside its completeness report; project, conversation and message nodes carry parser stamps and provider external ids; conversations are graphs whose messages reference parents, so branches and regenerated answers survive normalization. One shared content-part grammar (text, markdown, image, asset, citation, tool call, tool result) serves both providers, and unrecognized parts are preserved verbatim through normalization and re-export. The `ai_archive.archive.imported.v1`, `ai_archive.conversation.added.v1` and `ai_archive.conversation.updated.v1` events carry the head and whole conversations inside the common envelope.

### Backup policies

`ratatoskr-github` publishes `DesiredBackupPolicy` (`ratatoskr-backup-contracts`): a monotonic policy version naming, per repository, a mirror cadence class, priority and size hints, and explicit exclusions — WHAT must be preserved as an explicit, versioned, auditable contract between two services that never share a database, instead of implicit job configuration. Coverage is default-deny between versions: a catalog repository a version does not name stays out of scope until a successor names it, and an entry pointing outside Vault's catalog is reportable drift rather than a silent skip. Vault answers each version through the `vault.backup_policy.acknowledged.v1` event inside the common envelope — accepted or rejected with machine-actionable reasons beside the last policy version it fully applied.

### Notifications

The legacy monolith notified in-process; the fleet cannot. `ratatoskr-notification-contracts` is the documented bus surface `ratatoskr-telegram`'s notification sender honors: one registered event type, `platform.notification.raised.v1`, through which `ratatoskr-knowledge`, `ratatoskr-github`, `ratatoskr-vault` and `ratatoskr-x` state the completed fact that one of their users should be told something — never an order to send, so preference filtering, dedupe and channel choice stay on Telegram's side of the wire.

A raised notification carries its own identity (`NotificationId`, also the aggregate as `notification:<uuid>` and the logical key for suppressing re-raises), a class from a versioned taxonomy whose unknown tokens are preserved rather than rejected — six classes at registry version 1: `operation_completed`, `operation_failed`, `analysis_ready`, `backup_outcome`, `watch_triggered`, `archive_imported` — the recipient in the closed tenancy grammar, carrier-safe title and optional message text, opaque correlation references (`operation_ref`, `analysis_ref`), and advisory delivery hints (a priority level and a daily quiet-hours window in seconds from UTC midnight) whose enforcement is nobody's job but the consumer's. Delivery guarantees are the bus's own at-least-once; nothing stronger is promised here.

### Channel digests

`ratatoskr-channel-digest-contracts` is the credential-free boundary between Platform,
`ratatoskr-channel-digests`, and Knowledge. Platform emits
`channel_digest.subscription.set_requested.v1` and `channel_digest.run.requested.v1`; the digest
service consumes those commands, acquires public-channel source evidence under its own provider
session, and emits `knowledge.channel_digest_recap.requested.v1` with only an owner-bound immutable
manifest reference, its SHA-256 digest, bounded counts, window, and language. Knowledge answers with
exactly one of `knowledge.channel_digest_recap.completed.v1` or
`knowledge.channel_digest_recap.failed.v1`.

The completion contains stable analysis/result references, content digests, completion time, and
exact selected/included/omitted coverage — never recap narrative or post bodies. The failure carries
one closed safe code and no provider diagnostic. Public-channel usernames, MTProto sessions,
Telegram destinations, prompts, model selectors, URLs, source bodies, and credentials are absent
from the cross-service messages. Consumers preserve additive fields for compatibility; producers
must call each payload's `validate_for_publish` and author an empty extension map.

### Blob transfers

The legacy monolith accepted uploads synchronously in one process; the fleet cannot. `ratatoskr-blob-transfer-contracts` is one chunked, resumable, digest-first transfer discipline for every upload-capable client delivering bytes to a receiving service's blob store — mobile and export-agent on the sending side, extractor and the AI-archive services receiving — so no receiver invents its own wire dialect.

A session opens digest-first: the declaration carries total size, media type and the whole-payload SHA-256 before any byte moves, so a receiver refuses unwanted uploads for free. Chunks are addressed by zero-based index over a declared fixed chunk size (bounds are protocol constants), replay of identical bytes is idempotent, divergent bytes conflict without corrupting the record, and an interrupted client resumes from a status view naming exactly which indices survived. Finalize verifies the streamed digest and answers two terminal truths: `stored` carrying the complete `BlobRef` the workspace store spec `blob-references` defines, or explicit `digest_mismatch` with expected and computed digests. Failures speak stable `blob_transfer.` codes through the shared error envelope; the canonical HTTP binding is normative prose beside transport-honest types, per platform ADR-0015's transfer route class. Receiving services keep ownership of storage placement, quarantine policy and their own announcement surfaces.

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
ratatoskr-channel-digest-contracts = { git = "https://github.com/po4yka/ratatoskr-contracts.git", rev = "<published-full-commit-sha>" }
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

1. Define directories and machine-readable contract metadata.
2. Choose the Rust-first canonical source and deterministic generator architecture.
3. Implement shared identifiers and the event envelope.
4. Implement operation and error contracts.
5. Implement Document IR and provenance.
6. Implement social-source contracts.
7. Implement AI archive graph and content contracts.
8. Add deterministic Rust and TypeScript generation.
9. Add fixture, compatibility, and package CI.
10. Publish the first tagged artifacts and integrate them in one vertical slice.

## Project status

Milestones 1 through 9 of `docs/IMPLEMENTATION_PLAN.md` are implemented: contract metadata, shared identifiers, the event envelope, error and operation contracts, Document IR, the social-source contracts, the AI-archive contracts, the backup-policy contracts, the deterministic generator and gate, generated JSON Schema, the fixture suite, and milestone 8's deterministic TypeScript generation mirroring `schemas/` one-to-one. Milestone 9 is the assurance layer: `.github/workflows/ci.yml` runs the gate, and `.github/workflows/contracts.yml` adds the `compatibility` job (every crate's public API diffed against its frozen baseline under `compat/api/`), the `determinism` job (regeneration reproduces the committed tree byte for byte) and the `package` job (the TypeScript output uploaded as a workflow artifact). What remains of milestones 8 through 10 — any further cross-language targets and publishing tagged packages to a registry — is still target architecture.

Two decisions are recorded and accepted: [ADR-0001](docs/adr/0001-canonical-schema-source-format.md) (Rust-first canonical source) and [ADR-0002](docs/adr/0002-event-naming-and-major-version-strategy.md) (event naming and the two version axes). Nothing is published, so every contract here remains subject to further ADRs and contract-focused review.
