# operational-query-contracts Specification

## Purpose

Defines canonical, validated wire shapes for sanitized public status and bounded owner operational
inspection without exposing service topology, request payloads, credentials, or user content.

## Requirements

### Requirement: Public status vocabulary is closed and sanitized

The contract SHALL define four stable component identifiers, closed component and overall states,
an observation timestamp, and an explicit stale flag. It SHALL reject unknown enum values and
fields outside the contracted shape.

#### Scenario: Sanitized status round trip succeeds
- **WHEN** a producer serializes a status document containing only contracted identifiers, states, timestamps, and stale facts
- **THEN** the document round trips through the canonical type and generated schema without information loss

#### Scenario: Diagnostic enrichment is rejected
- **WHEN** a status document contains an internal address, service name, diagnostic, or unknown component
- **THEN** canonical deserialization or schema validation rejects the document

### Requirement: Operational pages are bounded and content-free

Operation, schedule, and audit response pages SHALL accept at most 100 rows and a bounded opaque
continuation cursor. Their rows SHALL contain only contracted identifiers, closed states, bounded
labels or stable codes, timestamps, and documented nullable attribution fields.

#### Scenario: Bounded page round trip succeeds
- **WHEN** a page contains no more than 100 valid operation, schedule, or audit rows and a valid continuation cursor
- **THEN** the page round trips through the canonical type and generated schema

#### Scenario: Oversized or private content is rejected
- **WHEN** a page exceeds its row or text bounds or attempts to include arbitrary JSON, a request payload, credential, private URL, or diagnostic field
- **THEN** canonical validation rejects the page

### Requirement: Authorization vocabulary is exact

The contract SHALL publish exact constants for `platform.owner`, `platform.operations.inspect`,
`platform.schedules.inspect`, and `platform.audit.inspect` so producers and consumers do not invent
aliases or browser-owned privilege vocabularies.

#### Scenario: Consumer uses the canonical vocabulary
- **WHEN** a consumer branches on the exported grant and capability constants
- **THEN** the observed string values exactly match the fleet operational-inspection specification

### Requirement: Generated artifacts remain deterministic and compatible

The generator SHALL register the operational family once and SHALL emit committed JSON Schema,
TypeScript, fixtures, and Rust public API evidence from the canonical Rust types.

#### Scenario: Contract drift is detected
- **WHEN** canonical Rust types, metadata, fixtures, generated outputs, or the frozen public API disagree
- **THEN** the repository contract gate fails without rewriting committed evidence
