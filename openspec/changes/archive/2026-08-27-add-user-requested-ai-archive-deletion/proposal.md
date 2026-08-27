## Why

The canonical AI-archive tombstone reason vocabulary cannot currently encode an authorized owner privacy deletion, so a producer would have to lie about its evidence or invent an ungoverned event. Changeset `AIARCH-009` adds the missing additive reason to the existing v1 contract.

## What Changes

- Add `user_requested` to `AiArchiveTombstoneReason` without changing the event name, payload shape, or major version.
- Add valid and compatibility fixtures proving the new reason round-trips through an older-compatible consumer shape without dropping fields.
- Regenerate JSON Schema and TypeScript artifacts from the Rust canonical source and update compatibility surface evidence.
- Keep account-erasure commands unchanged; they coordinate tenant-wide work but do not replace subject-level AI-archive tombstones.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ai-archive-contracts`: Define `user_requested` as authoritative owner privacy-deletion evidence for archive, conversation, project, or artifact tombstones.

## Impact

- Producer: `ratatoskr-chatgpt`; consumer: `ratatoskr-knowledge`; future Claude Archive adoption is outside this change.
- Classification: additive v1 vocabulary expansion. Knowledge already deletes by subject independently of the reason value, but its updated pin and fixture gate must land before ChatGPT emits the new token.
- Privacy: payload remains metadata-only and retains the existing immutable evidence reference; no content, credential, filename, or provider response is added.
- Rollback is a revert before producer enablement. After publication, the contract and Knowledge consumer remain able to replay the fact even if ChatGPT stops producing new ones.
