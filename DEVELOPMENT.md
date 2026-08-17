# Developing Ratatoskr Contracts

> Status: Proposed  
> Owner: `ratatoskr-contracts`  
> Last reviewed: 2026-08-17

## Current stage

Architecture bootstrap. Canonical schemas, generators, packages, fixtures, and CI do not exist yet. Commands below are requirements for the first scaffold, not claims about current tooling.

## Intended toolchain

Rust, Serde, Schemars, JSON Schema, OpenAPI, deterministic generators, and generated Rust/TypeScript packages. All versions and lockfiles must be pinned.

## Expected workflow

1. Identify contract owner, producers, consumers, privacy classification, and changeset.
2. Change the canonical schema, never generated output.
3. Add fixtures and compatibility expectations.
4. Regenerate all targets deterministically.
5. Compile generated packages and run round-trip tests.
6. Validate rollout with expand/migrate/contract.

Expected command surface: bootstrap/fetch, schema validation, generation, clean-tree drift check, compatibility comparison, tests, and package build/publish. The first code PR must document exact commands.

## Rules

No service-private domain models, ORM entities, credentials, or real private data belong here. Generated artifacts are reviewed but not hand-edited. Cross-repository contract work must use `ratatoskr-workspace`.
