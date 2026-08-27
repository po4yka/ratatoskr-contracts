## Purpose

Defines the asynchronous, secret-free wire messages that coordinate verified
account erasure between Platform and every Ratatoskr user-data owner.

## ADDED Requirements

### Requirement: Account-erasure command identifies one durable operation

The contracts SHALL define `identity.account.erasure_requested.v1` as a typed
command payload. Its command envelope SHALL identify exactly one internal
owner and one durable erasure operation. Neither the payload nor its generated
schemas SHALL contain a confirmation phrase, idempotency key, provider account
identifier, source content, credential, or secret.

#### Scenario: An owner receives a secret-free erasure command
- **WHEN** a participating data owner decodes an
  `identity.account.erasure_requested.v1` command for one operation
- **THEN** it can identify the target internal owner and operation while no
  confirmation value, idempotency key, provider identity, content, or secret
  is present in the typed payload

### Requirement: Owner acknowledgement distinguishes verified erasure from incomplete revocation

The contracts SHALL define `identity.account.erasure_acknowledged.v1` as a
typed terminal event payload. It SHALL identify the same owner and erasure
operation and SHALL distinguish verified local Ratatoskr-data erasure from an
incomplete outcome caused by external provider-grant revocation. It SHALL not
encode provider account identifiers, provider response bodies, or credentials.

#### Scenario: A provider grant revoke remains an incomplete acknowledgement
- **WHEN** an owner removes its local Ratatoskr data and credentials but an
  external provider grant cannot be revoked
- **THEN** its acknowledgement is terminal and incomplete rather than verified,
  without exposing the provider account or provider response

### Requirement: Erasure messages preserve additive wire compatibility

The command and acknowledgement SHALL preserve unknown additive payload
members under the established extension policy and SHALL have valid, invalid,
and forward-compatibility fixtures. Their generated JSON Schema and TypeScript
declarations SHALL be deterministic products of the canonical Rust types.

#### Scenario: A newer acknowledgement retains an unknown additive member
- **WHEN** an older consumer decodes and re-emits an acknowledgement containing
  one unknown additive member
- **THEN** the member is retained unchanged and the known terminal outcome
  remains interpretable
