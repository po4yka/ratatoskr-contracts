# Contracts ADRs

> Status: Proposed  
> Last reviewed: 2026-08-18

Use `NNNN-short-title.md`. Each ADR records context, drivers, options, decision, consequences, security/privacy, compatibility/migration, validation, and follow-up. Supersede decisions; do not rewrite history.

Accepted:

- [ADR-0001](0001-canonical-schema-source-format.md): Canonical schema source format. Rust-first; `schemas/**` is generated and never hand-edited.
- [ADR-0002](0002-event-naming-and-major-version-strategy.md): Event naming and major-version strategy. Two version axes: `event_type`'s `.v<major>` versions the payload, `schema_version` versions the envelope. The `schema_version` reading is confirmed and the follow-up is closed.
- [ADR-0007](0007-identifier-wire-form.md): Identifier wire form. Own identity is a bare typed UUID; a pointer at another record is `<kind>:<local_id>`; an external-system handle keeps that system's grammar. One spelling per UUID reference. Supersedes ADR-0002 §Consequences' identifier bullet.
- [ADR-0008](0008-error-envelope-structured-detail.md): Structured detail on the error envelope. No `details` member on `core.error_envelope` major 1; `extensions` is a preservation channel, not an authoring channel.

Backlog. These numbers are reserved and are cited by number from the accepted ADRs, so they are not reassigned:

- ADR-0003: Document IR and provenance representation.
- ADR-0004: Unknown-field and unknown-variant policy.
- ADR-0005: Deterministic generated package publishing.
- ADR-0006: Expand/migrate/contract compatibility policy.
- ADR-0009: Error retry classification. Raised by ADR-0008 §Follow-up: `retryable: bool` versus a closed `retry` class, and where `retry_after` lives.
