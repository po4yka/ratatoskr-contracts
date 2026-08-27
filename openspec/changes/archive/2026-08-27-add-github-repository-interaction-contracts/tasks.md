## 1. Preview Contract

- [x] 1.1 Add `crates/github-contracts/tests/repository_interaction.rs::preview_contract_round_trips_and_refuses_unknown_fields` plus only the minimal compiling declarations; predict and observe the assertion fail because invalid/unknown preview data is accepted or valid fields do not round-trip, using `build-gate -- cargo nextest run --locked -p ratatoskr-github-contracts --test repository_interaction preview_contract_round_trips_and_refuses_unknown_fields`
- [x] 1.2 Implement strict preview request/response, stable target, bounded display values, account/capability types, and exports; verify the named test and the whole `ratatoskr-github-contracts` crate pass

## 2. Action Request Contract

- [x] 2.1 Add `repository_action_requires_mode_target_confirmation_and_idempotency` in the same test file; predict and observe failure because an unknown mode, missing star account, empty confirmation, or unsafe payload is accepted
- [x] 2.2 Implement the closed mode and validated action-request types without credentials or policy JSON; verify the named test and crate tests pass

## 3. Truthful Component Results

- [x] 3.1 Add `component_outcomes_reject_inconsistent_aggregate_and_backup_claims`; predict and observe failure because an inconsistent aggregate or illegal `accepted` provider/metadata result can be constructed or decoded
- [x] 3.2 Implement component-specific outcomes, safe reason codes, aggregate derivation/validation, and partial-result serialization; verify the named test and crate tests pass

## 4. Generated Contract Artifacts

- [x] 4.1 Update the generator registrations, JSON Schema, TypeScript output, and public API baseline; this cannot start with a separate failing behavior test because these files are deterministic derivatives, so verify generation twice leaves the second run clean and inspect every generated diff
- [x] 4.2 Add/refresh invalid and secret-lint fixtures for unknown fields, illegal outcome combinations, credential-like members, and invalid bounds; verify the focused `contractsc` suites fail on the bad fixtures and pass on the valid contract

## 5. Validation and Delivery

- [x] 5.1 Run the exact fenced gate from `DEVELOPMENT.md` through `build-gate` for compiler-backed commands, run `openspec validate add-github-repository-interaction-contracts --type change --strict`, and review `git diff --check` plus the complete intended diff
- [x] 5.2 After network access is restored, fetch/rebase on current `origin/main`, rerun the full gate, commit only this change, integrate it into contracts `main`, push `main`, and record the immutable merged SHA for downstream pins (`f3de4f977335194712946b7c7f18d24ea6038c04`)
