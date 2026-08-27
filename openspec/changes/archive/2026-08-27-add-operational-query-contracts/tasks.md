## 1. Status contract

- [x] 1.1 RED: add `crates/operational-contracts/tests/public_status.rs` test
  `public_status_rejects_unknown_or_unsanitized_components`; verify valid round trips and the
  unknown/private-field assertion fails before implementation.
- [x] 1.2 GREEN: implement status types, validation, and exact grant/capability constants; verify
  the focused test and crate tests pass.

## 2. Inspection contracts

- [x] 2.1 RED: add `crates/operational-contracts/tests/inspection.rs` test
  `operational_pages_are_bounded_and_content_free`; verify the over-limit/private-content assertion
  fails before implementation.
- [x] 2.2 GREEN: implement bounded operation, schedule, and audit row/page types; verify the focused
  test and crate tests pass.

## 3. Generator integration

- [x] 3.1 RED: add `operational_contract_is_registered_once` to
  `tools/contractsc/tests/registry.rs`; verify it fails because the family is absent.
- [x] 3.2 GREEN: add workspace/metadata registration, fixtures, invalid expectations, and public API
  baseline; verify the registry and fixture tests pass.
- [x] 3.3 Generate JSON Schema and TypeScript outputs and run `cargo contracts check-typescript`;
  generated files cannot start from a behavior test, so verify reviewed deterministic diffs and no
  orphan output.

## 4. Verification and lifecycle

- [x] 4.1 Run the exact DEVELOPMENT.md gate through `build-gate --` for compiler-backed commands;
  verify every step passes and inspect the final diff for privacy and scope.
- [x] 4.2 Validate and archive this change with every prior task checked, then verify
  `openspec validate --archived --strict` passes before publication.
