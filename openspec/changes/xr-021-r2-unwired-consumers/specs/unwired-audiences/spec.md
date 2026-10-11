## Purpose

Defines how the contracts registry test tells a wired producer or consumer from a declared-only one.

## ADDED Requirements

### Requirement: Declared audiences that nothing wires are listed

Every producer or consumer that the registry declares on a pinned message row SHALL be either wired, as stated by the pinned S01 row, or listed in the `UNWIRED` table with a reason.

#### Scenario: An unlisted declared audience is rejected
- **WHEN** the registry row of a pinned message carries a service that is neither in its wired list nor in `UNWIRED`
- **THEN** the registry test fails naming the message type

#### Scenario: A listed entry that the registry does not declare is rejected
- **WHEN** an `UNWIRED` entry names a service that is not on that side of the registry row
- **THEN** the registry test fails naming the contract and the service

### Requirement: Transfer rows are not bus messages

The `transfer.*` registry rows SHALL NOT be registered as a command or an event.

#### Scenario: A transfer row registered as an event is rejected
- **WHEN** a `transfer.*` row carries an event or command registration
- **THEN** the registry test fails naming the row
