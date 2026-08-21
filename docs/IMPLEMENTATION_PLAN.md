# Contracts implementation plan

> Status: Proposed  
> Last reviewed: 2026-08-17

## Milestones

1. Define directories and machine-readable contract metadata.
2. Choose canonical schema source format and generator architecture.
3. Implement shared identifiers and event envelope.
4. Implement operation and error contracts. **Implemented.**
5. Implement Document IR and provenance. **Implemented.**
6. Implement social-source contracts.
7. Implement AI archive graph/content contracts.
8. Add deterministic Rust/TypeScript generation.
9. Add fixture, compatibility, and package CI.
10. Publish first tagged artifacts and integrate them in one vertical slice.

## Definition of Done

Schemas, fixtures, generation, compatibility, packages, security scanning, and consumer compilation pass. Contract changes include rollout/rollback and update all affected documentation.

Deferred: generic shared domain utilities, runtime services, and provider-specific internal payload mirrors.
