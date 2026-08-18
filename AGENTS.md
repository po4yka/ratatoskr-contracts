# Ratatoskr Contracts Agent Instructions

## Scope

These instructions apply to the `ratatoskr-contracts` repository.

This repository owns **versioned wire contracts** shared between independently released Ratatoskr components. It does not own the business implementation of any bounded context.

## Repository mission

The repository provides stable, reviewable, language-neutral contracts for:

- public API schemas;
- asynchronous command and event envelopes;
- operation status and error representations;
- identifiers and minimal value types used on the wire;
- canonical Document IR interchange;
- social-source interchange;
- AI archive interchange;
- generated client/server artifacts and compatibility reports.

Its purpose is to make cross-repository evolution explicit and safe. It must not become a shared application layer or a dependency that couples every service to one internal model.

## Current phase

Implementation milestones 1 through 4 exist. The checked-out tree contains:

- `contracts.toml` — machine-readable contract metadata (owner, family, major version, lifecycle, classification, producers, consumers, canonical path, per-field authority/unit/nullability, lint vocabulary);
- four contract crates — `crates/identifiers`, `crates/event-envelope`, `crates/error-contracts`, `crates/operation-contracts`;
- `tools/contractsc` — the deterministic generator and gate, run as `cargo contracts`;
- `schemas/json-schema/**` and `schemas/events/**` — generated JSON Schema artifacts;
- `fixtures/**` — valid, invalid and compatibility fixtures, with `fixtures/invalid-expectations.toml` naming the layer that must reject each invalid fixture.

Commands: `cargo contracts generate`, `cargo contracts check`, `cargo contracts compat <OLD> <NEW>`, `cargo test --workspace --locked`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, then `git diff --exit-code`. `DEVELOPMENT.md` holds the full list.

Milestones 5 through 10 do **not** exist. There is no Document IR, social-source or AI-archive crate, no OpenAPI document, no generated TypeScript, Kotlin or Swift, no CI configuration, no frozen compatibility baseline, and no publishing. Do not assume any of them exist unless they are present in the checked-out tree.

Keep the repository independently buildable and generated outputs reproducible. Do not create speculative schemas unrelated to an active consumer/producer changeset.

## Sources of truth

Use this order:

1. active cross-repository changeset and accepted ADRs;
2. `README.md` and repository architecture documentation;
3. the typed source crates in `crates/`; `schemas/` is generated from them and is never a source (ADR-0001);
4. generated artifacts;
5. consumer-specific copies, which must never override the canonical contract.

When generated artifacts disagree with canonical sources, canonical sources win and generation must be rerun. Never patch generated files as the primary fix.

## Hard boundaries

### This repository may contain

- JSON Schema, OpenAPI, and event schema sources;
- Rust wire types with serialization and schema derivation;
- minimal identifiers and enums required for interoperability;
- compatibility metadata and fixtures;
- generators and validators;
- generated artifacts for supported consumers;
- examples that demonstrate valid and invalid payloads.

### This repository must not contain

- ORM entities or database migrations owned by services;
- service repositories, use cases, or business workflows;
- provider SDK clients or credentials;
- generalized helper crates unrelated to wire compatibility;
- shared mutable domain state;
- private internal types that do not cross a process/repository boundary;
- UI models copied from a client;
- prompts, scraping logic, Git execution, or storage adapters.

If a type exists only to make one service implementation convenient, it belongs in that service.

## Contract design principles

1. **Wire contracts are explicit.** Every serialized field, default, nullability rule, unit, and timestamp semantic must be documented.
2. **Contracts describe facts, not assumptions.** Use names such as `first_observed_saved_at` when the provider does not expose an authoritative creation time.
3. **Unknown future variants are preservable where required.** Archive and provider imports must not discard unrecognized records.
4. **Identifiers are stable and namespaced.** Do not expose database sequence IDs as global identities.
5. **Timestamps use UTC and an unambiguous standard representation.** Document whether a timestamp is provider-authored, observed, received, or processed.
6. **Money, sizes, durations, and counters have explicit units.** Avoid unqualified numeric fields.
7. **Errors are machine-actionable.** Separate stable error codes from human-readable messages and provider diagnostics.
8. **Sensitive values are never part of broadly distributed events.** Send references or redacted metadata instead.
9. **Events represent completed facts; commands request work.** Do not name a request as if it already happened.
10. **Schemas are minimal.** Do not expose service-private fields merely because they exist in a database row.

## Event envelope

All asynchronous contracts must use the canonical event/command envelope once implemented. The envelope should include, at minimum:

- globally unique event or command ID;
- event or command type, whose `.v<major>` suffix is the **payload** major;
- envelope schema version, which is the **envelope** major and never mirrors the payload major (ADR-0002);
- occurrence/request timestamp;
- producer;
- aggregate or target ID;
- correlation ID;
- causation ID when applicable;
- owner/tenant identity where required;
- payload.

Envelope metadata must support at-least-once delivery, idempotency, traceability, replay, and audit without embedding secrets.

Use the naming convention:

```text
<bounded_context>.<aggregate>.<action>.v<major>
```

Examples:

```text
content.document.extracted.v1
github.repository.observed.v1
vault.snapshot.verified.v1
social.source.upserted.v1
knowledge.analysis.completed.v1
platform.operation.progressed.v1
```

Do not rename existing event types for style. A semantic rename is a compatibility change and requires a migration plan.

## Compatibility policy

Default to **expand/migrate/contract**:

1. **Expand:** add backward-compatible fields, variants, endpoints, or events.
2. **Migrate:** update consumers and producers while both old and new forms remain supported.
3. **Contract:** remove obsolete forms only after usage and rollout evidence confirms safety.

### Backward-compatible changes generally include

- adding optional fields with documented behavior;
- adding new event types;
- adding enum variants when consumers preserve or safely handle unknown values;
- adding endpoints;
- relaxing validation without changing existing meaning.

### Breaking changes generally include

- removing or renaming fields;
- changing a field's meaning, unit, authority, or nullability;
- changing identifier semantics;
- changing required ordering or deduplication rules;
- reusing an event name for a different fact;
- making a previously optional field required;
- changing enum handling when consumers are exhaustive;
- changing canonicalization or hashing rules.

A breaking change requires:

- a new major contract/event version or endpoint version;
- an active changeset listing all producers and consumers;
- migration and rollback plans;
- compatibility fixtures for old and new forms;
- explicit deprecation and removal criteria.

Never coordinate an unversioned simultaneous cutover across repositories.

## Canonical source and generated artifacts

**Decided in [ADR-0001](docs/adr/0001-canonical-schema-source-format.md): Rust-first.** The canonical source of every contract's shape is the Rust type in `crates/*/src/**.rs`. Everything under `schemas/` is generated from those types by `cargo contracts generate` and must never be hand-edited.

The generated JSON Schema is a **lower bound** on validity: it accepts payloads the Rust type rejects, because JSON Schema draft 2020-12 cannot express the cross-field invariants or the one-spelling-per-instant rule. Each invalid fixture therefore declares which layer must reject it in `fixtures/invalid-expectations.toml`.

OpenAPI is explicitly not decided by ADR-0001; `ARCHITECTURE.md` S4 gives it a different canonical source, and a follow-up ADR at milestone 8 decides it.

Do not maintain two hand-edited authoritative copies.

Generated files must:

- contain a generated-file marker;
- be deterministic for identical inputs and tool versions;
- record generator/version metadata;
- be reproducible in CI;
- never be edited manually;
- be checked for drift.

A PR changing canonical sources should include regenerated outputs when the repository policy requires committed artifacts.

## Document IR rules

The canonical document contract preserves structure and provenance. It must not reduce documents to Markdown as the only representation.

When changing Document IR:

- preserve block ordering;
- define block-specific required fields;
- keep source/provenance spans addressable;
- support unknown or extension blocks where forward compatibility requires it;
- document normalization and hashing rules;
- avoid embedding extraction-engine-specific implementation details;
- ensure clients can render or safely ignore unsupported blocks.

Do not add LLM summaries or embeddings to the canonical extracted document contract. Those belong to Knowledge outputs linked by stable source IDs and content hashes.

## Social contract rules

Social contracts must distinguish:

- platform;
- acquisition method;
- authority of saved state;
- provider external ID;
- canonical URL;
- provider-authored publication time;
- user capture time;
- observed upstream status;
- native collections versus Ratatoskr collections;
- raw blob references versus normalized content.

Never model Instagram or Threads explicit capture as authoritative membership in the provider's native Saved list. X bookmarks may use authoritative provider state when obtained through the supported API, but exact bookmark timestamps must not be fabricated.

## AI archive contract rules

AI archive contracts must support:

- immutable provider export snapshots;
- projects and project sources;
- conversations as graphs, not only linear lists;
- message revisions, branches, regenerated responses, and parent relationships;
- heterogeneous content parts;
- files, artifacts, Canvas-like assets, and citations;
- provider-specific unknown records;
- explicit completeness and warning reports;
- conservative upstream deletion state.

Do not equate ChatGPT product history with OpenAI API Conversations, or Claude product archives with Anthropic inference Messages.

## Review requirements

Every contract change must identify:

- owning bounded context;
- producer repositories;
- consumer repositories;
- authority and provenance of every new field;
- compatibility classification;
- rollout and rollback order;
- security/privacy impact;
- persistence and replay impact;
- examples and validation tests.

Reject fields that are added with vague names such as `status`, `data`, `metadata`, or `timestamp` without a precise documented contract.

## Testing expectations

When tooling exists, changes should include the applicable checks:

- schema validation for positive and negative fixtures;
- serialization round trips;
- golden snapshots of generated schemas/artifacts;
- backward/forward compatibility tests;
- unknown-field and unknown-variant behavior;
- canonicalization and content-hash tests;
- OpenAPI linting;
- deterministic generation and drift checks;
- consumer compile tests for supported generated clients.

Property-based testing is preferred for identifiers, canonicalization, parsers, and serialization invariants.

Do not claim compatibility based only on successful compilation of this repository. Validate representative producers and consumers through the workspace changeset.

## Security and privacy

- Never include access tokens, refresh tokens, session cookies, passwords, webhook secrets, or raw authorization headers in contracts or fixtures.
- Avoid broad events containing private source bodies when a blob reference and authorized retrieval path are sufficient.
- Mark fields containing user content, provider identifiers, or audit data so retention and redaction behavior can be designed explicitly.
- Ensure error contracts do not leak provider responses containing secrets.
- Keep public API contracts separate from privileged internal/admin contracts.
- `extensions` is a preservation channel, not an authoring channel (ADR-0008). A producer never invents a key in it; every value a producer intends a consumer to read is a typed field in this repository. A relay forwards upstream keys verbatim, which is what the channel exists for. The testable form is `extensions.is_empty()` on the envelopes a service **constructs**, never on the envelopes it forwards.

## Git and PR workflow

- Work on a task-specific branch and link the cross-repository changeset when other repositories are affected.
- Keep one conceptual contract migration per PR when possible.
- Do not mix contract semantics with unrelated generator refactors.
- State whether the PR is additive, deprecating, or breaking.
- List affected producers and consumers in the PR body.
- Include generated artifact changes and compatibility evidence.
- Never merge a breaking contract before compatible consumer paths are available according to the rollout plan.

## Completion criteria

A contract task is complete only when:

- the owning context and consumers are identified;
- canonical sources and generated outputs agree;
- field authority, units, nullability, and timestamp semantics are documented;
- compatibility classification is explicit;
- schemas and fixtures validate;
- required compatibility tests pass;
- no service-private domain/ORM logic leaked into this repository;
- no secrets or sensitive real-user data entered examples;
- rollout, rollback, and deprecation plans are recorded in the changeset;
- workspace-level producer/consumer validation has been performed when applicable.
