# Add deterministic TypeScript generation

## Why

Milestone 8 of `docs/IMPLEMENTATION_PLAN.md` requires deterministic Rust-to-TypeScript generation, and `docs/ARCHITECTURE.md` forbids independently maintained per-language definitions while requiring every generated artifact to carry provenance metadata and reproduce byte-for-byte. Today only JSON Schema is generated, so a TypeScript consumer must hand-write types that can silently drift from the canonical Rust sources.

## What Changes

- Add a TypeScript emitter to `tools/contractsc` that projects each contract root type into a `.ts` declaration file from the same normalized JSON Schema value the JSON Schema family is rendered from.
- Commit a second generated output tree, `generated/typescript/`, whose layout mirrors `schemas/` one-to-one (leading `schemas/` becomes `generated/typescript/`, suffix `.schema.json` becomes `.ts`).
- Render the provenance header as a leading block comment carrying the same nine provenance members as the JSON Schema artifacts, including the SHA-256 source digest, and never a timestamp.
- Extend `generate` to write both families and extend `check` so missing, stale, tampered, and orphaned TypeScript artifacts are detected exactly like their JSON Schema counterparts.
- Add a `cargo contracts check-typescript` verb that compiles the generated declarations in a scratch strict-mode `tsc` project when a TypeScript compiler is available, for editing-loop verification; repository CI wiring stays out of scope.
- Document the regeneration and compile commands in `DEVELOPMENT.md` and refresh the repository status notes that currently state generated TypeScript does not exist.
- Out of scope: runtime validators in the emitted TypeScript, npm packaging, publication, and client-repository integration (later milestones).

## Capabilities

### New Capabilities

- `typescript-generation`: The deterministic projection of canonical Rust contract types into TypeScript declarations under `generated/typescript/`, their provenance headers, and the drift, tamper, and orphan detection `cargo contracts check` applies to them.

### Modified Capabilities

None. The wire behaviour defined by `social-source-contracts` and `ai-archive-contracts` does not change; this change only adds a projected representation of it.

## Impact

- `tools/contractsc`: new emitter module, `lib.rs` API additions, `main.rs` verb wiring, new integration tests.
- New committed tree `generated/typescript/**`, produced by `cargo contracts generate`.
- Documentation: `DEVELOPMENT.md`, `README.md`, and the phase notes that claim TypeScript generation does not exist.
- Unchanged: `crates/*`, `contracts.toml`, the contents of `schemas/`, `fixtures/*`, `.github/workflows/ci.yml`, and the gate command list (`cargo contracts check` simply gains authority over the new family).
- Compatibility classification: additive. No wire contract semantics change; producers and consumers of existing JSON Schema artifacts are unaffected.
