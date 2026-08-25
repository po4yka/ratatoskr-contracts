# Contracts testing strategy

> Status: Proposed  
> Last reviewed: 2026-08-25

Required tests:

- Schema self-validation and valid/invalid fixtures.
- Backward/forward compatibility by family.
- Public-API compatibility against the frozen baselines under `compat/api/`.
- Deterministic generation and clean-tree checks.
- Rust/TypeScript compile and round-trip serialization.
- Unknown field/variant behavior.
- OpenAPI lint and generated-client compilation.
- Secret/PII fixture scanning.
- Workspace producer/consumer impact and version-range checks.

A test must demonstrate detection of a deliberate breaking change. Updating snapshots without explaining semantic change is prohibited. Release artifacts are rebuilt and compared with committed/generated expectations.

## Public-API compatibility

The nine contract crates are consumed across the workspace, so their exported Rust surface is watched the same way generated schemas are. `compat/api/<package>.txt` freezes each crate's public items, produced by `cargo-public-api`; `cargo contracts api-write` regenerates them and `cargo contracts api-check` diffs current sources against the committed files, failing on any difference — additions included, because consumers compile against everything a crate exports.

The suite is `tools/contractsc/tests/api_compat.rs`, tests A-1 to A-4: an identical snapshot classifies clean, a removed item classifies breaking by name, an added item classifies additive by name, and regenerating the baseline blesses an approved change without other edits. CI runs the real check per push in the `compatibility` job of `.github/workflows/contracts.yml`.

The same stated limit as the schema classifier applies: the comparison sees presence, absence and signature text, never a meaning change behind an unchanged signature. Review stays the guard.

## Test-first

A change is planned before it is built, and the plan is a task list in which behaviour arrives in
pairs: one task adds a failing test, the next makes it pass. `openspec/config.yaml` carries that
rule, which is what puts it into every planning and implementation request rather than only into this
document.

The loop:

1. Write the test the scenario names. Run it. Confirm it fails, and read the failure — a test that
   fails because it does not compile has proved nothing about the behaviour.
2. Write the smallest change that makes it pass. Run it again.
3. Refactor only once it is green, adding no test and changing no behaviour.

Two checks stand behind this, and neither of them can see the order:

- `openspec validate --archived`, in `.github/workflows/openspec.yml`, fails when a change was
  archived with a task left unticked.
- A step in `.github/workflows/fleet.yml` fails when this repository holds a manifest and a `ci.yml`
  that never runs a test.

`ratatoskr-workspace/docs/QUALITY_GATES.md` records why the order itself is not checkable, rather
than leaving the gap to be discovered.
