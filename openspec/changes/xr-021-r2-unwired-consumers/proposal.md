## Why

Changeset XR-021 round 2 (decision R2-19 in XR-021 CONTRACTS.md) keeps several registry audiences that nothing wires: the Knowledge completion facts have no consumer, `platform.notification.raised.v1` has no publisher, and `platform.operation.progressed.v1` is not published on the bus. Rule R7 forbids an empty producers or consumers list, so the rows keep the audience they were designed for. `tools/contractsc/tests/live_message_types.rs` pins only the rows S01 wires, so a declared-but-unwired audience is indistinguishable from a wired one, and a new declared audience that nobody wires would pass unnoticed.

## What Changes

- Add an `UNWIRED` table to `tools/contractsc/tests/live_message_types.rs` that names every declared registry audience nothing wires, with a one-line reason, and extend the pinned rows with the four contracts that carry them.
- The bus-table test now asserts that each pinned row's registry audience equals its wired audience plus its `UNWIRED` entries, so a declared audience that is neither wired nor listed fails.
- Add a test that the `transfer.*` rows are HTTP bodies, not bus messages, so their HTTP counterparts are not read as bus audiences.
- One paragraph in `DEVELOPMENT.md` states that a registry audience is a declaration and that S01 plus the table say which are wired.

No wire shape, schema, fixture, generated artifact or API baseline changes. Cross-repository behaviour is defined in XR-021 CONTRACTS.md sections S01 and R2-19.

## Capabilities

### New Capabilities

- `unwired-audiences`: the registry test that separates wired audiences from declared-only ones.

### Modified Capabilities

<!-- None. -->

## Impact

Touches `tools/contractsc/tests/live_message_types.rs` and `DEVELOPMENT.md`. No other repository waits for this commit or moves its pin to it. Rollback: revert the commit.
