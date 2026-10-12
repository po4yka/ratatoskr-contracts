## Context

See proposal.md and XR-021 CONTRACTS.md R5-11 and R5-12. Both messages live in `ratatoskr-operation-contracts`, which every consumer already depends on, so no consumer gains a dependency. `schedule_registration.rs` is the model for a command with a typed payload; `account_erasure.rs` and the GitHub completion facts are the model for an event.

## Decisions

- `owner` is a `TenantRef` (`user:<uuid>`), so a malformed owner fails at deserialization. `acknowledged_event_id` is an `EventId`, so a non-UUID fails at deserialization. `subject` and `request_ref` are `EntityRef`; kinds are open tokens, and `privacy_request` needs no typed id.
- The enums are closed and `#[non_exhaustive]`, like `AccountErasureOutcome`: an unknown word is rejected by both the JSON Schema and serde.
- Cross-field rules live in `validate()` and also run inside `Deserialize` through a private wire struct, the pattern of `ChannelDigestRunRequested`. A payload that breaks a rule therefore never reaches a consumer, and the registry fixture check (which exercises `serde` only) can reject the invalid fixtures. Those fixtures are listed in `fixtures/invalid-expectations.toml` with `rejected_by = ["serde"]`, because a JSON Schema cannot express a rule that compares two members; a closed enum word is rejected by both layers. The sequence-list bounds that a schema can state (`minItems`, `maxItems`, `minimum`, `maximum`) are published in the schema as well.
- Stream sequences stay `Vec<u64>` and are exact in Rust. `validate()` additionally refuses a value above 2^53 - 1 with a distinct error, so a TypeScript or JavaScript consumer never reads a rounded sequence and then erases a different message. Real stream sequences are nowhere near that bound.
- The acknowledgement carries ids, a tenant reference and a subject reference, never content, because each of the five producers sees every acknowledgement. The erasure command can only name sequences, and Edge refuses any message the producer did not publish for the named owner; that rule is Platform's, documented here.

## Risks / Trade-offs

- A producer list that names a service outside the registry vocabulary would fail the registry lint; the spellings are copied from neighbouring rows.
- The API baseline needs the pinned nightly of `DEVELOPMENT.md`; it is blessed with the toolchain recorded in the existing baseline header.

## Migration Plan

Additive. Wave 2 repositories re-pin to the resulting commit; nothing is removed.
