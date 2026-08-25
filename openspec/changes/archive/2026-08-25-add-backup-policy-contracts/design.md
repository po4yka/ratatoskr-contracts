# Design: add-backup-policy-contracts

## Context

The repository generates everything from Rust types declared in `contracts.toml` and compiled into `tools/contractsc/src/registry.rs`, which is a hand-written enumeration - the discovery mechanism, not source scanning. Adding a family therefore touches the crate itself, three manifests (workspace root, contractsc dependencies, registry), governance metadata, fixtures, generated trees, API baselines and docs. Existing conventions this design relies on: payloads derive `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema` with flattened `Extensions` and no `deny_unknown_fields`; identifiers and timestamps come exclusively from `ratatoskr-identifiers`; all tests live in integration binaries under `tests/`; event fixtures are bare payloads, not wrapped envelopes; cross-field invariants are enforced in `Deserialize` so the generated JSON Schema remains a lower bound (ADR-0001); closed enums reject unknown variants in both layers.

## Goals / Non-Goals

**Goals:**

- One new crate, two governed contracts, drift semantics as pure functions, full gate green including regenerated artifacts and a frozen API baseline.

**Non-Goals:**

- Retention/tombstone vocabulary, scheduler mechanics, concrete cadence intervals, git operations, ownership/tenant fields on the policy (the task scopes none of these; adding them would be speculative), publication mechanism of the document (transport is the services' choice).

## Decisions

### D1: New crate `ratatoskr-backup-contracts` at `crates/backup-contracts`

One bounded context (backup coordination between GitHub and Vault), one crate, matching the seven-crate precedent. `contractsc` derives API-baseline expectations from registry crate prefixes and expects package `ratatoskr-<short>` at `crates/<short>/`, fixing the directory name. Alternative considered - folding into operation-contracts - rejected: different owner pair (github/vault), different lifecycle, and mixing would couple unrelated review audiences.

### D2: Repository identity is `EntityRef`, kind token `repository`

No `RepositoryId` exists anywhere; the established grammar for pointing at a repository is `EntityRef("repository:<uuid>")` (already exercised by `fixtures/core/event-envelope/valid/unknown-aggregate-kind.json` and `[entity_kinds].known`). The local part is the Ratatoskr-owned repository-record UUID; provider coordinates stay in provider-specific systems, never derived into identity. Alternative - a new `RepositoryId` uuid newtype in identifiers - rejected for now: it changes a shared crate's public API and `EntityKind` match arms for zero wire benefit today; the upgrade path stays open because the wire form is identical.

### D3: Acknowledgment aggregates on `backup_policy:<version>`

The fact being acknowledged is "policy version N was answered". Kind token `backup_policy` joins `[entity_kinds].known` as fixture-governance widening (the wire enum stays open per ADR-0007); the local part is the decimal policy version, permitted by the existing `EntityLocalId` grammar. Alternative - aggregating on `user:<uuid>` - rejected: ownership is deliberately not modeled in v1 (see Non-Goals).

### D4: Closed vocabularies for cadence class, priority hint and rejection codes

`MirrorCadence` (`eager`, `daily`, `weekly` - coarse classes; interval mapping is Vault-scheduler territory) and `BackupPriorityHint` (`critical`, `standard`, `bulk`) are closed enums like `AcquisitionMethod`: an unrecognized value must stop processing because guessing would silently change backup frequency or ordering. Rejection reasons use a flat struct - `code` from a closed two-value set plus `repository_ref: Option<EntityRef>` - instead of a tagged enum, keeping the schema simple and the cross-field rule (reference present iff code demands it) enforceable serde-side, mirroring the `terminated_at`-exactly-when-terminal precedent.

### D5: Invariants live in `Deserialize`; schemas stay derived

Public types derive `Serialize` + `JsonSchema`; `Deserialize` goes through a private checked intermediate (`#[serde(try_from = "...")]`) validating: `policy_version > 0`; unique `repository_ref` per policy; `accepted` implies empty reasons and `last_applied < acknowledged`; `rejected` implies at least one reason; reason/reference pairing; exclusion expressions free of control characters. Every such rule becomes a `rejected_by = ["serde"]` fixture row - the ADR-0001 lower-bound consequence made auditable, same shape as `platform/operation-snapshot`.

### D6: Exclusions are scoped opaque expressions

`BackupExclusion { scope: RefsMatching | PathsMatching, expression }`. The expression is a crate-local validated string newtype (non-empty, max 256 bytes, no C0/DEL) whose interpretation - refname globs vs path patterns - belongs to the consuming mirror implementation. The contract fixes carrier safety and scope, not glob semantics; declaring a canonical glob grammar here would import git's matching rules into a language-neutral contract without a consumer that needs them yet.

### D7: Governance metadata additions

Two `[[contract]]` rows: `backup.desired_backup_policy` (family `backup`, owner/producers `ratatoskr-github`, consumers `ratatoskr-vault`, privacy `boundary_metadata`, `unknown_policy = "preserve"`) and `vault.backup_policy_acknowledged` (family `events`, owner `ratatoskr-vault`, producers `ratatoskr-vault`, consumers `ratatoskr-github`). `[lint].timestamp_property_names` gains `produced_at` (authority `producer_asserted`, unit `instant_rfc3339_utc_canonical`, note distinguishing it from publish/receive time); `[entity_kinds].known` gains `backup_policy`. Numeric fields already satisfy L2 (`policy_version`, `_bytes`). Compat fixture buckets are omitted, following the `content/document` precedent for a family's first version.

### D8: Registry and manifest wiring

`tools/contractsc/Cargo.toml` gains the crate dependency (this is what makes discovery compile-time); `registry.rs` gains two sorted `root_types!` entries plus one `event_payload_types()` insert for `PolicyAcknowledged`; workspace root gains member + dependency. Generated outputs follow R3 mechanically: `schemas/json-schema/backup/desired-backup-policy.v1.schema.json` and `schemas/events/vault.backup_policy.acknowledged.v1.schema.json` with TypeScript mirrors. Baseline `compat/api/ratatoskr-backup-contracts.txt` is blessed once via `cargo contracts api-write` after the API settles.

## Risks / Trade-offs

- [Cadence/priority vocabularies may miss real classes] -> Closed enums reject unknowns loudly, and adding a variant later is an additive, non-breaking expansion reviewed through the normal fixture path.
- [Aggregate token `backup_policy` is new fixture vocabulary] -> Widening `[entity_kinds].known` is a review-visible data change; the wire enum itself needed no change.
- [Drift helpers could creep into business logic] -> They are total functions over borrowed slices returning owned descriptions, no I/O, no state; they define contract semantics (what "covered" means), not workflows.
- [Schema/serde divergence confuses consumers] -> Every divergence is pinned by a named invalid fixture row stating which layer rejects and why.

## Migration Plan

Additive, single-version change under development status. Rollout: merge contracts first; consumers adopt by moving their git-dependency pin forward; nothing existing changes meaning, so rollback is reverting the pin. Cross-repository ordering (github publishes before vault reconciles; vault acks before github prunes) is recorded in the workspace changeset, not here.

## Open Questions

None.

## Compatibility Evidence

- The family is additive by construction: no pre-existing schema, event or fixture changed meaning. Classification for both new contracts is `internal`, lifecycle `proposed`.
- `cargo contracts compat <schema> <schema>` over each generated artifact against itself reports `compatible: no contract difference` (identity baseline; there is no predecessor to diff against).
- Producers and consumers: `ratatoskr-github` produces the policy document and consumes acknowledgments; `ratatoskr-vault` consumes the policy and produces acknowledgments. Rollout order (contracts first, then consumers move their dependency pins) is recorded in the cross-repository changeset, not here.
- `compat/api/ratatoskr-backup-contracts.txt` freezes the new crate's exported surface; `api-write` left all seven existing baselines byte-identical, which independently re-proves pinned-toolchain determinism.
- Registry-size pins updated where their own failure text demands it (`determinism.rs`: 19 roots / 38 artifacts / 19 TypeScript files; `metadata.rs`: 8 event contracts), plus canonical renderers for both new root types in `tools/contractsc/tests/fixtures.rs`.
