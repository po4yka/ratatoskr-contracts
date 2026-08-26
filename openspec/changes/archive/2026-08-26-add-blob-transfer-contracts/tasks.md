# Tasks: add-blob-transfer-contracts

## 1. Scaffold and registry wiring

- [x] 1.1 Create `crates/blob-transfer-contracts` (manifest mirroring notification-contracts; `[lints] workspace = true`; deps: identifiers, serde, serde_json, schemars, thiserror, regex) plus root-manifest member/dependency lines. (configuration — cannot start from a failing test)
- [x] 1.2 Add the two `[[contract]]` blocks to `contracts.toml`, extend `[services].known` with `ratatoskr-mobile` and `ratatoskr-export-agent`, and add `expires_at` to `[lint].timestamp_property_names`. (configuration — cannot start from a failing test)
- [x] 1.3 Register the six roots in `tools/contractsc` (`Cargo.toml` dependency, sorted `root_types!` entries, count pins 20→26 in `tests/determinism.rs`, canonical() arms in `tests/fixtures.rs`, crate-count doc lines). (configuration — cannot start from a failing test)

## 2. Protocol semantics — reference state machine

- [x] 2.1 RED: `crates/blob-transfer-contracts/tests/state_machine.rs::chunk_plan_derives_count_and_lengths_totally` — asserts `ChunkPlan` derives chunk count and per-index lengths for boundary declarations (exact multiple, short last chunk) and refuses out-of-bounds declarations; must fail because no such type exists.
- [x] 2.2 GREEN: implement `src/chunks.rs` bounds and arithmetic so 2.1 passes. Verify: the single test exits 0.
- [x] 2.3 RED: `state_machine.rs::session_transitions_follow_the_protocol` — open→record→finalize(stored) path, out-of-range index refused without state change, premature finalize refused with the incomplete code leaving the session open; fails because the session model does not exist.
- [x] 2.4 GREEN: implement the session transition model (`src/session.rs`) so 2.3 passes. Verify: the two tests exit 0.
- [x] 2.5 RED: `state_machine.rs::resume_after_interruption_sends_only_missing_chunks` — record indices {0,1,3}, query status view listing exactly those, replay 1 idempotently, then complete via missing indices to a stored outcome; fails for want of the status view.
- [x] 2.6 GREEN: implement the status view and gap accounting so 2.5 passes. Verify: the three tests exit 0.
- [x] 2.7 RED: `state_machine.rs::digest_mismatch_is_a_truthful_terminal_outcome` — full recording then finalize with divergent streamed digest answers the mismatch variant carrying expected/computed hex and terminal-failed state; fails for want of the outcome type.
- [x] 2.8 GREEN: implement finalize verification and `UploadCompletionOutcome` so 2.7 passes. Verify: all four tests exit 0.
- [x] 2.9 RED: `state_machine.rs::identical_chunk_replay_is_idempotent_and_divergence_conflicts` — same-digest redelivery succeeds once more with unchanged state; different digest for a recorded index is refused as conflict while the original survives and the session stays usable; fails for want of conflict handling.
- [x] 2.10 GREEN: implement per-chunk idempotency/conflict so 2.9 passes. Verify: the five tests exit 0.

## 3. Wire layer

- [x] 3.1 RED: `tests/wire.rs::token_and_digest_grammars_reject_malformed_input` — deserializing a receipt with a token violating the published pattern and a declaration with an uppercase/non-hex digest both fail with the crate's parse errors; fails because the newtypes do not exist.
- [x] 3.2 GREEN: implement `UploadResumptionToken`, `Sha256Hex`, `MediaType`, `DigestAlgorithm` with manual `JsonSchema` grammars so 3.1 passes. Verify: the test exits 0.
- [x] 3.3 RED: `tests/wire.rs::every_message_shape_round_trips_and_preserves_unknown_fields` — each of the six roots round-trips through the wire form and an injected unknown sibling field survives re-serialization on at least one root; fails for want of the payload types.
- [x] 3.4 GREEN: implement the six payload types (checked-intermediate `Deserialize` where cross-field rules apply) so 3.3 passes. Verify: both test files exit 0.

## 4. Fixtures and generated artifacts

- [x] 4.1 Author valid golden fixtures for every message shape under `fixtures/transfer/upload-request/valid/` and `fixtures/transfer/upload-receipt/valid/` (minimal + full + one per variant), and invalid fixtures with `invalid-expectations.toml` rows quoting the rejecting layer's message verbatim. (fixtures are data — cannot start from a failing test; validated by the gate)
- [x] 4.2 Run `cargo contracts generate` and commit regenerated schemas and TypeScript; bless `compat/api/ratatoskr-blob-transfer-contracts.txt`. Verify: `cargo contracts check` reports no findings.

## 5. Documentation

- [x] 5.1 Update README status/tree/models/event-examples, DEVELOPMENT present-list, AGENTS crate enumeration (nine→ten), ARCHITECTURE S5.8 (family section incl. canonical HTTP binding), TESTING crate-count line. (documentation — cannot start from a failing test)

## 6. Gate and delivery

- [x] 6.1 Run the full gate block from DEVELOPMENT.md in order; every command green including `cargo contracts check-typescript`.
