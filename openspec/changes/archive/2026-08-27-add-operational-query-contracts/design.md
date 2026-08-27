## Context

See proposal.md and `specs/operational-query-contracts/spec.md`. Existing contract families use one
Rust crate, strict serde shapes, Schemars output, metadata-owned fixtures, mirrored TypeScript, and a
frozen `cargo public-api` baseline. Platform currently defines operation-list response shapes
locally, while the fleet change requires one reusable contract boundary for status, operations,
schedules, and audit projections.

## Goals / Non-Goals

**Goals:**

- Make privacy and collection bounds part of type construction and schema validation.
- Reuse existing identifier, operation-state, timestamp, and generator conventions.
- Emit one deterministic operational contract family for Platform and generated clients.

**Non-Goals:**

- Route definitions, authorization decisions, database queries, or status aggregation policy.
- A role model, job model, schedule mutation, audit export, or diagnostic payload.
- A second API or contract major.

## Decisions

### One additive crate owns the operational response family

Add `ratatoskr-operational-contracts` rather than expanding operation lifecycle contracts. The
existing crate describes one user's long-running operation and its events; deployment-wide queries,
schedule projection, audit history, and public readiness form a separate read boundary. Platform-
local structs were rejected because Web and future clients need generated artifacts from one
canonical source.

### Pages validate at construction and deserialize through validated wire forms

Each page is limited to 100 entries and each opaque cursor or free string has an explicit byte
bound. Public structs use strict unknown-field rejection. Closed enums represent values on which a
consumer may branch; bounded newtypes represent display-only service, schedule, action, target, and
failure labels. Unbounded `String` and `serde_json::Value` fields are not exposed.

### Existing canonical primitives are reused

Operation lifecycle state and safe snapshots remain owned by `ratatoskr-operation-contracts`;
identifier types remain owned by `ratatoskr-identifiers`; timestamps use Jiff as in other families.
The new crate depends on those workspace members instead of copying wire types or accepting casts in
consumers.

### One metadata family emits all public response roots

`contracts.toml` registers status and the three page roots under an `operational` family, with valid,
invalid, and privacy fixtures for every boundary. The registry test requires exactly one family and
both JSON Schema and TypeScript targets. Generator output and the Rust public API baseline remain
read-only gate inputs after deliberate generation.

## Risks / Trade-offs

- [A shared crate can become a generic admin dumping ground] -> Register only the status and three
  fleet-specified projections; every new root requires its own observable contract change.
- [String bounds can reject a currently emitted Platform value] -> Use documented generous bounds
  and test representative maximums before Platform pins the crate.
- [Duplicating operation detail would drift] -> Re-export or embed the existing safe snapshot type
  rather than defining a second detail shape.

## Migration Plan

Generate and publish the additive crate first. Platform pins the resulting full commit SHA and uses
the shared types on new additive routes. Rollback removes Platform's consumption; the unused
additive contract can remain without affecting existing consumers.
