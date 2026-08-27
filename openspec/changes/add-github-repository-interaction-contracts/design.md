## Context

See `proposal.md` and `specs/github-repository-interaction-contracts/spec.md`. The existing `ratatoskr-github-contracts` crate owns repository-analysis wire types and is already consumed by GitHub Catalog. The repository's generator derives JSON Schema and TypeScript from Rust types and checks a committed public API baseline.

## Goals / Non-Goals

**Goals:**

- Add strict additive types that both GitHub and Telegram can pin.
- Prevent inconsistent aggregate status and unsafe provider text from becoming representable.
- Keep stable numeric identity distinct from mutable display aliases.

**Non-Goals:**

- Registering an event subject or changing the platform operation envelope.
- Encoding Telegram token storage or GitHub provider adapter types.
- Adding a second contract crate or production dependency.

## Decisions

### D1: Add an `interactions` module to the existing GitHub contract crate

The new API is part of the same GitHub bounded context and shares repository value types, so a separate crate would create an artificial package boundary. Existing repository-analysis exports remain untouched.

### D2: Use validated newtypes and closed tagged enums

The target contains positive `u64` provider identity, validated bounded `owner/name`, and canonical HTTPS URL. Mode and component outcomes are `snake_case` closed enums with `deny_unknown_fields`. Safe reason codes are closed and carry no provider message. Plain strings/open `serde_json::Value` were rejected because they defeat compatibility and secret linting.

### D3: Derive aggregate status from component outcomes

Construction/validation computes `succeeded`, `partial`, or `failed` from the three components. Deserialization validates any wire aggregate against those facts. This gives consumers a convenient lead while preventing a producer from claiming full success beside a failed component.

### D4: Treat backup `accepted` as a distinct status

`accepted` applies to desired backup policy only. It is not a synonym for component success and cannot be used for provider star or metadata. This preserves the boundary between Catalog intent and Vault evidence.

## Risks / Trade-offs

- [Outcome vocabulary is too generic] -> Validate status applicability by component and add invalid fixtures for illegal combinations.
- [Generated artifacts drift] -> Run the standard generator, inspect the complete diff, then run deterministic generation and API compatibility gates.
- [A future list step needs another component] -> Add it to the same first-version contract through an explicit coordinated change; do not pre-generalize this slice.

## Migration Plan

Merge this additive contract first. GitHub then pins the new revision and produces it; Telegram pins the same revision afterward. Rollback before consumers ship is a normal revert. Once consumers compile against the types, roll back the consumers before reverting the contract addition.
