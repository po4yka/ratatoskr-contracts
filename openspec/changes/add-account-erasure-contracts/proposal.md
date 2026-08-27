## Why

Platform cannot coordinate verified all-Ratatoskr-data erasure across independent
data owners without a shared, secret-free command and terminal acknowledgement.
The workspace change `coordinate-account-controls-lifecycle` establishes the
cross-repository behaviour; this change supplies only its asynchronous wire
boundary.

## What Changes

- Add an account-erasure command, owner acknowledgement, progress, and terminal
  outcome contracts to `ratatoskr-operation-contracts`.
- Require the command and acknowledgement to identify the internal owner and
  erasure operation without carrying confirmation text, idempotency keys,
  provider identities, source content, credentials, or secrets.
- Model verified local erasure and incomplete external-grant revocation as
  distinct terminal facts that Platform can aggregate truthfully.
- Generate deterministic JSON Schema and TypeScript artifacts plus valid,
  invalid, and compatibility fixtures from the canonical Rust types.

## Capabilities

### New Capabilities

- `account-erasure-events`: Secret-free asynchronous erasure command and owner
  acknowledgement contracts for Platform and every user-data owner.

### Modified Capabilities

<!-- None. -->

## Impact

Extends `crates/operation-contracts`, `contracts.toml`, generated schemas and
TypeScript declarations, and fixtures. Producers are Platform and every
participating data owner; consumers are Platform's operation aggregator and
the same owners. It is additive and precedes service implementations. Public
Edge account-controls projection and mutation endpoints remain exclusively in
`ratatoskr-platform`.
