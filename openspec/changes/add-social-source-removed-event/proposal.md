# Proposal: add-social-source-removed-event

## Why

The social-source contract can express that a source entered a user's library and that its record changed, but not that the user stopped holding it, and it cannot represent a capture that resolved without exposing any author account. Downstream consumers therefore cannot honour privacy deletions, and producers must either fabricate authorship or stay silent about truthfully preserved fallback records. Both gaps block `ratatoskr-instagram` plan item 5 (publish SocialSource and integrate with Knowledge).

## What Changes

- Add the `social.source.removed.v1` event payload (`SocialSourceRemoved`): the library identity, the owner, a closed-vocabulary removal reason (`user_requested` | `retention_policy`), and the producer-clock removal instant. A removal fact states that the user's library no longer holds the source; it says nothing about upstream state.
- Make `SocialSourceSnapshot.author` optional (`Option<SocialAuthor>`, absent-not-null on the wire). Absence means authorship is unknown — for example an unavailable capture whose preserved fallback never saw an author — and MUST NOT be read as "this post has no author".
- Register both edits through the usual five agreement points: crate exports, tool registry maps, `contracts.toml` blocks, generated JSON Schema + TypeScript pair, compat API baseline.

Both edits are backward-compatible expansion within major version 1: adding an event type, and relaxing validation on the read path (every payload that parses today still parses). The new producer freedom to omit `author` breaks consumers pinned to older crate revisions, so rollout order is consumers first. Development status forbids a v2, which settles the classification this policy tension would otherwise leave open.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `social-source-contracts`: two added requirements — one defining the removed fact and its closed reason vocabulary, one defining absent-author semantics and the lossless tolerance for authorless snapshots.

## Impact

- `crates/social-contracts`: new payload type + vocabulary enum + export; snapshot field type change with wire-mirror and builder updates.
- Registration: `tools/contractsc/src/registry.rs` root and event-payload maps; new `contracts.toml` contract block with generated schema/TypeScript pair.
- Fixtures: valid/invalid/compat sets for the new event family; an author-absent compat fixture under the snapshot family's old-consumer-new-producer direction.
- Compat API baseline moves (additive surface plus changed field type); blessed deliberately via `cargo contracts api-write`.
- Consumers: `ratatoskr-instagram` (planned publisher) gains the ability to propagate privacy deletions and to publish unavailable captures honestly; no existing consumer code exists yet.
