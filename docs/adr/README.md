# Contracts ADRs

> Status: Proposed  
> Last reviewed: 2026-08-17

Use `NNNN-short-title.md`. Each ADR records context, drivers, options, decision, consequences, security/privacy, compatibility/migration, validation, and follow-up. Supersede decisions; do not rewrite history.

Accepted:

- [ADR-0001](0001-canonical-schema-source-format.md): Canonical schema source format. Rust-first; `schemas/**` is generated and never hand-edited.
- [ADR-0002](0002-event-naming-and-major-version-strategy.md): Event naming and major-version strategy. Two version axes: `event_type`'s `.v<major>` versions the payload, `schema_version` versions the envelope.

Backlog:

- ADR-0003: Document IR and provenance representation.
- ADR-0004: Unknown-field and unknown-variant policy.
- ADR-0005: Deterministic generated package publishing.
- ADR-0006: Expand/migrate/contract compatibility policy.
