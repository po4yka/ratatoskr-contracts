# Proposal: add-backup-policy-contracts

## Why

The ratatoskr-github plan ("publish versioned desired backup policy to Vault") and the ratatoskr-vault plan ("desired-state target reconciliation") both depend on a desired-backup-policy contract that does not exist yet. Legacy git_backup_sync encoded what must be preserved implicitly inside job configuration, so no independent observer could audit what was supposed to be backed up, and two services that never share a database had no stable way to agree on it. Making WHAT should be preserved an explicit, versioned, auditable contract — with Vault answering through acknowledgment events — unblocks both plans and replaces implicit configuration with a reviewable wire agreement.

## What Changes

- Add a new contract crate `ratatoskr-backup-contracts` under `crates/backup-contracts`, owning one bounded-context family:
  - `DesiredBackupPolicy` v1 — the versioned policy document GitHub publishes: a monotonic `policy_version`, per-repository entries carrying a repository identity reference (existing identifier grammar, ADR-0007), a mirror cadence class, priority and size hints, explicit exclusions, and generation metadata (producing service, produced-at instant with documented authority).
  - `vault.backup_policy.acknowledged.v1` — the event payload Vault answers with, riding the canonical `EventEnvelope`: accepted/rejected outcome with stable, machine-actionable reason codes, and the last-applied policy version Vault has seen.
  - Drift semantics specified as typed rules expressed as pure functions (what an entry means when catalog state changes mid-version), testable without I/O.
- Register both contracts in `contracts.toml` with full governance metadata (owner, producers, consumers, classification, field authority/nullability/unit rows, privacy).
- Add valid, invalid, and compatibility fixtures under `fixtures/`, wired into `cargo contracts check` through `fixtures_dir` declarations and `fixtures/invalid-expectations.toml`.
- Regenerate JSON Schema and TypeScript artifacts deterministically (`cargo contracts generate`) and freeze the new crate's public-API baseline under `compat/api/`.
- Update repository documentation (README contract tree, DEVELOPMENT present-lists, docs) for the eighth contract family.

This is an additive change. No existing contract type, event, or fixture changes meaning; nothing is **BREAKING**.

## Capabilities

### New Capabilities

- `backup-policy`: the versioned desired-backup-policy document GitHub publishes to Vault, the acknowledgment events Vault returns over the canonical envelope, and the drift rules that decide how policy versions apply when repository catalog state changes between versions.

### Modified Capabilities

None. No existing capability's requirements change.

## Impact

- Workspace: new member `crates/backup-contracts`; new entries in root `Cargo.toml` (`members`, `workspace.dependencies`).
- `contracts.toml`: two new `[[contract]]` blocks (document family `backup`; event family `events`); possibly one timestamp-name addition to the `[lint]` vocabulary and, if the design introduces a dedicated identifier kind, an `[entity_kinds]` addition — decisions recorded in design.md.
- Generated trees: `schemas/json-schema/backup/desired-backup-policy.v1.schema.json`, `schemas/events/vault.backup_policy.acknowledged.v1.schema.json`, and their TypeScript mirrors — produced only by `cargo contracts generate`.
- Fixtures: `fixtures/backup/desired-backup-policy/` and `fixtures/events/vault.backup_policy.acknowledged.v1/` plus rows in `fixtures/invalid-expectations.toml`.
- Compatibility: new frozen baseline `compat/api/ratatoskr-backup-contracts.txt` blessed via `cargo contracts api-write`; `cargo contracts compat` evidence collected for the new schemas.
- Producers/consumers: `ratatoskr-github` produces the policy document and consumes acknowledgments; `ratatoskr-vault` consumes the policy and produces acknowledgments. Rollout ordering across repositories is recorded in the cross-repository changeset, not invented here.
