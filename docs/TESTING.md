# Contracts testing strategy

> Status: Proposed  
> Last reviewed: 2026-08-17

Required tests:

- Schema self-validation and valid/invalid fixtures.
- Backward/forward compatibility by family.
- Deterministic generation and clean-tree checks.
- Rust/TypeScript compile and round-trip serialization.
- Unknown field/variant behavior.
- OpenAPI lint and generated-client compilation.
- Secret/PII fixture scanning.
- Workspace producer/consumer impact and version-range checks.

A test must demonstrate detection of a deliberate breaking change. Updating snapshots without explaining semantic change is prohibited. Release artifacts are rebuilt and compared with committed/generated expectations.
