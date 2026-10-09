## Why

Changeset XR-021 fixes cross-repository integration defects in the Ratatoskr fleet. Fifteen repositories need typed wire contracts that do not exist yet: a content capture command, a document-extracted event, a Vault policy command, a schedule registration command, a digest manifest with canonical bytes, read-API views, social capture report builders, the AI-archive receipt binding and one shared content-digest implementation. This change lands every additive contract in one series of commits so each consumer pins a single contracts revision. The cross-repository behaviour is defined in the workspace changeset document "XR-021 CONTRACTS.md" (sections S01 to S13); this proposal cites those sections and restates nothing that is visible to more than one repository.

## What Changes

- Add `OperationReported::validate` (XR-021 CONTRACTS.md section S06 D3).
- Add the `ai_archive_contracts::platform_receipt` module with the loopback receipt binding constants, the capability document and the incomplete-import warning (S06 D1, D2, D3).
- Add `AiConversation::compute_content_digest` and `verify_content_digest`, call them from the conversation added and updated validators, document the tombstone and the erasure replay rule, and add the user-requested archive tombstone fixture (S07, S12).
- Add `channel_digest.manifest` with canonical bytes, validation and the HTTP binding constants, plus the read-API views (S08).
- Add the `platform.schedule_registration_requested` command (S08).
- Add `content.capture.requested.v1` (S11), `content.document.extracted.v1` and `vault.backup_policy.apply_requested.v1` (S09).
- Add the social capture report builders and the canonical `captured_at` fixtures (S10 CD1, CD4).
- Correct two registry rows and add a registry test for the live message table (S01).

Every item is additive: no existing wire shape, subject or version changes.

## Capabilities

### New Capabilities

- `fleet-message-contracts`: the typed messages, builders and validators listed above, as seen inside this repository.

### Modified Capabilities

<!-- None. -->

## Impact

Extends `ai-archive-contracts`, `operation-contracts`, `channel-digest-contracts`, `document-contracts`, `backup-contracts`, `social-contracts`, `contracts.toml`, `tools/contractsc/src/registry.rs`, generated schemas and TypeScript, fixtures and the blessed API baselines under `compat/api/`. `social-contracts` gains a dependency on `operation-contracts` and `channel-digest-contracts` gains `sha2`. Producers and consumers are named per message in S01. Rollback: consumers pin the previous contracts revision; nothing here alters data already on the wire.
