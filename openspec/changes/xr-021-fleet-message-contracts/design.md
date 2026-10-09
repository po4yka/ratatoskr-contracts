## Context

See proposal.md and XR-021 CONTRACTS.md sections S01 to S13. The editing loop of `DEVELOPMENT.md` applies: change the canonical Rust type, then run `cargo contracts generate`, then bless the public API baselines.

## Decisions

- Layering: `operation-contracts` depends on `ai-archive-contracts`, so the receipt binding lives in `ai-archive-contracts` and `OperationReported::validate` lives in `operation-contracts`. `social-contracts` gains a dependency on `operation-contracts`; there is no cycle.
- The receipt binding is not a registered wire message: `contracts.toml` and `schemas/` stay untouched for it.
- Digest verification has one implementation: `AiConversation::compute_content_digest` hashes the canonical JSON of the messages array, exactly the rule already documented on the field. Fixtures that carry a stale digest are corrected, not exempted.
- Manifest canonical bytes are produced by rebuilding every object with keys inserted in lexicographic order and emitting compact JSON, so a `preserve_order` feature elsewhere in a consumer graph cannot change the bytes.
- `content.capture.requested.v1` payload is a closed pair of forms (url or blob). Serde validation and the generated schema both reject a payload with both or neither.
- Every file stays under 850 lines; `lib.rs` re-exports each new public module.

## Risks / Trade-offs

- A fixture whose digest was stale is rewritten; the new digest is computed by the same function that producers will call, so a stale fixture cannot hide a divergent implementation.
- The `captured_at` fixtures pin existing behaviour, so they cannot start from a failing test; this is recorded in the task list.

## Migration Plan

Additive only. Consumers pin the final commit of this series. Rollback is to keep the previous pin.
