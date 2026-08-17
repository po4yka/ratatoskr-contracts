# Contracts domain model

> Status: Proposed  
> Last reviewed: 2026-08-17

## Terms

- **Contract family:** related commands, events, APIs, or shared value types.
- **Canonical schema:** reviewed source from which artifacts are generated.
- **Wire contract:** representation exchanged across independent versions/processes.
- **Fixture:** stable valid or invalid example payload.
- **Compatibility report:** machine-readable comparison of revisions.
- **Generated artifact:** deterministic source/package derived from canonical schemas.

## Invariants

1. Shared contracts contain only boundary data, not service implementation.
2. Every contract has explicit authority and version.
3. Generated code is derivative and replaceable.
4. Nullability and absence semantics are explicit.
5. Provider observations, user intent, and derived analysis are distinguishable.
6. Unknown external variants are preserved or rejected explicitly.
7. Contraction occurs only after consumers have migrated.

Schema changes are facts about process boundaries; they never grant direct access to another service's database.
