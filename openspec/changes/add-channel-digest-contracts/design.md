## Context

See `proposal.md` and the workspace `channel-digest-system` delta. Existing command and event envelopes
already provide message identity, tenant scope, correlation, causation, occurred/issued time, and
unknown additive field preservation. Existing identifiers provide internal user and operation
identity, while `ContentDigest` is the canonical SHA-256 value. No current crate owns channel-digest
wire types.

## Goals / Non-Goals

**Goals:**

- Publish one canonical Rust source for all digest commands and Knowledge recap facts.
- Enforce cross-field window, count, reference, and outcome invariants before a message is accepted.
- Generate deterministic JSON Schema and TypeScript output with privacy and compatibility fixtures.

**Non-Goals:**

- REST DTOs, database rows, provider session types, prompt/output rendering, or runtime transport.
- A generic feed or arbitrary source aggregation contract.

## Decisions

### Use a dedicated `ratatoskr-channel-digest-contracts` crate

The crate contains channel username, subscription/run/manifest/result references, UTC window,
trigger/outcome/failure vocabularies, and four envelope payload families. It depends only on existing
identifier and envelope crates plus the workspace serialization/schema stack.

Adding the types to operation contracts was rejected because the source manifest and Knowledge recap
facts form a domain vocabulary used without a Platform operation in scheduled replay and evaluation.
Adding separate subscription and recap crates was rejected because neither has an independent release
or dependency boundary.

### Reuse envelopes and keep payload identities explicit

`ChannelDigestSubscriptionSetRequested`, `ChannelDigestRunRequested`, and
`KnowledgeChannelDigestRecapRequested` implement the existing command payload trait. `Completed` and
`Failed` implement the event payload trait. Payloads repeat only the domain identities needed for idempotent handling: internal
owner, operation/run, manifest/result reference, content digest, trigger, window, language, and bounded
counts. Provider IDs, post bodies, session bytes, prompts, model output, and provider errors are
structurally absent.

A free-form metadata map was rejected because it defeats privacy lint and closed invariant checking.

### Validate semantic invariants in Rust after schema validation

Serde and generated schema enforce required members, closed vocabularies, string/count bounds, and
unknown policy. Hand-written validation enforces `window_start < window_end`, scheduled occurrence
presence only for scheduled runs, desired-state locator/reference rules, `source_count <= 100`,
`channel_count <= 20`, non-empty completion counts, and completion/failure field exclusivity.

Encoding timestamps as integers was rejected because the fleet already uses canonical RFC3339 UTC
instants. Letting consumers validate independently was rejected because malformed cross-field states
would become a published contract.

### Register one source of truth and generated surfaces

`contracts.toml` gains the new service, entity kinds where exact fixture aggregates require them, all
root types, producer/consumer declarations, field authority, units, privacy classification, and any
fully justified lint waivers. `contractsc` generates schemas and TypeScript declarations from the Rust
types. Positive, negative, old-consumer/new-producer, and new-consumer/old-producer fixtures cover each
message and additive unknown member behavior.

Manual schema files were rejected because they can drift from the Rust surface. A second major was
rejected by development status and because every change is additive.

## Risks / Trade-offs

- [The dedicated crate becomes a service SDK] → expose wire values only; forbid clients, persistence,
  provider types, and business services in its public API.
- [A future source limit changes] → keep the first-version upper bounds normative; changing them is a
  deliberate compatible constraint review, not an untyped extension.
- [A sensitive field enters a fixture] → secret/provenance lint plus explicit forbidden-member tests
  scan canonical and generated artifacts.
- [Generated Rust/JSON/TypeScript surfaces diverge] → generation drift and round-trip tests run in the
  full Contracts gate.

## Migration Plan

1. Add RED compile/serialization tests and invalid fixtures before the crate or generated artifacts.
2. Add the crate, canonical types, metadata, and generated outputs until the focused tests pass.
3. Publish Contracts before Knowledge or the new digest producer consumes the subjects.
4. Rollback stops all producers first; remove the additive crate only after no deployed consumer or
   stored message requires it.
