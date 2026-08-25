# Tasks

## 1. Public-API comparison core

- [x] 1.1 Add `tools/contractsc/tests/api_compat.rs` with the four tests the delta spec names (`check_passes_when_baseline_matches`, `removed_item_is_breaking`, `added_item_is_additive`, `write_then_check_round_trip`) driving the new `api` module through temporary directories, plus a minimal compiling stub of that module whose classifier reports "no differences"; run the suite and confirm the three negative-path tests fail on their own assertions (non-zero expected, zero received), which is the reason the task states.
- [x] 1.2 Implement the real comparison and classification in `tools/contractsc/src/api.rs`: strip comment lines, sort-compare line sets per crate, classify baseline-only lines as breaking and current-only lines as additive, report crate + items, exit non-zero on any difference; make all four tests pass with `cargo test -p ratatoskr-contractsc --test api_compat`.

## 2. Verbs and baselines

- [x] 2.1 Wire clap verbs `api-write` and `api-check` into the contractsc command surface; verify by hand that `cargo contracts api-check` errors cleanly when `compat/api/` is absent and that `--help` documents both verbs.
- [x] 2.2 Run `cargo contracts api-write` to produce `compat/api/<crate>.txt` for the seven contract crates with sorted lines and the provenance header, then verify `cargo contracts api-check` exits 0 and prints every crate as unchanged. (Cannot start from a failing test: generated files.)

## 3. Workflow

- [ ] 3.1 Add `.github/workflows/contracts.yml` with `compatibility`, `determinism` and `package` jobs copying ci.yml's trigger, permissions, concurrency, SHA pinning and checkout style, including the header comment stating that fixture validation stays the gate's job. (Cannot start from a failing test: workflow configuration.)
- [x] 3.2 Validate the workflow mechanically: YAML parse, every `uses:` pinned to a 40-hex commit, no unpinned container image, named jobs, and the packaging tarball built locally matches `git ls-files generated/typescript`.

## 4. Documentation

- [x] 4.1 Update DEVELOPMENT.md: the two new verbs and their local prerequisite, the compat/determinism/package jobs and why they sit outside the gate list, the milestone-status sentence, and the verified cargo-public-api install/toolchain guidance. (Cannot start from a failing test: documentation.)
- [x] 4.2 Update README.md status, docs/IMPLEMENTATION_PLAN.md item 9, and docs/TESTING.md compatibility section. (Cannot start from a failing test: documentation.)

## 5. Acceptance and gate

- [x] 5.1 Prove the failure paths: a temporary added pub type makes `cargo contracts api-check` exit non-zero as additive; a regenerated-baseline bless makes it pass again; a tampered fixture makes `cargo contracts check` exit non-zero (revert afterwards); a dirtying write after `generate` is caught by the same diff/status pair the determinism job runs. Revert every scratch change.
- [x] 5.2 Run the full gate block from DEVELOPMENT.md in this worktree until green, then commit the branch, merge into `main`, push `main`, and delete the worktree and branch.
