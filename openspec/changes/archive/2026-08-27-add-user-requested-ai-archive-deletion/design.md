## Context

See `proposal.md` for motivation. `AiArchiveTombstoneReason` is a bounded wire string newtype generated into JSON Schema and TypeScript. The payload already has the correct subject, owner, and evidence fields; only the authority vocabulary is incomplete.

## Goals / Non-Goals

**Goals:**

- Add one source-compatible reason token to the existing v1 event.
- Keep Rust, schema, TypeScript, fixtures, registry metadata, and compatibility surfaces deterministic.

**Non-Goals:**

- Add service deletion logic, account-erasure coordination, another event, or a contract major.
- Put audit content or user data into fixtures.

## Decisions

### 1. Extend the canonical Rust token declaration

Add `user_requested` to the newtype pattern and examples, then regenerate all derived artifacts. Editing generated schemas directly is rejected because Rust is canonical.

### 2. Prove both positive and negative vocabulary behavior

A valid subject tombstone fixture and round-trip test cover the new token. Existing fixtures prove no regression, and the existing invalid-reason layer continues to reject undeclared tokens.

### 3. Classify as additive with guarded rollout

The payload and event identifiers do not change. Knowledge uses the reason as validated evidence metadata rather than an exhaustive Rust enum branch, and its pin/test lands before the producer.

## Risks / Trade-offs

- [Generated surfaces drift] → run the read-only contract gate after deterministic generation and inspect all generated diffs.
- [A hidden consumer assumes three reasons] → keep producer disabled until the known Knowledge consumer passes the new fixture; record the boundary in `AIARCH-009`.

## Migration Plan

Generate, gate, commit, merge, and push contracts first. Consumers then pin this exact commit. If no producer has emitted the token, revert normally; after emission, retain parsing support even if producers stop emitting it.
