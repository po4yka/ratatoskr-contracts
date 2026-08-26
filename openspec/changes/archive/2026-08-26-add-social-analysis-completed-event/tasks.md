# Tasks: add-social-analysis-completed-event

## 1. Payload contract

- [x] 1.1 Add failing test `tools/contractsc/tests/fixtures.rs::social_source_analysis_completed_is_registered`; it asserts the typed payload is registered as `knowledge.analysis.completed.v1` and owns a fixture family for its linkage fields. Observed passing on current `main`.
- [x] 1.2 Implement and export `SocialSourceAnalysisCompleted` in the social contracts event module until the test passes. Verified by the registry test and envelope round-trip test.

## 2. Contract artifacts

- [x] 2.1 Add failing fixture test that requires the new family to have valid, invalid, and both compatibility-direction fixtures registered in `contracts.toml`. Verified by the registered fixture family and both compatibility-direction tests.
- [x] 2.2 Register the payload, generate JSON Schema and TypeScript artifacts, add fixtures and invalid expectations, and make the fixture test pass. `cargo contracts check` reports every artifact current.

## 3. Verification

- [x] 3.1 Run the exact `DEVELOPMENT.md` gate and OpenSpec validation; archive only after every task is checked. Full local gate, `openspec validate --all --strict`, and hosted checks for `b53fd76` passed.
