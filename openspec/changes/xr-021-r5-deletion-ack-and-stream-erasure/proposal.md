## Why

Changeset XR-021 round 5 (XR-021 CONTRACTS.md R5-11 and R5-12) fixes two privacy defects that need a registered wire message each. The events stream drops its oldest message at 7 days or 1 GiB, so a deletion tombstone that Knowledge has not consumed in time is lost silently while the producer has already marked it published (R5-11). Conversation, project and artifact facts carry message text and stay in the stream after the producer deletes the source, and no producer holds a stream-management permission to remove them (R5-12).

## What Changes

- Add `knowledge.deletion.acknowledged.v1` (event): Knowledge tells the producer of an `ai_archive.subject.tombstoned.v1` or a `social.source.removed.v1` that the deletion was applied or refused. Payload `KnowledgeDeletionAcknowledged` in `crates/operation-contracts/src/deletion_ack.rs`, with the closed enums `DeletionAckFamily`, `DeletionAckOutcome` and `DeletionAckRefusal`, and a `validate()` that requires `refusal` exactly when `outcome` is `refused`. It names no content.
- Add `platform.stream_erasure.requested.v1` (command): a producer asks Edge to erase messages it published from the events stream. Payload `PlatformStreamErasureRequested` in `crates/operation-contracts/src/stream_erasure.rs`, with a `validate()` that requires a `privacy_request` reference, 1 to 500 strictly ascending sequences, each at least 1 and at most 2^53 - 1 (the largest integer a JSON consumer reads exactly).
- Register both in `contracts.toml`, add fixtures, regenerate the JSON Schema and TypeScript, bless the `ratatoskr-operation-contracts` API baseline, extend `tools/contractsc/tests/live_message_types.rs` with the two S01 rows, and document both messages.

Both changes are additive: no existing message, field, schema or API line changes. Cross-repository behaviour is defined in XR-021 CONTRACTS.md R5-11 and R5-12, not here. The resulting commit is CONTRACTS_SHA_R5, the revision every Rust repository of the fleet moves to.

## Capabilities

### New Capabilities

- `deletion-ack`: the acknowledgement payload and its validation.
- `stream-erasure`: the erasure command payload and its validation.

### Modified Capabilities

<!-- None. -->

## Impact

Touches `crates/operation-contracts`, `contracts.toml`, `fixtures/`, `schemas/`, `generated/typescript/`, `compat/api/ratatoskr-operation-contracts.txt`, `tools/contractsc/tests/live_message_types.rs` and `docs/`. Every wave-2 repository of round 5 waits for the commit. Rollback: revert the commit before any repository pins it.
