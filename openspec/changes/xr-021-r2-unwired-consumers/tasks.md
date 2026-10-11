## 1. Pin the unwired set (XR-021 CONTRACTS.md S01, R2-19)

- [x] 1.1 Extend `tools/contractsc/tests/live_message_types.rs` with the four contracts that carry declared-only audiences and an empty `UNWIRED` table; `every_live_message_has_the_producers_and_consumers_of_the_bus_table` and `declared_audiences_that_nothing_wires_are_listed_and_nothing_else` MUST fail on the audience assertion because the registry declares audiences that are neither wired nor listed. Then mutation-verify: remove one `UNWIRED` entry and expect the audience assertion to fail, rename a service in `contracts.toml` and expect the existence assertion to fail, and restore both.
- [x] 1.2 Fill `UNWIRED` with the nine entries named in S01 and R2-19 and add `transfer_rows_are_http_counterparts_and_not_bus_messages`; the tests pass.

## 2. Documentation

- [x] 2.1 Add one paragraph to `DEVELOPMENT.md` saying that a registry audience is a declaration and that S01 plus the table say which are wired. This cannot start from a failing test (documentation).

## 3. Final gate

- [x] 3.1 Run the full gate of `DEVELOPMENT.md`, `cargo contracts compat`, `cargo deny check` and `openspec validate --all --strict`; every step passes on the committed tree and `cargo contracts check` reports no generated-artifact drift. Verification only, no test to write.
