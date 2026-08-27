## 1. User-requested tombstone vocabulary

- [x] 1.1 RED — add `user_requested_tombstone_roundtrips_authoritative_evidence` in `crates/ai-archive-contracts/tests/events.rs` plus its safe fixture; run `build-gate -- cargo test --locked -p ratatoskr-ai-archive-contracts user_requested_tombstone_roundtrips_authoritative_evidence` and confirm the assertion fails because the current reason validator rejects `user_requested`
- [x] 1.2 GREEN — extend the canonical `AiArchiveTombstoneReason` vocabulary and examples, regenerate contract artifacts, rerun the targeted test, and verify existing provider/compliance/reconciliation fixtures remain byte-compatible

## 2. Contract evidence and publication

- [x] 2.1 Generated artifacts cannot start from a separate failing test because they are deterministic outputs of task 1.2; inspect the Rust, JSON Schema, TypeScript, registry, fixture, and compatibility diffs and verify no payload field, event name, major version, or private value changed
- [x] 2.2 Run the full fenced gate from `DEVELOPMENT.md` through `build-gate` where compiler-backed, run `openspec validate --all --strict`, archive this change with its spec synced, rerun `openspec validate --archived`, then commit, integrate into `main`, push, and record the published contract commit for downstream pins
