## 1. Scaffolding (configuration - cannot start from a failing test)

- [x] 1.1 Create `crates/backup-contracts` skeleton: package `ratatoskr-backup-contracts` inheriting workspace fields/lints, deps (`ratatoskr-identifiers`, `ratatoskr-event-envelope`, serde, serde_json, schemars, regex, thiserror), lib.rs with crate docs and `mod policy;`/`mod events;`; stub public types with full field shapes, standard derive set, flattened `Extensions`, `impl EventPayload for PolicyAcknowledged`, permissive placeholder bodies for the succession and drift helpers (documented as such) so later tests fail at runtime rather than compile time. Add root-Cargo.toml member + workspace dependency. Verify: `cargo check -p ratatoskr-backup-contracts` exits 0.
- [x] 1.2 Wire discovery: add the dependency to `tools/contractsc/Cargo.toml`; add two sorted entries to `root_types!` in `tools/contractsc/src/registry.rs` and one insert to `event_payload_types()`; append `produced_at` to `[lint].timestamp_property_names` and `backup_policy` to `[entity_kinds].known`; add both `[[contract]]` blocks with `[[contract.field]]` governance for `produced_at`. Configuration task: no behaviour exists yet to test. Verify: `cargo contracts check` reports no metadata-rule or registry-set failures.
- [x] 1.3 Write valid fixtures `fixtures/backup/desired-backup-policy/valid/{full-policy,minimal-policy}.json` and `fixtures/events/vault.backup_policy.acknowledged.v1/valid/{accepted,rejected-stale-version}.json` (stale rejection stays representable at stub stage - regression pin). Generated files cannot start from a failing test. Verify: `cargo contracts generate` then `cargo contracts check` exits 0 with all four valid fixtures accepted.

## 2. Policy document behaviours

- [x] 2.1 RED: add invalid fixture `fixtures/backup/desired-backup-policy/invalid/policy-version-zero.json` with its `invalid-expectations.toml` row (`rejected_by = ["serde"]`) AND failing tests in `crates/backup-contracts/tests/policy.rs`: `succession_validation_rejects_zero` asserting `validate_policy_succession(0, 5)` is `Err`, `succession_validation_rejects_non_increase` asserting equal and lower candidates are `Err` against the permissive stubs. Verify: `cargo contracts check` fails naming the fixture's undeclared acceptance, and `cargo test -p ratatoskr-backup-contracts --test policy` fails exactly those assertions.
- [x] 2.2 GREEN: enforce `policy_version > 0` inside `Deserialize` following the crate's checked-intermediate pattern, and implement `validate_policy_succession` returning distinct errors for zero and non-increasing successions. Verify: both 2.1 commands now exit 0.
- [x] 2.3 RED: add invalid fixture `fixtures/backup/desired-backup-policy/invalid/duplicate-repository-entry.json` (`rejected_by = ["serde"]`). Verify: `cargo contracts check` fails because the duplicate still deserializes.
- [x] 2.4 GREEN: reject two entries naming the same repository reference during deserialization. Verify: `cargo contracts check` exits 0.
- [x] 2.5 Closed vocabularies pin: add invalid fixtures `unknown-mirror-cadence.json` and `unknown-priority-hint.json` (`rejected_by = ["json_schema", "serde"]`). Exemption: unknown-variant refusal falls out of the derived closed enums the moment they exist, so these are green on arrival - they pin the property regressively like golden output. Verify: `cargo contracts check` exits 0 with both rows enforced.
- [x] 2.6 Canonical-instant pin: add invalid fixture `produced-at-trailing-zeros.json` (`rejected_by = ["serde"]`). Exemption: canonicality is inherited from `WireTimestamp` in `ratatoskr-identifiers`, so the fixture documents inherited behaviour rather than driving new logic. Verify: `cargo contracts check` exits 0.
- [x] 2.7 RED: add invalid fixture `exclusion-expression-control-character.json` (`rejected_by = ["serde"]`). Verify: `cargo contracts check` fails while `ExclusionExpression` accepts control characters.
- [x] 2.8 GREEN: enforce the expression grammar (non-empty, max 256 bytes, no C0/DEL) in `ExclusionExpression` validation. Verify: `cargo contracts check` exits 0.

## 3. Acknowledgment event behaviours

- [x] 3.1 Envelope composition pins: create `crates/backup-contracts/tests/events.rs` with `event_type_constants_are_the_registered_names` (parses, segments, past-tense action), `accepted_acknowledgment_travels_inside_a_real_envelope` (`set_payload` binds type, canonical round trip, empty `dropped_field_pointers`, typed read back), and `mismatched_payload_read_is_refused`. Exemption: these exercise envelope mechanics that already hold once the payload type and `EventPayload` impl exist, so they are regression pins, not red-green pairs. Verify: `cargo test -p ratatoskr-backup-contracts --test events` exits 0.
- [x] 3.2 RED: add invalid fixtures `accepted-with-reasons.json`, `rejected-without-reasons.json`, `reason-reference-without-matching-code.json`, `repository-reason-missing-reference.json`, `acknowledged-version-zero.json`, `accepted-with-applied-ahead.json` (all `rejected_by = ["serde"]`) plus inline assertions in `tests/events.rs::self_contradicting_acknowledgments_are_refused` covering the same six cases. Verify: `cargo contracts check` fails on the six undeclared acceptances and the new test fails.
- [x] 3.3 GREEN: implement the acknowledgment checked deserialization: rejected implies at least one reason, accepted implies none, reason/reference pairing per code, `acknowledged_policy_version > 0`, and accepted implies strictly-later version than `last_applied_policy_version`. Verify: `cargo contracts check` and the events test binary exit 0.
- [x] 3.4 Unknown reason code pin: add invalid fixture `unknown-reason-code.json` (`rejected_by = ["json_schema", "serde"]`). Exemption: representation-inherent closed enum, pinned regressively. Verify: `cargo contracts check` exits 0.

## 4. Drift semantics as pure functions

- [x] 4.2 GREEN: implement both catalog-drift functions over borrowed slices. Verify: the two tests pass.
- [x] 4.3 RED: failing test `apply_exclusions_yields_a_subset_preserving_order` proving output is an order-preserving subset of the input under arbitrary matchers, red against a stub returning an empty vector. Verify: the test fails.
- [x] 4.4 GREEN: implement `apply_exclusions` taking the scoped values, the exclusions, and a caller-supplied match predicate. Verify: the test passes.

## 5. Generated artifacts, baselines, compatibility evidence

- [x] 5.1 Regenerate deterministically and freeze the API surface: run `cargo contracts generate`, then `cargo contracts api-write` to bless `compat/api/ratatoskr-backup-contracts.txt`, reviewing the diff. Generated/baseline files cannot start from a failing test. Verify: `cargo contracts check` exits 0; `cargo contracts api-check` exits 0.
- [x] 5.2 Record classification evidence: this family is additive by construction (no pre-existing schema to diff); capture `cargo contracts compat` output between the generated document/event schemas and themselves as the identity baseline and note producers/consumers in the change notes. Verification task over tooling output.

## 6. Documentation

- [x] 6.1 Update README contract tree (`backup-contracts (now)`), status paragraphs, a "Backup policies" principal-models section, and the event-name examples if needed. Documentation task: no runtime behaviour. Verify: rendered tree lists eight crates.
- [x] 6.2 Update DEVELOPMENT.md present-lists and docs (`ARCHITECTURE.md` structure tree plus a backup-policy section as S5.6 to avoid renumbering S9-S16, AGENTS.md crate enumeration, TESTING.md crate count). DATA_MODEL.md and IMPLEMENTATION_PLAN.md carry no family enumeration needing a change - verified by grep. Verify: grep finds no remaining "seven contract crates" claims outside archived history.

## 7. Gate

- [x] 7.1 Run the full gate block from DEVELOPMENT.md in order (`cargo fetch --locked`, `cargo deny check`, fmt check, clippy `-D warnings`, 850-line size check, `cargo contracts check`, `cargo test --workspace --locked`). Single verification task spanning all implementation groups. Verify: every command exits 0.
