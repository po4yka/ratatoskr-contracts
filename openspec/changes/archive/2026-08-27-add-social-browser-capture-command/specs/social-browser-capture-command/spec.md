## Purpose

Defines the versioned Platform-to-social-owner request and outcome vocabulary for an explicit
browser capture without exposing browser or provider session data.

## ADDED Requirements

### Requirement: Commands have one canonical envelope
Every inter-service command SHALL use the canonical command envelope. The envelope SHALL carry its
own command identity, command type, issued timestamp, producer, aggregate identity, correlation
identity, optional causation and tenant identities, envelope schema version, object payload, and
unknown additive extensions. A command payload SHALL bind to exactly one command type, so a caller
cannot produce a command whose type disagrees with its payload. Consumers SHALL reject an unknown
command-envelope schema major and preserve unknown additive fields of a supported major.

#### Scenario: A typed command round-trips through the canonical envelope
- **WHEN** a `social.capture.requested.v1` payload is placed in a command envelope and then
  deserialized
- **THEN** its command type and payload round-trip together, and the envelope retains its identity
  and correlation fields

#### Scenario: A mismatched command payload is refused
- **WHEN** a consumer requests a social-capture payload from an envelope with another command type
- **THEN** the consumer receives a typed mismatch error and no social-capture payload

### Requirement: Published commands have verified command metadata
Every published command payload SHALL be registered in the contracts metadata as a `commands`
family member. Its metadata SHALL declare both its command type and its payload type, and contract
generation SHALL reject a declaration when either differs from the `CommandPayload` implementation.
The generated JSON Schema and TypeScript declaration SHALL be derived from that registered payload,
while the canonical command envelope is published as its own root schema.

#### Scenario: A matching command registration generates artifacts
- **WHEN** `social.capture.requested.v1` is registered with its matching payload type
- **THEN** contract generation produces its schema and TypeScript artifact from the declared root
  type

#### Scenario: A mismatched command registration is rejected
- **WHEN** a command metadata entry claims a type other than its payload's `COMMAND_TYPE`
- **THEN** contract validation fails before it accepts or generates the declaration

### Requirement: The social capture command has closed provenance and routing fields
`social.capture.requested.v1` SHALL carry the operation identity, idempotency key, original public
permalink, and capture timestamp in its payload, and a provider selected from `x`, `instagram`, or
`threads`. Its owner and command identity SHALL be carried by the canonical command envelope. It
SHALL represent this lane using the existing `browser_extension` acquisition and
`explicit_user_capture` saved-authority wire values, and SHALL reject an unknown provider or
provenance value. It SHALL NOT contain provider credentials, cookies, browser sessions, page
content, or native-Saved/bookmark state.

#### Scenario: A valid browser capture round-trips
- **WHEN** a command for each supported provider is serialized and deserialized
- **THEN** the original permalink, timestamp, operation identity, idempotency key, and declared
  provenance round-trip unchanged inside the typed command payload

#### Scenario: An unknown provider is refused
- **WHEN** a command contains a provider outside the closed vocabulary
- **THEN** deserialization fails and no typed command is produced

### Requirement: Social terminal outcome codes are closed and machine-readable
The contracts SHALL expose one closed `SocialCaptureOutcomeCode` vocabulary. It SHALL contain
`social.source.unavailable` and `social.source.deleted` for terminal source outcomes, and
`social.linked_article.extraction_failed` for a non-terminal linked-article warning. Every member
SHALL explicitly map to the shared `ErrorCode`; unknown spellings SHALL be refused before a caller
can classify the outcome. A consumer SHALL use this vocabulary rather than parsing human-readable
message text. A producer reporting a deleted or unavailable source SHALL not pair either terminal
outcome with a preserved `social.post` result.

#### Scenario: Partial social work retains the dependent failure
- **WHEN** a producer reports a preserved social post with a failed linked article
- **THEN** its operation report can carry `social.post` and
  `social.linked_article.extraction_failed` as distinct typed values
