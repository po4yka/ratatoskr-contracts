## 1. Contract behaviour

- [x] 1.1 RED: add a focused `OperationResultRef` test that expects an authored
  `ai_archive_import_summary` to decode as a typed field rather than an `extensions` member;
  observe it fail because the field does not yet exist. Verified by the observed failing assertion
  in `snapshot_roundtrip`.
- [x] 1.2 GREEN: add the bounded `AiArchiveOperationSummary` type and optional typed operation
  result field; verify the focused test passes.
- [x] 1.3 RED: add result/snapshot tests for a summary on a non-AI result and for a summary id that
  differs from its `ai_archive` target; verify both fail because the association is not yet
  enforced.
- [x] 1.4 GREEN: validate the summary kind and target association during construction and
  deserialization; verify the focused tests pass.

## 2. Artifacts and compatibility

- [x] 2.1 Add a valid archive-operation snapshot fixture and invalid association fixtures with
  declared rejecting layers. Fixture data cannot begin as a failing test; verify it through
  `cargo contracts check` after generation.
- [x] 2.2 Generate JSON Schema and TypeScript outputs and update the public API baseline. Generated
  artifacts cannot begin as a failing test; verify `cargo contracts check` and
  `cargo contracts api-check`.

## 3. Gate

- [x] 3.1 Run the documented contracts gate in `DEVELOPMENT.md` after all artifacts are current.
