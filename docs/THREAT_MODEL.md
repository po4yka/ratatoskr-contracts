# Contracts threat model

> Status: Proposed  
> Last reviewed: 2026-08-17

## Assets

Contract integrity, generated packages, compatibility guarantees, identity/privacy semantics, fixtures, and release provenance.

## Threats

- **Breaking schema accepted as additive:** semantic compatibility tests plus human review.
- **Ambiguous or unsafe schema:** explicit bounds, nullability, discriminators, formats, and authority semantics.
- **Sensitive fixture leak:** synthetic fixtures, secret/PII scanning of every file under `fixtures/`, and no personal exports.
- **Provider payload smuggled through a preserved extension channel:** the tolerant-reader contracts carry an unbounded `extensions` map that consumers re-emit verbatim, so the typed surface's exclusion of stack traces and raw provider responses does not bind what a producer puts there. Mitigation (ADR-0008): the channel is declared preservation-only, and a service asserts `extensions.is_empty()` on the envelopes it constructs. Residual, unchanged in kind and stated in full: the assertion binds only the test suites of in-workspace services on the current major. It does not bind a producer on a newer major than a relay, an out-of-workspace producer, or a service that hand-rolls its own JSON. `ErrorEnvelope.extensions` is `#[serde(flatten)]` and `ErrorEnvelope` is embedded in `OperationSnapshot.errors`, so an upstream key fans out through `platform.operation.progressed.v1` and the SSE stream to a lower trust tier. Keeping the channel clean remains a producer-side review obligation, not a wire constraint.
- **Generator compromise:** pinned dependencies, reproducible output, code review, and package provenance.
- **Malicious schema text/codegen injection:** treat descriptions/examples as data and escape generated output.
- **Consumer downgrade or drift:** versioned packages, supported-range tests, and workspace impact analysis.
- **Overshared domain model:** reject types added only to reuse internal implementation.

Re-review when adding a contract carrying credentials, private content, identity assertions, destructive intent, or executable-like data.
