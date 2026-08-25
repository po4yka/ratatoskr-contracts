# Proposal: fixture, compatibility, and package CI

## Why

The seven contract crates are consumed across the Ratatoskr workspace, so a breaking public-API or schema change must be caught in this repository before it lands, not discovered downstream by sixteen consumers. The existing gate (`ci.yml`) validates schemas and fixtures but nothing watches the Rust public API surface, nothing proves that a clean checkout regenerates byte-identical output, and no packaged TypeScript artifact exists for consumers to trial before milestone 10 publishing. This is implementation plan milestone 9.

## What Changes

- Add a committed **public-API baseline** per contract crate under `compat/api/`, produced by `cargo-public-api`.
- Add two verbs to `contractsc`: `api-write` (regenerate baselines from current sources) and `api-check` (compare regenerated output against committed baselines, classify every difference as breaking (removed or changed public item) or additive (new public item), exit non-zero when any difference exists). The comparison logic carries its own unit tests.
- Add `.github/workflows/contracts.yml` with three jobs, in the style of `ci.yml`:
  - **compatibility** — run `cargo contracts api-check`; any uncommitted-blessed public-API difference fails the run.
  - **determinism** — run `cargo contracts generate` on a fresh checkout and require the tree to remain unchanged, proving the write path reproduces committed bytes end to end.
  - **package** — build a tarball of `generated/typescript/` and upload it as a workflow artifact.
- Document that fixture validation against current types remains the gate's job (`cargo contracts check` + `cargo test --workspace`) and is therefore enforced on every push and pull request; this change adds no duplicate fixture job.
- Update `DEVELOPMENT.md`, `README.md`, `docs/TESTING.md` and the implementation plan so the documents state what now exists.

Out of scope: publishing/tagging (milestone 10), consumer-repo integration, npm packaging decisions beyond the tarball artifact.

## Capabilities

### New Capabilities

- `contract-assurance-ci`: the CI-visible assurance behaviours of this repository beyond the gate — public-API compatibility checking against a frozen baseline, deterministic regeneration of generated artifacts, fixture validation coverage, and packaging of the TypeScript generation output as an artifact.

### Modified Capabilities

None. The two existing specs (`ai-archive-contracts`, `social-source-contracts`) describe wire shapes and are unaffected.

## Impact

- `tools/contractsc/src/` gains a module with two clap verbs and pure comparison logic; no dependency additions are planned (snapshot production shells out to the installed `cargo-public-api` binary).
- New committed tree `compat/api/*.txt` (seven files, one per contract crate).
- New workflow file `.github/workflows/contracts.yml`; `ci.yml` and the documented gate list stay untouched, which keeps their mechanical one-list assertion valid.
- Docs: `DEVELOPMENT.md`, `README.md`, `docs/TESTING.md`, `docs/IMPLEMENTATION_PLAN.md`.
