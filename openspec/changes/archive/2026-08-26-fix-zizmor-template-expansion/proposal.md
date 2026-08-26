## Why

The hosted `zizmor` security check fails on the package job because GitHub expression syntax is interpolated directly into a shell script. The runtime value is a commit SHA, but putting any template expansion in `run:` creates an avoidable code-injection sink and correctly blocks the security gate. Separately, the current `main` branch registers a twenty-eighth generated root while determinism tests still pin the prior 27/54 totals, so the documented CI gate fails before it can validate the repository.

## What Changes

- Build the TypeScript artifact filename from GitHub Actions' existing `GITHUB_SHA` shell environment variable.
- Keep the artifact's name, contents, and upload contract unchanged.
- Align determinism-test cardinality pins with the already committed 28 JSON Schema and 28 TypeScript roots.
- Restore the hand-written root registry's required Rust-path ordering and update its event-count pin for the already registered analysis-completion event.

## Capabilities

No contract behaviour changes. This is workflow security maintenance only; `skip_specs: true` is set in the change manifest.

## Impact

- `.github/workflows/contracts.yml` package job and `tools/contractsc/tests/determinism.rs` test pins.
- Hosted `zizmor` and documented CI gate; no wire contract, generated artifact, or dependency changes.
