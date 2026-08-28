## Purpose

Defines the canonical first-version wire values and replay-safe messages exchanged when Platform, the channel-digest owner, and Knowledge create subscriptions and grounded public-channel recaps.

## ADDED Requirements

### Requirement: Channel-digest identifiers and values have one wire grammar

The contract SHALL define distinct UUID-backed identifiers for a digest subscription, digest run, and digest result; a normalized `ChannelUsername`; a `DigestWindow`; an `OutputLanguage`; and bounded idempotency and safe failure values. `ChannelUsername` SHALL contain 5 through 32 lowercase ASCII letters, digits, or underscores and begin with a letter. A `DigestWindow` SHALL contain RFC 3339 UTC instants with `start_at < end_at`, an exclusive end, and duration no greater than seven days. `OutputLanguage` SHALL accept only `ru` or `en` in this contract.

All deserialization SHALL enforce the same invariants as constructors and generated schemas wherever the schema vocabulary can express them. Provider numeric IDs, Telegram user/chat IDs, database sequence IDs, raw URLs, and session values SHALL NOT substitute for these identifiers.

#### Scenario: Degenerate window is rejected

- **WHEN** a payload has equal start and end instants or an end before its start
- **THEN** both typed deserialization and the contract fixture gate reject it

#### Scenario: Mixed-case channel is not canonical wire data

- **WHEN** a payload carries `Example_Channel` as a channel username
- **THEN** the contract rejects it rather than silently normalizing signed or hashed bytes

### Requirement: Subscription set command carries complete authority and idempotency

`channel_digest.subscription.set_requested.v1` SHALL be a command payload containing `operation_id`, authenticated internal `owner`, stable `idempotency_key`, normalized `channel_username`, and closed `desired_state` of `active` or `inactive`. The command SHALL contain no Telegram chat/user identifier, MTProto session, invite hash, provider credential, display title, post content, or claimed current subscription state.

The carrying command envelope SHALL name the Platform operation as its correlation and aggregate authority according to the canonical envelope rules. Reuse of one idempotency key with different owner, username, or desired state SHALL be a conflict rather than a duplicate.

#### Scenario: Exact subscription replay is valid

- **WHEN** two command envelopes carry the same owner, operation, idempotency key, channel username, and desired state under distinct transport command IDs
- **THEN** both payloads validate and consumers can recognize one logical mutation

#### Scenario: Reused key changes desired state

- **WHEN** a second payload reuses an idempotency key but changes `active` to `inactive`
- **THEN** the consumer must classify it as an idempotency conflict and cannot interpret it as replay

### Requirement: Run command distinguishes on-demand and scheduled authority

`channel_digest.run.requested.v1` SHALL contain `operation_id`, `owner`, `digest_run_id`, `idempotency_key`, validated `window`, and `output_language`. Its closed `trigger` SHALL be either `on_demand` with the authenticated request's accepted instant or `scheduled` with stable Platform schedule and occurrence identifiers plus the due instant. Scheduled trigger due time SHALL equal the window end; on-demand accepted time SHALL equal the window end.

The command SHALL contain no client-selected model/provider, prompt, source body, arbitrary tenant selector, MTProto credential, Telegram destination, or claimed result. An exact replay SHALL preserve run, operation, trigger, window, and language.

#### Scenario: Scheduled trigger contradicts its window

- **WHEN** a scheduled run payload's due instant differs from `window.end_at`
- **THEN** typed deserialization rejects the command before domain work begins

#### Scenario: On-demand command contains a model selector

- **WHEN** a producer authors an additional model/provider field instead of leaving extensions empty
- **THEN** contract construction or linting rejects the producer-authored extension

### Requirement: Recap request references immutable source evidence without bodies

`knowledge.channel_digest_recap.requested.v1` SHALL be a command payload containing `operation_id`, `owner`, `digest_run_id`, validated `window`, `output_language`, `source_count` from 1 through 100, `channel_count` from 1 through 20 and no greater than `source_count`, an owner-scoped `manifest_ref`, and the manifest's SHA-256 digest. The request SHALL carry the analysis family and contract identifiers fixed to the first-version channel recap.

The payload SHALL NOT contain channel/post bodies, usernames, display titles, public links, provider IDs, MTProto/Bot API credentials, prompts, model choice, or raw errors. The manifest reference and digest SHALL be stable for an exact replay; changing either under the same run SHALL be a conflict.

#### Scenario: Full valid request round-trips

- **WHEN** a request with 12 sources across 3 channels, a valid window, owner-scoped manifest reference, and SHA-256 digest is serialized and parsed
- **THEN** every authoritative field round-trips unchanged and the extensions map remains empty

#### Scenario: Source content is placed in an extension

- **WHEN** a producer constructs a recap request with a `posts` or `text` extension
- **THEN** producer tests fail because authored messages must have an empty preservation extension map

### Requirement: Recap completion links one verified result and exact coverage

`knowledge.channel_digest_recap.completed.v1` SHALL contain `owner`, `operation_id`, `digest_run_id`, the requested manifest digest, a stable Knowledge analysis reference, a stable digest-result reference, the result projection's SHA-256 digest, `completed_at`, and coverage. Coverage SHALL contain selected, included, omitted, and channel counts such that selected equals the request source count, included is positive, included plus omitted equals selected, and channel count is positive and no greater than the request channel count.

The completion SHALL contain neither the recap narrative nor source bodies; authorized consumers retrieve the result projection by reference. A completion whose owner, run, manifest digest, or counts contradict the accepted request SHALL not be linkable as that run's result.

#### Scenario: Coverage arithmetic is inconsistent

- **WHEN** completion coverage says 10 selected, 8 included, and 1 omitted
- **THEN** typed deserialization rejects the completion rather than guessing which count is authoritative

#### Scenario: Completion contains only references and counts

- **WHEN** a valid completion fixture is inspected
- **THEN** it exposes result/analysis identity, digests, coverage, and completion time but no headline, overview, topic text, post body, or credential

### Requirement: Recap failure is safe and terminally typed

`knowledge.channel_digest_recap.failed.v1` SHALL identify the same owner, operation, run, and manifest digest and SHALL carry one closed safe failure code from `manifest_unavailable`, `manifest_integrity`, `unsupported_language`, `context_budget`, `provider_unavailable`, `provider_timeout`, `invalid_output`, `cost_budget`, or `cancelled`, plus `failed_at`. It SHALL NOT carry provider response text, prompt/source excerpts, stack traces, URLs, credentials, or a successful result reference.

#### Scenario: Provider failure remains content-free

- **WHEN** inference fails after the provider returns a diagnostic containing source text and an authorization header
- **THEN** the published failure contains only `provider_unavailable` or `provider_timeout` and the stable request linkage

### Requirement: Generated artifacts and compatibility declare real consumers

The canonical Rust types SHALL generate deterministic JSON Schemas and TypeScript declarations. Contract metadata SHALL name Platform as producer of subscription/run commands, the digest service as their consumer and as producer of recap requests, Knowledge as recap request consumer and completion/failure producer, and the digest service as completion/failure consumer. Valid, invalid, and additive compatibility fixtures SHALL cover every message and cross-field invariant.

Adding the crate and message types SHALL be classified as additive. No existing event meaning, identifier grammar, or notification payload SHALL be silently changed.

#### Scenario: Old consumer sees an additive message family

- **WHEN** compatibility classification compares the pre-change contract tree with the new tree
- **THEN** existing schemas remain compatible and the new channel-digest artifacts are reported as additive rather than breaking
