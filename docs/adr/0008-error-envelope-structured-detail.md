# ADR-0008: Structured detail on the error envelope

> Status: Accepted  
> Last reviewed: 2026-08-18

## Context

`ARCHITECTURE.md` S5.5 sketched the error envelope with `pub details: Option<serde_json::Value>`. The shipped `ErrorEnvelope` has no such member: field-level detail is the typed `field_violations: Vec<FieldViolation>`, and unknown additive members land in the preserved `extensions` map.

Two questions were left open by that divergence, and the S5.5 sketch — an authority-2 document, which under `AGENTS.md`'s source-of-truth order outranks the crate — still declared the field, so the next reader would have re-added it.

1. Does the error contract carry an untyped `details` blob?
2. What, exactly, is `extensions` for, given that `THREAT_MODEL.md` already records it as an unbounded producer-writable channel?

## Drivers

1. `AGENTS.md` design principle 1: every serialized field, default, nullability rule, unit and timestamp semantic must be documented.
2. `AGENTS.md` principle 7: separate stable error codes from human-readable messages **and provider diagnostics**.
3. `AGENTS.md` review rule: reject vague field names "without a precise documented contract".
4. `AGENTS.md` security rule: keep public API contracts separate from privileged internal/admin contracts. `contracts.toml` classifies `core.error_envelope` as `public`.
5. `ARCHITECTURE.md` S14: error payloads exclude stack traces and raw provider responses **by default**; provider-specific raw payloads use **bounded `metadata` or `unknown` fields** and are not propagated indiscriminately.
6. `README.md`: consumers ignore unknown additive fields, and provider-specific raw data remains opaque and lossless when the common model cannot represent it.
7. Adding an optional field later is backward-compatible (`AGENTS.md`); removing one after publication is breaking.

## Options

### Option A — re-add `details: Option<serde_json::Value>`, per a literal reading of S5.5

Against: on a `public` contract it is an unbounded free-JSON member with no documented meaning, which is what drivers 3, 4 and 5 are written against. No producer asks for one.

### Option B — add a typed `details` now, e.g. `Option<ProviderDiagnostic>`

Against: speculative. No producer or consumer needs it today, and adding an optional typed member later is the cheap direction.

### Option C — no `details` on this contract in v1; record the sanctioned path for when one is needed (chosen)

For: it costs nothing today, it keeps the public contract minimal, and it leaves the additive door open in the direction that is free.

## Decision

1. **`core.error_envelope` major 1 carries no `details` member, typed or untyped.** `field_violations: Vec<FieldViolation>` is the typed carrier for the one case a producer actually has — validation failure — with `FieldPath`'s `^(/[A-Za-z0-9_-]+)+$` and `SafeMessage`'s control-character ban making a leak structurally hard rather than merely forbidden.

2. **The scope of statement 1 is this contract and this major, and nothing wider.** This ADR does **not** rule that structured detail is "typed or it is not on the wire". That would contradict `ARCHITECTURE.md` S14, which provides for bounded `metadata`/`unknown` fields; `README.md`, which requires provider-specific raw data to remain opaque and lossless where the common model cannot represent it; and this repository's own `EventEnvelope.payload`, an untyped `serde_json::Map` shipped with a documented `[[contract.vague_field_waiver]]`.

3. **The sanctioned path, when a producer needs one.** A discriminated, bounded carrier — the `{kind, value}` shape `ARCHITECTURE.md` S6.1 already uses for `Block::Unknown`, and the shape `google.rpc.Status.details` uses so that a gateway can strip a detail *by type* at a trust boundary — on an `internal`-classified contract, registered with a `[[contract.vague_field_waiver]]` justification. Not a free blob, and not on the `public` error envelope. Discrimination is the security property: it makes leakage mechanically filterable instead of a review obligation.

4. **`extensions` stays, and is a preservation channel, not an authoring channel.** A producer never invents a key in it; every value a producer intends a consumer to read is a typed field in this repository. The map exists so a consumer built today re-emits a later producer's additive fields verbatim, which `README.md`, `DOMAIN.md` invariant 6 and ADR-0002 point 8 all require. The testable form is `extensions.is_empty()` on the envelopes a service **constructs**, never on the envelopes it **forwards** — `contracts.toml` lists `ratatoskr-platform` as both a producer and a consumer of this contract, and `OperationSnapshot.errors` re-emits an upstream envelope inside `platform.operation.progressed.v1`. A rule that forbade a non-empty forwarded `extensions` would forbid the behaviour the channel exists for.

5. **`Extensions::insert` stays public and is not hidden.** `Extensions` is `#[serde(transparent)]` and derives `Deserialize`, so `serde_json::from_value::<Extensions>` reopens anything an API change would close. The enforceable artefact is the stated rule plus the assertion, not the method's visibility.

6. **`ARCHITECTURE.md` S5.5 is corrected** to the shipped shape, with the absence of `details` and the sanctioned path stated in the same place. Leaving the stale Rust block would have re-seeded the question from the higher-ranked document.

## Consequences

- No code, no schema and no fixture byte changes for statements 1–5. The generated `ErrorEnvelope` description changes because its doc comment now points at this ADR.
- `AGENTS.md` gains the `extensions` rule as a stated, testable invariant instead of an unstated review habit.
- The `AGENTS.md` principle 7 obligation to separate provider diagnostics is satisfied today by *separation*, and today separation means "not on this contract". Statement 3 is how it is satisfied when a producer needs the third channel.
- Adding the internal diagnostic contract later is additive at the wire level and needs a changeset, not a major bump here.

## Security / privacy

`THREAT_MODEL.md` names provider-response leakage and names `extensions` as the residual. That entry is **extended, not replaced**: declaring the channel receive-side binds only the test suites of in-workspace services on the current major. It does not bind a producer on a newer major than a relay, an out-of-workspace producer, or a service that hand-rolls its own JSON. `ErrorEnvelope.extensions` is `#[serde(flatten)]` and `ErrorEnvelope` is embedded in `OperationSnapshot.errors`, so an upstream key fans out through `platform.operation.progressed.v1` and the SSE stream to a lower trust tier. The threat-model wording now says all of that.

The workspace draws the line at public-versus-restricted, not typed-versus-untyped: `github/AGENTS.md` permits provider errors "in restricted diagnostics after redaction, but user-facing errors use stable internal codes"; `social/x/AGENTS.md` asks to "preserve raw/provider evidence safely for diagnostics and migration"; `platform/AGENTS.md` says "keep provider diagnostics in authorized internal records", which presupposes such a record exists. Statement 3 is that line drawn in this repository's own vocabulary.

## Compatibility / migration

Nothing is published here, so there is nothing to migrate from this repository.

There **is** a named migration gap outside it, and it is the concrete reason statement 2 exists. The incumbent public contract this platform replaces already ships `details` to live clients: `legacy/ratatoskr/docs/openapi/mobile_api.yaml` declares `details: {type: object, additionalProperties: true, nullable: true}` on the public `ErrorObject`, and `legacy/ratatoskr/docs/reference/api-error-codes.md` shows it alongside `retry_after`. This repository's error envelope has neither. A client migration therefore has nowhere to map `error.details` and nowhere to map `error.retry_after`. That is a migration item for the platform changeset, not a reason to re-add a blob here — but it is exactly why "no `details` ever" was not the decision taken.

Re-adding an optional typed member later is backward-compatible under `AGENTS.md`. Removing `extensions`, or narrowing it, is breaking.

## Validation

- The error-envelope fixture suite: every valid fixture round-trips byte-exactly, and `field_violations`, `correlation_id`, `trace_id` are absent when empty.
- `fixtures/core/error-envelope/compat/old-consumer-new-producer/future-optional-field.json`: a newer producer's unmodelled key survives a parse and re-emits verbatim through `extensions`. This is the frozen proof that statement 4's forwarding clause is the behaviour under contract.
- L-1: `contracts.toml`'s vague-name lint over the committed catalogue, which is the mechanised form of driver 3.
- The secret/PII scan over every byte under `fixtures/`.

## Follow-up

**Escalate before the first release, as a separate decision:** `retryable: bool` is contradicted from three directions and is on the same free-today/breaking-later clock as the questions this ADR closes.

- This repository's own `INTERFACES.md` says "retry **class**"; `clients/export-agent/docs/INTERFACES.md` says "retry classification"; `platform/README.md` says "structured retryability".
- Every client-facing repository draws a three-way distinction the boolean collapses: retry automatically, prompt the user to act (re-auth), give up. `OperationStatus` is a closed enum, so "reauth-required" cannot be rescued as a status.
- `ErrorEnvelope` is embedded in `OperationSnapshot.errors` and delivered over SSE and `platform.operation.progressed.v1`, where the HTTP `Retry-After` header two clients name cannot reach — and where the legacy contract's `retry_after` has no successor.

Candidate shape: a closed `retry: transient | after_user_action | permanent`, plus a decision on where `retry_after` lives. This is a fourth question and needs its own ADR.
