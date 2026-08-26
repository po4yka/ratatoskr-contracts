# Design: add-social-source-removed-event

## Context

The published snapshot requires `author` and offers no removal fact; see the proposal for the motivation. The crate's wire conventions (state-carried payloads, closed vocabularies, extensions flatten, absent-not-null optionals) and the five registration points constrain every choice below.

## Goals / Non-Goals

Goals: honest representation of library removals and authorless captures, minimal payload surface, zero change to existing payload bytes.

Non-Goals: tombstone storage or deletion workflows in consuming services; representing upstream deletion (`deleted_upstream` already covers that fact); any new snapshot field beyond the author relaxation.

## Decisions

### D1: Removal is a standalone minimal payload, not a snapshot carrier

`SocialSourceRemoved` carries only `social_source_id`, `owner`, `reason`, `removed_at`, and the extension map. Re-sending the whole snapshot would imply the removed record is still indexable content and tempt consumers to resurrect it from redeliveries.

Alternative rejected: reusing `SocialSourceUpdated` with a sentinel availability value — overloads a closed vocabulary with a different fact.

### D2: Removal reason is a two-value closed enum

`RemovalReason::UserRequested | RetentionPolicy`. Both name why the library let go; neither claims anything about the provider. Unknown values are refused at parse like every other social vocabulary. Adding reasons later is additive.

Alternative rejected: free-form reason string — unbounded consumer behaviour on an unvalidated token.

### D3: Absent author is `Option<SocialAuthor>` with absent-not-null serialization

Follows the established optional-field convention (`#[serde(default, skip_serializing_if = "Option::is_none")]`). The hand-written wire mirror gains a `#[serde(default)] Option` field so old payloads (author present) parse unchanged and new payloads (author absent) parse to `None`; lossless re-emission keeps absence absent. The drift-guard test in `snapshot_roundtrip.rs` continues to fail if public struct and mirror diverge.

Alternatives rejected: a sentinel `EntityLocalId` (dishonest — it is not a provider id) and an `unknown: bool` side flag (two sources of truth for one fact).

### D4: Compat fixtures encode both directions explicitly

Snapshot family gains an author-absent fixture under `old-consumer-new-producer/` (today's updated type must accept and re-emit what a future producer may send); the existing minimal-first-day fixture keeps its present author and stays valid under `new-consumer-old-producer/`. The new event family gets non-empty sets in both directions per the compat harness's empty-dir failure rule.

## Risks / Trade-offs

[Producers may now omit author where they previously could not] → rollout order consumers-first is stated in the proposal; the compat fixture pins the tolerance so it cannot regress silently.

[Removal facts replayed after re-capture] → consumers deduplicate on `event_id`; a later `social.source.captured.v1` with a fresh identity supersedes by being newer evidence, which matches the contract's state-carried model.

[Blessed API baseline hides unintended diffs] → baseline move happens in its own commit step with `git diff` review of `compat/api/ratatoskr-social-contracts.txt`.

## Migration Plan

No deployment coordination inside this repository. Consumers upgrade first; producers adopt omission afterwards. Rollback is reverting producer usage; nothing here removes existing parse capability.

## Open Questions

None.
