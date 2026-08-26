# Tasks: add-social-analysis-completed-event

## 1. Payload contract

- [ ] 1.1 Add failing test `tools/contractsc/tests/fixtures.rs::social_source_analysis_completed_is_registered`; it asserts the typed payload is registered as `knowledge.analysis.completed.v1` and owns a fixture family for its linkage fields.
- [ ] 1.2 Implement and export `SocialSourceAnalysisCompleted` in the social contracts event module until the test passes.

## 2. Contract artifacts

- [ ] 2.1 Add failing fixture test that requires the new family to have valid, invalid, and both compatibility-direction fixtures registered in `contracts.toml`.
- [ ] 2.2 Register the payload, generate JSON Schema and TypeScript artifacts, add fixtures and invalid expectations, and make the fixture test pass.

## 3. Verification

- [ ] 3.1 Run the exact `DEVELOPMENT.md` gate and OpenSpec validation; archive only after every task is checked.
