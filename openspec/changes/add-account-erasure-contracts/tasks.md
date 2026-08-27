## 1. Account-erasure wire contracts

- [x] 1.1 Add `crates/operation-contracts/tests/account_erasure.rs` with the
  compiling test `account_erasure_command_and_acknowledgement_are_redacted_and_owner_scoped`;
  it MUST fail because the expected generated command and acknowledgement
  schemas do not exist, while asserting their required operation/owner scope
  and forbidden secret-bearing members.
- [x] 1.2 Add the canonical `AccountErasureRequested`,
  `AccountErasureAcknowledged`, and closed terminal-outcome types; register
  them in `contracts.toml` and the generator registry, generate artifacts, and
  verify `cargo nextest run --locked -p ratatoskr-operation-contracts --test account_erasure`
  passes.
- [x] 1.3 Extend `crates/operation-contracts/tests/account_erasure.rs` with
  the compiling test `acknowledgement_preserves_unknown_additive_members`;
  it MUST fail because the forward-compatibility fixture is absent.
- [x] 1.4 Add valid, invalid, and two-direction compatibility fixtures for the
  typed command and acknowledgement, register their validation expectations,
  regenerate artifacts, and verify the new test and `cargo contracts check`
  pass.

## 2. Contract verification

- [x] 2.1 Run `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets --locked -- -D warnings`, `cargo contracts check`, and
  `cargo test --workspace --locked`; inspect generated output and public API
  compatibility before recording their observed results.
- [x] 2.2 Run `openspec validate add-account-erasure-contracts --strict` after
  every task is complete; archive only after the workspace change records the
  compatible service rollout.
