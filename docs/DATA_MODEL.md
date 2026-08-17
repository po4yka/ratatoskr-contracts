# Contracts data model

> Status: Proposed  
> Last reviewed: 2026-08-17

This repository has no runtime database.

## Owned records

- Canonical schema sources by contract family.
- Metadata: owner, producer, consumers, version, status, classification, canonical path, generated targets.
- Valid/invalid fixtures and compatibility baselines.
- Generated package manifests and source artifacts.
- Deprecation and compatibility reports.

## Constraints

Contract identifiers and versions are unique. Generated output is fully derived. Fixtures reference synthetic identities and data. Contract metadata is machine-readable. Schema ordering and output formatting are deterministic.

## Lifecycle

`proposed -> accepted -> published -> deprecated -> removed`.

Removal requires evidence that supported consumers no longer depend on the contract. Historical versions remain available according to package retention policy.
