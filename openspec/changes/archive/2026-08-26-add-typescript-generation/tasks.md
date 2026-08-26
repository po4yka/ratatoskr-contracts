# Tasks: add-typescript-generation

## 1. TypeScript emitter — parity, determinism, fail-closed projection

- [x] 1.1 Add failing tests to `tools/contractsc/tests/determinism.rs`: `generated_typescript_artifacts_mirror_the_schema_tree` (after `generate()` every root type has exactly one `.ts` under `generated/typescript/` mirroring its schema path, exports the final schema-id segment as root type, exports `$defs` sorted and referenced bare, contains no imports and no `any`), `generated_typescript_is_byte_deterministic` (two `generate()` runs byte-identical across the staged tree), `generated_typescript_contains_no_timestamps` (no timestamp-shaped content), and `unrepresentable_construct_aborts_generation` (staged metadata carrying a construct outside the supported subset makes `generate()` fail naming that schema identifier and leaves no `.ts` for it). Run `cargo test -p contractsc --test determinism` and confirm each new test fails on its stated assertion (missing files / unexpected output), not on a compile error.
- [x] 1.2 Implement the TypeScript emitter consuming the normalized JSON Schema values per design D1–D4 and wire it into `generate()`'s output set; rerun `cargo test -p contractsc --test determinism` until section 1 tests pass.

## 2. Provenance headers on TypeScript artifacts

- [x] 2.1 Add failing tests to `tools/contractsc/tests/provenance.rs`: `typescript_header_carries_required_members` (each `.ts` starts with a leading block comment holding the marker line first, all nine members, and no timestamp member) and `typescript_body_digest_detects_tampering` (the recorded source digest equals SHA-256 over the header-less body stripped through the closing comment delimiter, and modifying any body byte breaks the match). Run `cargo test -p contractsc --test provenance` and confirm both fail on their assertions.
- [x] 2.2 Render the provenance header as a leading block comment inside the emitter; rerun `cargo test -p contractsc --test provenance` until section 2 tests pass.

## 3. Drift check manages the TypeScript family

- [x] 3.1 Add failing tests to `tools/contractsc/tests/determinism.rs` beside the existing `check_detects_*` suite: `check_reports_missing_typescript_declaration`, `check_reports_stale_typescript_regeneration`, `check_reports_tampered_typescript_declaration`, and `check_reports_orphaned_typescript_file`, each staging the scenario and asserting the corresponding finding is reported and `check()` fails. Run `cargo test -p contractsc --test determinism` and confirm they fail because the findings never fire for `.ts` files today.
- [x] 3.2 Derive the expected `.ts` set from `contracts.toml` outputs via the documented path derivation, classify missing/stale/tampered through the existing finding machinery, and extend the orphan sweep to `generated/typescript/**/*.ts`; rerun `cargo test -p contractsc --test determinism` until section 3 tests pass.

## 4. Compile verification verb

- [x] 4.1 Create `tools/contractsc/tests/check_typescript.rs` with an injected runner closure covering three cases — clean compilation exits zero, a compiler diagnostic is surfaced non-zero, and an unavailable compiler yields actionable guidance non-zero — plus the minimal verb entry point stubbed to return not-implemented so the assertions (not a compile error) are what fail. Run `cargo test -p contractsc --test check_typescript` and confirm all three fail on assertions.
- [x] 4.2 Implement `cargo contracts check-typescript`: materialize current `generate()` TypeScript output into a temporary strict-mode project (`noEmit`, strict `tsconfig.json`), resolve `tsc` via the environment override then local fallback with an install-or-override message when neither resolves, keep the process spawn behind the injectable runner; rerun `cargo test -p contractsc --test check_typescript` until section 4 tests pass.

## 5. Regenerated artifacts and documentation

Cannot start from a failing test: this group is generated files and prose only.

- [x] 5.1 Run `cargo contracts generate` at the repository root and land the regenerated tree — committed `schemas/` byte-identical plus the new `generated/typescript/` directory — verifying `cargo contracts check` exits zero against the committed state.
- [x] 5.2 Update the editing-loop sections of `DEVELOPMENT.md` (regeneration workflow including `cargo contracts check-typescript`, ownership of `generated/typescript/`), the `README.md` status line, and the `AGENTS.md` phase notes; the gate command list and `.github/workflows/ci.yml` stay untouched. Verify every documented command runs as written.

## 6. Repository gate

- [x] 6.1 Run the full read-only gate block from `DEVELOPMENT.md` at the repository root — dependency policy, formatting, Clippy, source-size limits, `cargo contracts check`, and `cargo test --workspace --locked` — and confirm every step exits zero.
