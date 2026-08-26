## Context

See [proposal.md](proposal.md). `zizmor` reports two `template-injection` findings in the package job because `${{ github.sha }}` appears inside shell interpolation. The social analysis-completion contract has already made the generated root set 28/28, while four determinism assertions still describe the preceding 27/27 set.

## Goals / Non-Goals

**Goals:**

- Eliminate shell-template expansion while preserving the exact SHA-named archive and upload path.
- Make the cardinality guard match the committed generated root set, retaining a clear maintenance signal when it changes again.

**Non-Goals:**

- Change artifact contents, publication policy, permissions, or the security scanner's severity/persona.
- Derive the cardinality from production code: these tests deliberately pin it so a registry expansion requires an explicit reviewed update.

## Decisions

Use the runner-provided `GITHUB_SHA` environment variable inside the shell script and store the full filename in a quoted shell variable. GitHub Actions guarantees that variable for a workflow run; it avoids rendering expression syntax into shell source while yielding the same full commit SHA as `github.sha`.

Keep `github.sha` in action inputs where it is data rather than shell syntax. Replacing it there would not address the reported sink and would broaden the change without benefit.

Update all four test assertions and their diagnostic text from 27/54 to 28/56. The generator and committed outputs already agree at 28 per family, so changing production registry code or regenerating artifacts would be unrelated and would hide the actual stale-test defect.

Move `SocialSourceAnalysisCompleted` before `SocialSourceCaptured` in the hand-written root registry, where the documented M-2 rule requires lexical `rust_path` order. Update M-8's reviewed event count from 10 to 11; the event was already correctly represented in the metadata and event-payload map, so this restores the guard rather than changing event semantics.

## Risks / Trade-offs

- [Filename differs from the action-input value] → The test asserts the constructed filename uses `GITHUB_SHA`, and the existing upload input still uses `github.sha`.
- [A future workflow edits the shell command] → `zizmor` remains a fail-closed gate for template injection.
- [A future root changes the total] → The explicit cardinality assertion fails with an instruction to update the reviewed pin.

## Migration Plan

Merge the one-workflow-line change and observe `zizmor` pass. Reverting restores the former workflow behaviour; no data or compatibility migration exists.
