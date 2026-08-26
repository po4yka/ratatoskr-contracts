# contract-assurance-ci Specification

## Purpose
Defines the assurance behaviours this repository must enforce beyond the documented gate so that a change visible to downstream consumers cannot land unnoticed: public-API compatibility against a frozen baseline, deterministic regeneration, continued fixture enforcement, and a packaged TypeScript artifact.

## Requirements

### Requirement: Public-API compatibility is checked against a frozen baseline

The repository SHALL carry a committed public-API baseline per contract crate under `compat/api/`, and SHALL provide a check that regenerates each crate's public-API text from current sources and compares it to the baseline. The check SHALL exit non-zero when any difference exists, SHALL classify removed or changed public items as breaking and newly added public items as additive, and SHALL name the affected crate and items. A deliberately updated baseline (produced by the provided regeneration verb) SHALL make the check pass again without code changes elsewhere.

#### Scenario: Identical baseline passes
- **WHEN** the committed baseline of every contract crate equals the public API regenerated from current sources
- **THEN** the compatibility check exits 0 and reports every crate as unchanged
- **Test**: `tools/contractsc/tests/api_compat.rs::check_passes_when_baseline_matches`

#### Scenario: Removed public item fails as breaking
- **WHEN** a public item present in the committed baseline is absent from, or changed in, the regenerated output of its crate
- **THEN** the compatibility check exits non-zero, classifies the crate as breaking, and names the removed or changed items
- **Test**: `tools/contractsc/tests/api_compat.rs::removed_item_is_breaking`

#### Scenario: Added public type fails as additive
- **WHEN** the regenerated output of a crate contains a public item absent from the committed baseline
- **THEN** the compatibility check exits non-zero, classifies the crate as additive, and names the added items
- **Test**: `tools/contractsc/tests/api_compat.rs::added_item_is_additive`

#### Scenario: Regeneration blesses an approved change
- **WHEN** an intentional public-API change is regenerated into the baselines with the provided write verb
- **THEN** the compatibility check exits 0 on the updated tree
- **Test**: `tools/contractsc/tests/api_compat.rs::write_then_check_round_trip`

### Requirement: Regeneration is verified deterministic in CI

CI SHALL run the generator on a fresh checkout and fail when the working tree changes in any way, covering modified tracked files and newly written untracked files alike.

#### Scenario: Clean checkout regenerates identical bytes
- **WHEN** `cargo contracts generate` runs on a clean checkout whose committed artifacts were produced by that generator
- **THEN** `git diff --exit-code` succeeds and `git status --porcelain` prints nothing
- **Test**: the determinism job in `.github/workflows/contracts.yml`; locally the same two commands after a generate run

### Requirement: Fixture validation remains enforced by the gate

Every push and pull request SHALL validate all committed fixtures against current types through the existing gate steps (`cargo contracts check`, which rejects a fixture not rejected for its declared reason, plus `cargo test --workspace --locked`, which asserts JSON-Schema-layer expectations); this repository SHALL NOT run a second, duplicate fixture job.

#### Scenario: Invalid fixture fails the workflow's gate job
- **WHEN** a committed fixture stops satisfying its declared expectation against current types
- **THEN** the gate job of `.github/workflows/ci.yml` fails because `cargo contracts check` exits non-zero
- **Test**: `tools/contractsc/tests/fixtures.rs` (existing) proves the rejection; the doc-sync step of `ci.yml` proves the gate list runs there

### Requirement: TypeScript output is packaged as a CI artifact

CI SHALL produce a tarball of `generated/typescript/` named for the source commit and upload it as a workflow artifact on pushes to `main` and on pull requests. Publishing the package to a registry is out of scope until milestone 10.

#### Scenario: Artifact contains the full TypeScript tree
- **WHEN** the packaging job runs
- **THEN** the uploaded archive contains every tracked `.ts` file under `generated/typescript/` at its original relative path
- **Test**: building the archive locally with the same tar invocation and asserting its listing matches `git ls-files generated/typescript`
