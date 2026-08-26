# Design: add-typescript-generation

## Context

`tools/contractsc` already renders every contract root type into a normalized JSON Schema value, serializes it deterministically, stamps a nine-member provenance header (marker, canonical source, contract id and major version, generator and versions, source digest, validation note — no timestamps), and writes the result under `schemas/` with byte-for-byte reproducibility enforced by golden tests. `cargo contracts check` classifies each expected artifact as present/stale/tampered and sweeps for orphaned files. The TypeScript emitter plugs into this pipeline rather than introducing a second projection path. See proposal.md - Why for motivation and the specs delta for the normative behavior.

## Goals / Non-Goals

**Goals:**

- One deterministic Rust-to-TypeScript emitter consuming the same normalized JSON Schema value the JSON Schema renderer consumes, so there is a single projection pipeline.
- A committed `generated/typescript/` tree mirroring `schemas/` layout one-to-one, managed end-to-end by `generate` and `check`.
- Provenance headers on TypeScript artifacts equal in rigor to the JSON Schema family.
- An editing-loop compile verifier (`check-typescript`) usable locally without touching CI wiring.

**Non-Goals:**

- Runtime validators, zod-style schemas, or serialization helpers in the emitted TypeScript.
- npm packaging, publication, or consumer-repository integration (milestones 9–10).
- CI changes: the gate command list and `ci.yml` stay as they are; `cargo contracts check` simply gains authority over the new family.
- OpenAPI or Kotlin/Swift generation.

## Decisions

### D1. Output paths derive from the existing `output` field

Each root type's `.ts` path is derived mechanically from its `[[contract.root_type]] output` value: replace the leading `schemas/` segment with `generated/typescript/` and the trailing `.schema.json` suffix with `.ts`.

- *Why:* `contracts.toml` remains the single naming authority; two authored path columns could disagree, and derivation keeps mirror-parity provable.
- *Alternative considered:* adding an explicit `output_ts` authoring field. Rejected: duplicates naming authority and invites divergence between the families.

### D2. The emitter consumes the normalized JSON Schema value, not raw schemars output

Projection walks the same in-memory normalized schema value the JSON Schema renderer serializes.

- *Why:* one normalization step feeds all families; emitter behavior is defined against exactly the artifact consumers can inspect, and existing determinism machinery applies unchanged.
- *Alternative considered:* walking schemars structures directly. Rejected: creates a second projection path that can drift from what JSON Schema artifacts show.

### D3. Mapping rules for the supported subset

| Normalized construct | Emitted TypeScript |
|---|---|
| string / integer / number / boolean | `string` / `number` / `number` / `boolean` |
| null | `null` |
| array | `T[]` |
| object with properties | `interface` with required vs `?:` fields per `required` |
| enum | literal union of the enum values |
| const | the single literal type |
| oneOf / anyOf | union of branches |
| allOf | intersection of branches |
| `$ref` (`#/$defs/...`) | bare reference to the exported definition name |
| nullable array-form type containing `null` | union including `null` appended last |
| nullable via `anyOf` null branch | null branch stripped, `null` appended last to the remaining union |
| `additionalProperties: true` | index signature on the interface |
| `additionalProperties: false` or absent | closed interface |
| `description` | JSDoc comment on the member or declaration |
| `format` | appended to the JSDoc (e.g. `Format: uuid.`) |

Anything outside this subset aborts generation naming the contract's schema identifier (fail-closed) — the generator never emits an unsound approximation.

### D4. File layout is self-contained and export-complete

Each `.ts` file contains: the provenance block comment, the root type export named after the final schema-id segment, then all `$defs` exported as named types in sorted order, referenced bare. There are no imports across files, so every artifact compiles alone or alongside the rest.

- *Why:* mirrors how JSON Schema artifacts embed definitions; keeps per-file compilation meaningful and diffing simple.
- *Alternatives considered:* one bundled `index.d.ts` (breaks one-to-one mirror parity and orphan granularity); cross-file imports (couples artifacts and complicates consumers copying single files).

### D5. Provenance header renders as a leading block comment

Same nine members as JSON Schema, marker line first, rendered inside a `/* ... */` block at the top of the file. The `source_digest` is the SHA-256 over the header-less body, recomputable by stripping everything up to the closing comment delimiter. No timestamps.

- *Why:* reuses the provenance module and its test suite's expectations; keeps the two families auditable by the same rules.
- *Alternative considered:* line comments (`//`) per member. Rejected for no benefit over one block and a harder strip rule.

### D6. `check` manages the TypeScript family like the JSON Schema family

Expected-set computation derives from the mirrored paths (D1); findings reuse the existing missing/stale/orphan/tampered classification; the orphan sweep covers `generated/typescript/**/*.ts`. The gate command list does not change.

- *Why:* the spec requires identical treatment; reusing the finding machinery avoids a parallel code path.

### D7. `check-typescript` compiles a throwaway strict project

The verb materializes the current `generate()` TypeScript outputs into a temporary directory containing a minimal strict `tsconfig.json` (`noEmit`, `strict: true`) and invokes `tsc` over them. Compiler resolution order: `CONTRACTSC_TSC` environment override, then `npx --no-install tsc`; if neither resolves, exit non-zero with install/override guidance. The process-spawning step sits behind an injectable runner closure so tests cover success, diagnostic failure, and not-found without requiring `tsc` on the machine. Not added to the gate.

- *Why:* verifies compilability where it matters (the editing loop) without imposing a Node dependency on CI or contributors running the gate.
- *Alternatives considered:* putting compilation into the gate (adds a mandatory Node toolchain to every contributor and CI run — deferred to milestone 9 CI wiring); shipping our own pinned `tsc` binary (supply-chain and size cost unjustified).

## Risks / Trade-offs

- [JSON Schema is only a lower bound on validity] → the TypeScript projection inherits that looseness; mitigated by the fail-closed emitter (D3) and by documenting the mapping table so reviewers know exactly what was projected.
- [`npx` resolution varies across machines and offline setups] → `CONTRACTSC_TSC` explicit override plus `--no-install`; absence yields an actionable message rather than a confusing npx error (D7).
- [Emitter divergence from serde wire semantics] → impossible-by-construction is not claimed, but feeding the emitter the same normalized value rendered into committed JSON Schema (D2) means any divergence is visible as a mismatch between documented mapping and shipped schemas.
- [New committed tree doubles artifact review surface] → accepted: parity with `schemas/` is the point; diffs stay mechanical because output ordering and rendering are deterministic.
- [Orphan sweep may flag intentionally placed stray `.ts` files] → intended behavior; `generated/typescript/` is owned by the generator, and hand-placed files belong elsewhere.

## Migration Plan

Additive change. Land emitter, wiring, tests, and regenerated `generated/typescript/` tree in one PR; documentation updated in the same PR. Rollback is reverting the commit: the new tree is pure output and nothing consumes it yet. No wire contract semantics change, so no producer/consumer rollout ordering applies.

## Open Questions

None. Whether a local Node toolchain exists on the delivery machine affects only demonstration evidence, not the design, specs, or task breakdown.
