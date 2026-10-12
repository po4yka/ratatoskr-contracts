## Purpose

Defines the payload with which Knowledge acknowledges a deletion tombstone or a social removal.

## ADDED Requirements

### Requirement: An acknowledgement is applied or refused, never both

`KnowledgeDeletionAcknowledged::validate` SHALL require `refusal` exactly when `outcome` is `refused`.

#### Scenario: A refused outcome without a refusal is rejected
- **WHEN** a payload has `outcome` `refused` and no `refusal`
- **THEN** `validate()` returns `DeletionAckRefusedWithoutRefusal`

#### Scenario: An applied outcome with a refusal is rejected
- **WHEN** a payload has `outcome` `applied` and a `refusal`
- **THEN** `validate()` returns `DeletionAckAppliedWithRefusal`

### Requirement: The vocabulary and identities are closed and typed

The family, outcome and refusal words SHALL be the exact snake_case words `ai_archive`, `social`, `applied`, `refused` and `foreign_subject`, the acknowledged event id SHALL be an event id and the owner SHALL be `user:<uuid>`.

#### Scenario: A malformed identity fails to deserialize
- **WHEN** `acknowledged_event_id` is not a UUID or `owner` is not `user:<uuid>`
- **THEN** deserialization fails

#### Scenario: An unknown member survives a round trip
- **WHEN** a payload carries a member this version does not know
- **THEN** it lands in `extensions` and is emitted again unchanged
