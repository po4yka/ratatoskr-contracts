## 1. Canonical command envelope

- [x] 1.1 Add a failing `crates/event-envelope/tests/command_serde.rs::typed_command_round_trips`
  covering envelope-to-payload binding, command identity, and a mismatched type; run it and verify
  failure is due to the missing command envelope types.
- [x] 1.2 Add `CommandId`, `CommandType`, `CommandEnvelope`, and `CommandPayload` until 1.1 passes;
  verify schema derivation preserves the canonical envelope shape and unknown additive fields.

## 2. Social capture command

- [x] 2.1 Add a failing `crates/social-contracts/tests/browser_capture_command.rs::round_trips_closed_provenance`
  covering all three providers, canonical command binding, and absence of credential/page fields;
  run it and verify failure is due to the missing social command type.
- [x] 2.2 Implement the social command and its closed provider vocabulary until 2.1 passes; verify
  Rust serialization and schema derivation preserve the exact request shape.

## 3. Closed social outcome taxonomy

- [x] 3.1 Add a failing `crates/social-contracts/tests/social_capture_outcomes.rs::rejects_unknown_social_outcome`
  proving the three documented social outcome spellings map to `ErrorCode` while an unknown social
  spelling is refused; run it before implementation.
- [x] 3.2 Add `SocialCaptureOutcomeCode` and its `ErrorCode` mapping until 3.1 passes; verify a
  partial operation report distinguishes `social.post` from its linked-article warning.

## 4. Command registry semantics

- [x] 4.1 Add a failing `tools/contractsc/tests/metadata_commands.rs::rejects_mismatched_command_type`
  using a synthetic metadata declaration; run it and verify failure is due to absent `commands`
  metadata support.
- [x] 4.2 Implement first-class `commands` metadata and registry validation until 4.1 passes;
  verify the social command's declared type matches `CommandPayload::COMMAND_TYPE`.

## 5. Generated artifacts and gate

- [x] 5.1 Add valid and invalid command fixtures plus registry metadata; generated artifacts cannot
  start from a failing unit test, so verify `cargo contracts check` detects them before generation.
- [x] 5.2 Generate schemas and TypeScript declarations, run `build-gate -- cargo contracts check`,
  `build-gate -- cargo test --workspace --locked`, and the remaining DEVELOPMENT.md gate commands;
  verify no generated drift remains.
