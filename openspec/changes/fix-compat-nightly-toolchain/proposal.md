## Why

The `compatibility` CI job installs `nightly-2025-08-02`, but the workspace now requires Rust 1.97 and the checked-in public-API baselines were produced with `nightly-2026-06-12`. As a result, `cargo contracts api-check` cannot run in CI, so it does not provide compatibility evidence for the blob-transfer contract or any other crate.

## What Changes

- Pin the CI compatibility toolchain to `nightly-2026-06-12`, the compiler recorded by the committed API baselines.
- Invoke the public-API compatibility command with that exact toolchain rather than the floating `nightly` alias.
- Update the developer command documentation to use the same pinned toolchain.

## Capabilities

No contract behaviour changes. This is CI and documentation maintenance only; `skip_specs: true` is set in the change manifest.

## Impact

- `.github/workflows/contracts.yml` compatibility job.
- `DEVELOPMENT.md` public-API compatibility instructions.
- No wire types, generated artifacts, fixtures, or consumer-facing protocol semantics.
