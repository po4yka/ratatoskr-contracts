# Contracts threat model

> Status: Proposed  
> Last reviewed: 2026-08-17

## Assets

Contract integrity, generated packages, compatibility guarantees, identity/privacy semantics, fixtures, and release provenance.

## Threats

- **Breaking schema accepted as additive:** semantic compatibility tests plus human review.
- **Ambiguous or unsafe schema:** explicit bounds, nullability, discriminators, formats, and authority semantics.
- **Sensitive fixture leak:** synthetic fixtures, secret/PII scanning, and no personal exports.
- **Generator compromise:** pinned dependencies, reproducible output, code review, and package provenance.
- **Malicious schema text/codegen injection:** treat descriptions/examples as data and escape generated output.
- **Consumer downgrade or drift:** versioned packages, supported-range tests, and workspace impact analysis.
- **Overshared domain model:** reject types added only to reuse internal implementation.

Re-review when adding a contract carrying credentials, private content, identity assertions, destructive intent, or executable-like data.
