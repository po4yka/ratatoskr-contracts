# Contracts requirements

> Status: Proposed  
> Last reviewed: 2026-08-17

## Goals

1. Define canonical, versioned event, command, operation, error, Document IR, social, and AI archive schemas.
2. Publish deterministic Rust and TypeScript artifacts.
3. Enforce ownership, naming, privacy classification, compatibility, fixtures, and deprecation.
4. Make producer/consumer impact and rollout order visible.

## Non-goals

Shared ORM entities, business services, provider clients, runtime databases, secrets, and service-private models.

## Requirements

- Every public contract has owner, family, major version, producers, consumers, and canonical path.
- Event names follow `<context>.<aggregate>.<action>.v<major>`.
- Generated output is reproducible and never manually edited.
- Additive changes precede producer migration and later contraction.
- Unknown fields/variants remain representable where forward compatibility requires it.
- Examples validate and contain no sensitive real data.
- A breaking change requires a workspace changeset and explicit compatibility window.

## First-slice acceptance

Event envelope and identifier schemas validate; Rust/TypeScript generation is deterministic; fixtures round-trip; compatibility CI detects a deliberate break; package build succeeds.
