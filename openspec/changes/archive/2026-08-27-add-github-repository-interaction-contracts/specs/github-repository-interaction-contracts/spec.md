## Purpose

Defines strict reusable wire types for GitHub repository preview requests, confirmed repository actions, and component-level outcomes consumed across independently built Ratatoskr services.

## ADDED Requirements

### Requirement: Preview contracts preserve stable identity and bounded display fields

The contract SHALL define strict request and response types for a canonical GitHub repository URL. A successful response SHALL carry the stable positive GitHub numeric repository identifier separately from the current `owner/name` alias and canonical HTTPS URL, plus an optional bounded description, non-negative star count, optional bounded primary language, and a closed set of available action capabilities. Unknown fields and invalid URLs, identities, counts, or bounded strings SHALL be rejected.

#### Scenario: Valid preview data round-trips

- **WHEN** a preview containing stable identity, `owner/name`, canonical URL, description, stars, language, account reference, and action capabilities is serialized and deserialized
- **THEN** every field round-trips without changing identity or collapsing absent optional values into invented text

#### Scenario: Unknown preview members are refused

- **WHEN** a preview request or response contains an unrecognized member
- **THEN** strict deserialization rejects it rather than silently discarding contract drift

### Requirement: Action contracts carry mode, target, confirmation, and idempotency

The contract SHALL define the closed repository modes `metadata`, `track`, and `star`; an action request SHALL carry the exact stable preview target, an optional account reference only where the selected mode requires it, a bounded opaque confirmation evidence reference, and a bounded idempotency key. The wire type SHALL contain no GitHub credential, Telegram callback token payload, provider error body, or mutable policy JSON.

#### Scenario: A star request names its acting account

- **WHEN** a valid `star` request is decoded
- **THEN** it identifies the stable repository, acting account reference, confirmation evidence, and idempotency key without carrying a provider credential

#### Scenario: A mode outside the first-version vocabulary is refused

- **WHEN** an action request names an unknown repository mode
- **THEN** strict deserialization rejects the request

### Requirement: Component outcomes encode truthful partial results

The contract SHALL report exactly three named components: metadata, provider star, and desired backup. Each component SHALL use a closed tagged outcome that distinguishes `succeeded`, `already_applied`, `accepted`, `refused`, `failed`, and `skipped`; refused/failed/skipped outcomes SHALL carry only a closed safe reason code appropriate to that state. The aggregate outcome SHALL be derived consistently as `succeeded`, `partial`, or `failed`, and inconsistent aggregate/component combinations SHALL be rejected by construction or validation.

#### Scenario: Star success with backup failure is partial

- **WHEN** the provider-star component is `succeeded`, metadata is `succeeded`, and desired backup is `failed`
- **THEN** the aggregate is `partial` and serialization retains all three component outcomes

#### Scenario: Accepted backup is not backup success

- **WHEN** desired policy has been accepted for publication but Vault has not verified storage
- **THEN** the desired-backup component is `accepted`, never `succeeded` or a backup-verification claim

### Requirement: Generated artifacts and API compatibility remain deterministic

The Rust API, JSON Schema, TypeScript output, and compatibility baseline SHALL describe the same closed shapes deterministically. The new first-version types SHALL be additive to existing GitHub repository-analysis contracts.

#### Scenario: Regeneration is clean

- **WHEN** contract artifacts are generated twice from the same source
- **THEN** the second generation produces no diff and compatibility validation accepts existing repository-analysis consumers
