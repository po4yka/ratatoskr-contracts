## Why

Operation results currently provide only a generic pointer. Export clients cannot safely determine
whether a completed AI archive import is complete or contains gaps, and `extensions` cannot carry a
new authored contract field.

## What Changes

- Add a typed optional AI archive import summary to an operation result.
- Reuse the authoritative AI archive completeness vocabulary and expose only aggregate counts and
  a bounded report reference.
- Generate schemas and fixtures proving unknown operation-result extensions remain separate from
  the typed authored field.

## Capabilities

### New Capabilities

- `ai-archive-operation-result`: Typed operation-result completeness evidence for AI archive imports.

### Modified Capabilities

- `operation-progress`: Operation snapshots and reports carry the typed optional archive-import
  result summary.

## Impact

This is an additive wire-contract change consumed by Platform, AI archive producers, and
export-agent. It exposes no user content, paths, provider account identifiers, diagnostics, or
credentials. Older producers omit the optional field; consumers treat absence as unverified.
