## Context

See proposal.md and `account-erasure-events/spec.md`. The command envelope
already carries command identity, correlation, aggregate, and tenant scope;
the event envelope already carries at-least-once event identity and preserves
unknown additive fields. `ratatoskr-operation-contracts` is the existing home
for Platform operation messages and already depends on the envelope crate.

## Goals / Non-Goals

**Goals:**

- Add typed command and terminal acknowledgement payloads that bind owner work
  to a durable Platform operation without carrying sensitive account data.
- Make a provider-grant revocation failure observable without representing it
  as verified account erasure.
- Generate and test both wire artifacts from one Rust source of truth.

**Non-Goals:**

- Public Edge endpoints, account-controls projections, preference/API-key
  payloads, confirmation/idempotency request semantics, service deletion
  handlers, or a distributed transaction.

## Decisions

### Use the existing command and event envelopes

`AccountErasureRequested` implements `CommandPayload` with the type
`identity.account.erasure_requested.v1`; `AccountErasureAcknowledged`
implements `EventPayload` with the type
`identity.account.erasure_acknowledged.v1`. The command envelope supplies
tenant and correlation scope, while the payload supplies the durable operation
identity needed by an owner that receives an at-least-once delivery.

A hand-rolled erasure envelope is rejected because it would create duplicate
delivery, extension, and version semantics.

### Put the payloads in operation contracts

The types live in `ratatoskr-operation-contracts`, alongside the Platform
operation messages that receive owner reports. This avoids a new identity
crate for two wire payloads and preserves the existing dependency direction.

### Encode only an opaque terminal outcome

The acknowledgement uses a closed outcome that differentiates verified local
erasure from incomplete external-grant revocation. It carries no provider
identifier, remote response, content, credential, confirmation value, or
idempotency key. Platform combines acknowledgements; the event does not claim
global erasure completion.

## Risks / Trade-offs

- [An owner reports success before deleting local data] → service-owner tests
  must prove their handlers delete data before emitting a verified outcome.
- [Platform treats an incomplete acknowledgement as complete] → Platform
  aggregation tests consume the typed closed outcome before public completion.
- [Sensitive account data reaches the message] → fixtures and schema tests
  assert the forbidden members are absent; `cargo contracts check` scans
  fixtures for sensitive material.

## Migration Plan

1. Add red public-API tests for the command and acknowledgement payloads.
2. Add canonical Rust types, metadata registrations, fixtures, and generated
   artifacts; run compatibility checks.
3. Merge this additive contract before data owners and Platform implement their
   handlers and public account-controls operation.
4. Rollback stops publishing or consuming the new message types. It cannot
   restore a terminal account erasure already completed by a service owner.
