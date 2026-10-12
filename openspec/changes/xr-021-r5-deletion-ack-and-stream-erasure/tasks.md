## 1. Deletion acknowledgement payload (XR-021 CONTRACTS.md R5-11 rule 1)

- [x] 1.1 Add the signature-only stub `crates/operation-contracts/src/deletion_ack.rs` (the types, `EVENT_TYPE`, a `validate()` returning `Ok(())`, the two `OperationContractError` variants) re-exported from `lib.rs`, and `crates/operation-contracts/tests/deletion_ack.rs`. `validate_requires_a_refusal_exactly_when_the_outcome_is_refused` and `deserialization_enforces_the_outcome_rule` MUST fail on their assertions (`Ok(())` where `Err(DeletionAckRefusedWithoutRefusal)` is expected; a refused payload with no refusal decoding); the round trip, enum words, extensions and bad-identity tests already pass against the stub.
- [x] 1.2 Implement `validate()` and run it from `Deserialize`; all six tests pass.

## 2. Stream erasure command payload (XR-021 CONTRACTS.md R5-12 rule 1)

- [x] 2.1 Add the signature-only stub `crates/operation-contracts/src/stream_erasure.rs` (the type, `COMMAND_TYPE`, the limits, a `validate()` returning `Ok(())`, the error variants) re-exported from `lib.rs`, and `crates/operation-contracts/tests/stream_erasure.rs`. The empty/501, zero/descending/duplicate, wrong-kind, above-2^53 and deserialization tests MUST fail on their assertions (`Ok(())` where an `Err` variant is expected); the round trip and the accept-1-and-500 test pass against the stub.
- [x] 2.2 Implement `validate()` and run it from `Deserialize`; all seven tests pass. Sequences above 2^53 - 1 are refused (decision in design.md).

## 3. Registry, schemas, TypeScript and fixtures

- [ ] 3.1 Add the two `[[contract]]` rows to `contracts.toml`, the `privacy_request` entity kind, the fixtures and one `fixtures/invalid-expectations.toml` entry per invalid fixture, and extend `tools/contractsc/tests/live_message_types.rs` with the two S01 rows. `cargo contracts check` and `cargo test -p ratatoskr-contractsc` MUST fail because the generated schemas, TypeScript and the API baseline do not exist yet.
- [ ] 3.2 Run `cargo contracts generate`, bless `compat/api/ratatoskr-operation-contracts.txt` with `cargo contracts api-write`, and commit the generated artifacts; `cargo contracts check`, `cargo contracts api-check` and `cargo test --workspace --locked` pass. Generated files cannot start from a failing test of their own.

## 4. Documentation

- [ ] 4.1 Document both messages under `docs/` in the style of `docs/CHANNEL_DIGEST_MANIFEST.md`. This cannot start from a failing test (documentation).

## 5. Final gate

- [ ] 5.1 Run the full gate of `DEVELOPMENT.md`, `cargo deny check` and `openspec validate --all --strict`; `git diff --stat ad16855..HEAD -- schemas generated compat` shows only additions. Verification only, no test to write.
