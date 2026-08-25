# Design: fixture, compatibility, and package CI

## Context

Milestone 9 asks for four assurances: fixtures validated in CI (already true through the gate), a public-API compatibility check against a frozen reference, a packaged TypeScript artifact, and a determinism proof for generated output. The two decisions recorded so far (ADR-0001 Rust-first canonical source, ADR-0002 event naming) are untouched. The development status rules hold: nothing here introduces a second major version, version negotiation, or a deprecation window; the baseline mechanism records what the single current version looks like and gates changes to it.

## Decisions

### D1. Frozen baseline files, not tag-based diffing

Nothing is published and no tag exists, so there is no last tagged release to diff against. The check compares against **committed baseline files** under `compat/api/<crate>.txt` instead. This is the frozen compatibility baseline DEVELOPMENT.md already names as missing. When milestone 10 tags the first release, the baseline source of truth can move to `git describe`; until then the committed file is the only honest reference. The comparison logic takes two directories, so swapping the reference producer later does not change the checker.

### D2. The verbs live in `contractsc`

`cargo contracts` is already the repository's gate surface, and it already owns change classification (`cargo contracts compat OLD.json NEW.json`). Two new verbs extend that family:

- **`api-write`** — for every contract crate declared in `contracts.toml`, run `cargo-public-api` against the crate manifest and rewrite `compat/api/<crate>.txt`. This is the writer verb; like `generate`, it repairs the tree in place.
- **`api-check`** — regenerate into a temporary directory and compare against the committed baselines. Exit non-zero when anything differs. Every difference is classified: a line present in the baseline but absent now is **breaking** (a public item was removed or changed); a line absent from the baseline but present now is **additive** (a public item appeared). Output names crate, classification, and items. Additive differences fail too: an added public type is exactly the acceptance case this milestone must catch, because consumers compile against the full exported surface even when the addition is semver-compatible.

The comparison itself is pure string processing over two sets of lines, so it is unit-tested without any external binary (tests feed synthetic before/after snapshots). Shelling out happens only where the real tool is needed: producing the current snapshot.

### D3. Snapshot production uses `cargo-public-api`, with provenance in the file

`cargo-public-api --short-text --manifest-path <crate>/Cargo.toml` produces compact, one-item-per-line text. Two properties are imposed by us rather than trusted from the tool: `api-write` **sorts** the item lines before writing (order independence is our invariant, not the tool's), and each baseline file begins with one **provenance comment** recording the `cargo-public-api` and toolchain versions that produced it, mirroring how every other generated artifact in this tree carries a provenance header. Both readers strip `#` comment lines before comparing. A tool-version mismatch therefore surfaces as a visible diff on the header line rather than as silent noise, and the remediation is the same explicit bless: rerun `api-write`, review, commit.

Consequence, stated plainly: bumping `cargo-public-api` or the toolchain can change the emitted text, and that PR must carry regenerated baselines. That cost is accepted deliberately — an unreviewed public-API change is the failure mode this milestone exists to prevent.

### D4. One new workflow file; the gate list stays untouched

`.github/workflows/contracts.yml` carries three jobs and copies `ci.yml`'s skeleton: same triggers, `permissions: contents: read`, the same concurrency block, `persist-credentials: false`, SHA-pinned actions only (zizmor audits this tree pedantically), named jobs, explicit timeouts. Keeping the additions out of `ci.yml` preserves the mechanical assertion that ci.yml's `- run:` list and the fenced gate block in DEVELOPMENT.md are identical — adding runs there would force edits to the gate list, which would change what every developer must run locally. These jobs are assurance beyond the gate, not the gate.

Jobs:

1. **compatibility** — install `cargo-public-api`, run `cargo contracts api-check`. Fails on any un-blessed public-API difference.
2. **determinism** — checkout, pinned toolchain, cache, `cargo fetch --locked`, `cargo contracts generate`, then require `git diff --exit-code` to pass and `git status --porcelain` to be empty. DEVELOPMENT.md removed the generate-then-diff pair *from the gate* because `check` already proves committed bytes match freshly computed bytes; this job proves something `check` cannot: that the **write path** on a clean environment reproduces the committed tree end to end, which also covers DEVELOPMENT.md's stated residual gap (an orphan written under an unexpected extension would appear here as an untracked file).
3. **package** — build `ratatoskr-contracts-typescript-<sha>.tar.gz` from `generated/typescript/` with reproducible tar flags (`--sort=name --owner=0 --group=0 --numeric-owner`) and upload it with `actions/upload-artifact`. The archive is transport for consumers trialling the declarations; publishing remains milestone 10. Byte-determinism is asserted for generated sources, not for the archive, so gzip timestamps are acceptable here.

Fixture validation gets **no new job**: it is enforced today by the gate job (`cargo contracts check` fixture step plus `cargo test --workspace --locked` asserting the JSON-Schema-layer expectations), and the workflow header comment plus DEVELOPMENT.md say so explicitly, so nobody reads the absence of a fixture job in contracts.yml as a gap.

### D5. Toolchain reality of `cargo-public-api`

rustdoc JSON generation requires a nightly compiler; the repository pins stable 1.97.0 as its only supported toolchain. The resolution: the pinned stable toolchain keeps building and testing everything; `cargo-public-api` drives its own doc generation and may need a nightly toolchain present on the machine. CI installs the binary through `taiki-e/install-action` when supported (the same installer ci.yml already trusts for cargo-deny) and otherwise falls back to `cargo install cargo-public-api --locked`; whichever path is chosen, the nightly requirement is satisfied explicitly in the job with a commented step rather than left to surprise. Local developers get the same instruction in DEVELOPMENT.md. The exact installer support and auto-install behaviour were verified against upstream documentation during this change and are recorded in DEVELOPMENT.md rather than guessed here.

### D6. Where the tests live

`tools/contractsc/tests/api_compat.rs` follows the existing integration-suite layout (`determinism.rs`, `secrets.rs`, ...). Four tests named in the delta spec exercise the classifier and the write/check round trip against temporary directories; none of them invokes `cargo-public-api` itself, so they stay fast, hermetic and runnable under `cargo test --workspace --locked` without extra installed tools. The end-to-end invocation (real binary, real crates) is exercised once manually while landing this change and afterwards lives in CI's compatibility job.

## Risks

| Risk | Response |
| --- | --- |
| `cargo-public-api` output format changes across versions | Sorted lines plus provenance header turn any format shift into an explicit, reviewable diff; bless flow unchanged |
| Nightly toolchain missing in CI | Explicit install step in the compatibility job, decided from verified upstream behaviour |
| Baselines drift from reality because nobody regenerates | `api-check` fails closed; regeneration is one command and lands with the PR that changed the API |
| Duplicate fixture enforcement creeping in later | Spec states the gate is the single enforcement point; workflow comment says the same |

## Out of scope

Tag-based baselining (needs the first release), npm package metadata and registry publishing (milestone 10), consumer-repository integration, Kotlin/Swift generation.
