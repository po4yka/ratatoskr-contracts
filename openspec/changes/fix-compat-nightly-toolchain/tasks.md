## 1. Reproduce the compatibility failure

- [x] 1.1 Observe the `compatibility` job fail before the change because `nightly-2025-08-02` cannot satisfy `rust-version = 1.97`; this is CI configuration, so the existing job is the failing test rather than a new unit test. Observed in `contracts` run 32983169730: `nightly-x86_64-unknown-linux-gnu` was missing after the obsolete dated toolchain was installed.

## 2. Pin the checker environment

- [x] 2.1 Update the compatibility workflow and developer instructions to install and select the baseline's `nightly-2026-06-12`; verified the diff changes no contract or baseline artifact.

## 3. Verify the repair

- [x] 3.1 Run `RUSTUP_TOOLCHAIN=nightly-2026-06-12 cargo contracts api-check` through `build-gate` and verify every baseline is unchanged.
- [ ] 3.2 Run the full documented local gate and observe the hosted compatibility workflow pass after integration.
