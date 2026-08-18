# Contracts interfaces

> Status: Proposed  
> Last reviewed: 2026-08-17

## Inputs

Canonical schema source, metadata, fixtures, OpenAPI owned by API producers, and workspace producer/consumer inventory.

## Outputs

Versioned Rust crates, TypeScript packages, JSON Schema, OpenAPI artifacts, event catalogs, fixtures, and compatibility reports.

## Rules

- Contract metadata includes owner, version, classification, canonical source, and generated targets.
- Commands are requests; events are past-tense facts.
- Error envelopes expose stable code, safe message, retry class, correlation ID, and partial effects.
- Blob references expose hash, size, media type, and authorization metadata, not raw storage paths.
- Public IDs are stable, and are never a database sequence ID. "Namespaced" means the `<kind>:<local_id>` wire form of a **pointer** at another record (ADR-0007); a field carrying a record's own identity is a bare UUID and is not kind-prefixed.
- Packages are tagged/versioned; production repositories do not use committed sibling path dependencies.
- Additive changes are default. Breaking changes require new major contract or coordinated migration.

Consumers must test unknown fields, duplicate events, and supported version ranges.
