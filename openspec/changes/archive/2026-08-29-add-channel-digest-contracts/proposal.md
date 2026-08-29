## Why

The approved channel-digest boundary needs typed messages before Knowledge, Platform, the new digest service, or Telegram can implement against it. Without canonical payloads, each repository would invent incompatible identifiers, windows, idempotency, and source-provenance semantics.

## What Changes

- Add a `ratatoskr-channel-digest-contracts` crate for provider-neutral channel/digest value types and the additive first-version messages defined by workspace change `add-channel-digest-system-contract`.
- Define `channel_digest.subscription.set_requested.v1`, `channel_digest.run.requested.v1`, and deployment-wide `channel_digest.schedule.occurrence_requested.v1` command payloads with explicit owner authority where applicable, stable idempotency/occurrence identity, canonical public-channel locator, desired state/trigger, and exact UTC timing semantics.
- Define `knowledge.channel_digest_recap.requested.v1` and `knowledge.channel_digest_recap.completed.v1` messages carrying only immutable manifest/result references, content digests, counts, language, coverage, and provenance; channel post bodies and provider credentials remain absent.
- Define closed outcomes and safe failure classes for empty windows, partial source coverage, unavailable analysis, reauthorization, invalid public channels, and successful result linkage without embedding provider errors.
- Register contract metadata, canonical Rust sources, generated JSON Schemas/TypeScript declarations, positive/negative/compatibility fixtures, and producer/consumer declarations.
- Keep public REST/OpenAPI shapes, ORM state, MTProto session types, prompts, Telegram rendering, email delivery, and provider-private diagnostics outside this repository.

## Capabilities

### New Capabilities

- `channel-digest-contracts`: Wire values, commands, recap facts, validation invariants, compatibility, and privacy rules shared by digest, Knowledge, Platform, Telegram, and workspace consumers.

### Modified Capabilities

None.

## Impact

Affected surfaces are the Cargo workspace, `contracts.toml`, identifier/entity-kind registries where required, generated schemas and TypeScript declarations, fixtures, compatibility reports, and documentation. The new crate is additive and publishes no provider SDK or service logic.

Contracts merge first. Knowledge consumes recap requests before `ratatoskr-channel-digests` produces them; Platform and Telegram consume the published operation/result references later. Rollback stops producers first and removes the unconsumed additive contract last. If no producer has shipped, deleting the unmerged change is the complete rollback.
