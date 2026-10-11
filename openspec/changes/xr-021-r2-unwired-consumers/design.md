## Context

See proposal.md and XR-021 CONTRACTS.md sections S01 and R2-19. The registry is a declaration of intended audiences; rule R7 in `tools/contractsc/src/metadata.rs` requires non-empty owner, producers and consumers, so an unwired row cannot be expressed by an empty list.

## Decisions

- The table is keyed by contract id and side because the registry lists services per contract, while the pinned rows are keyed by message type. The test resolves a message type to its contract and adds the `UNWIRED` entries for that contract id to the wired lists before comparing.
- `platform.operation_progressed` lists both sides: Platform does not publish the event today (its README says so), so the producer is as unwired as the Knowledge consumer.
- The `transfer.upload_*` rows list HTTP counterparts. They are not bus audiences and are not wired or unwired in the bus sense, so they are not entered in `UNWIRED`; a separate test asserts that no `transfer.*` row is registered as a command or event.
- Wiring an audience means deleting its `UNWIRED` entry and adding the service to the wired list of the pinned row in the same commit.

## Risks / Trade-offs

- The first run pins existing data, so it cannot fail on its own. It was made red by extending the pinned rows before filling `UNWIRED`, and mutation-verified: removing an entry fails the audience assertion, and renaming a service in `contracts.toml` fails the existence assertion.
- Telegram's dispatcher reads `platform.operation.progressed.v1` while the registry lists only Knowledge as its consumer; correcting the registry audience is outside this change.

## Migration Plan

Test and documentation only. Nothing re-pins to the resulting commit.
