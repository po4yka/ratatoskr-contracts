# Tasks: define-block-kind-extension-procedure

## 1. Evidence fixture

- [x] 1.1 Add an INVALID fixture `fixtures/content/document/invalid/unknown-kind.json` (a document whose block carries `"kind": "table"`), run `cargo contracts check`, and confirm it FAILS with an undeclared-invalid-fixture error — this is the failing test; it names the exact gap the next task closes. Verified: `cargo run --locked --quiet --package ratatoskr-contractsc --bin contractsc -- check` exits 1 with `fixtures/content/document/invalid/unknown-kind.json has no entry in fixtures/invalid-expectations.toml`.
- [x] 1.2 Declare the fixture in `fixtures/invalid-expectations.toml` with `rejected_by = ["json_schema", "serde"]` and `error_contains = 'unknown variant'`; verify `cargo contracts check` passes and `cargo test -p ratatoskr-document-contracts --locked` asserts both directions (rejected by listed layers, accepted by none omitted). Verified: check exits 0 (`current: every contract artifact matches its canonical Rust source`) and the crate test suite passes 1/1.

## 2. Procedure documents

- [x] 2.1 Write `docs/adr/0010-document-ir-block-kind-extension-procedure.md` covering proposer, acceptor, evidence list, unknown-kind rejection stance citing fixture 1.x, version-movement rules under binding development status, rollout order, knowledge pattern-site follow-up, and extractor S8.2 prose discrepancy; cannot start from a failing test — documentation, not executable behaviour.
- [x] 2.2 Add the ADR-0010 index entry to `docs/adr/README.md` under Accepted and update its Last reviewed date; documentation task, no test.
- [x] 2.3 Point ARCHITECTURE.md S6.1's intersection sentence and S6.2's provenance sentence at ADR-0010 without changing their meaning; documentation task, no test.

## 3. Canonical source pointer and regeneration

- [x] 3.1 Extend the `DocumentBlock` doc comment in `crates/document-contracts/src/document.rs` to cite ADR-0010 governance; doc-comment-only change, no behavioural test applies.
- [x] 3.2 Run `cargo contracts generate` and confirm the only diffs are the regenerated schema description bytes for `content.document` (JSON Schema and TypeScript); verify drift cleanliness afterwards with `cargo contracts check`. Verified: generate wrote only `schemas/json-schema/content/document.v1.schema.json` and `generated/typescript/json-schema/content/document.v1.ts` (description + source_digest lines), all other artifacts reported unchanged; check exits 0 with `current: every contract artifact matches its canonical Rust source`.

## 4. Gate

- [x] 4.1 Run the full gate block from DEVELOPMENT.md (`cargo fetch --locked`, `cargo deny check`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, the 850-line source-size check, `cargo contracts check`, `cargo test --workspace --locked`) and record each command's exit status. Recorded: fetch 0; deny 0 (`advisories ok, bans ok, licenses ok, sources ok`; one transient advisory-DB fetch error on first attempt, clean on retry); fmt 0; clippy 0; size-check 0; `contractsc check` 0 (`current: every contract artifact matches its canonical Rust source`); workspace tests 0 with 72 `test result: ok` suite summaries and 0 failures.
