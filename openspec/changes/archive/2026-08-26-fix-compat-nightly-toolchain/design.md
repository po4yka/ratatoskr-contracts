## Context

See [proposal.md](proposal.md). `cargo-public-api` needs rustdoc JSON from a nightly compiler. The public-API baseline headers record `cargo-public-api 0.52.0` over `rustc 1.98.0-nightly (b30f3df3b 2026-06-11)`, supplied by `nightly-2026-06-12`. The current CI pin predates the workspace `rust-version = 1.97` and therefore cannot run the compatibility checker.

## Goals / Non-Goals

**Goals:**

- Give the compatibility job one explicit, reproducible nightly that can compile the workspace and reproduce the committed API baselines.
- Keep the local documented invocation identical to CI's compiler selection.

**Non-Goals:**

- Change any contract API or bless a changed compatibility baseline.
- Change the stable workspace toolchain or add a floating nightly alias.

## Decisions

Use `nightly-2026-06-12` for both installation and `RUSTUP_TOOLCHAIN` on `cargo contracts api-check`. This is the named toolchain corresponding to the provenance already committed in `compat/api/`; it avoids both the obsolete 2025 compiler and an ambient `nightly` channel selected by `cargo-public-api`.

Do not update `compat/api/`. A successful API check must compare against the existing baselines unchanged. Moving a pin first and regenerating baselines would hide an API surface change rather than repair the CI environment.

## Risks / Trade-offs

- [The pinned nightly becomes unavailable] → The job fails explicitly at toolchain installation; a reviewed pin and baseline update is then required.
- [The pin differs from baseline provenance] → Run `cargo contracts api-check` with `RUSTUP_TOOLCHAIN=nightly-2026-06-12`; it must leave every baseline unchanged.

## Migration Plan

Merge the workflow and documentation change, then observe the existing compatibility job pass with the unchanged baseline files. Revert the single commit to return to the previous CI configuration if necessary; no contract data or rollout state changes.
