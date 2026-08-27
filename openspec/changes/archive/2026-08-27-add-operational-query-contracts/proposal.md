## Why

Platform cannot expose plan item 12 without canonical, privacy-bounded wire shapes for public
status and owner-authorized operational inspection. The fleet change
`add-operational-status-workspace-integration` defines the cross-repository behavior; this local
change defines the Contracts-owned serialization and validation boundary that Platform will pin.

## What Changes

- Add `ratatoskr-operational-contracts` with closed public status, operational capability, and owner
  grant vocabularies.
- Add validated response types for bounded operation, schedule, and audit pages that omit payloads,
  credentials, arbitrary diagnostics, and user content.
- Register the contract family, compatibility fixtures, JSON Schema outputs, TypeScript outputs,
  and frozen Rust public API.

## Capabilities

### New Capabilities

- `operational-query-contracts`: Canonical validated wire shapes for sanitized public status and
  bounded owner operational inspection.

### Modified Capabilities

None.

## Impact

The Cargo workspace gains one additive crate and the generator gains one contract family with
reviewed fixtures and generated artifacts. Platform becomes the first downstream consumer after
this change is published. No database schema, API version, event envelope, or existing contract is
changed.
