# notification-contracts Specification

## Purpose
Defines the wire agreement through which producer services (`ratatoskr-knowledge`, `ratatoskr-github`, `ratatoskr-vault`, `ratatoskr-x`) raise typed user-facing notifications onto the documented bus surface that `ratatoskr-telegram`'s notification sender consumes: one registered event type inside the canonical envelope, a versioned and extensible class taxonomy, tenant/user addressing in the shared identifier grammar, opaque correlation references, and advisory delivery hints whose enforcement stays with the consumer.

## Requirements

### Requirement: One registered event type carries every raised notification

Every user-facing notification SHALL travel as the payload of exactly one registered event type, `platform.notification.raised.v1`, inside the canonical event envelope. The payload SHALL carry: the notification's own identity as a bare canonical UUID (`notification_id`); the taxonomy registry version the producer speaks; the notification class; the recipient as a closed tenant reference (`user:<uuid>`); a required carrier-safe `title`; an optional carrier-safe `message` detail; optional opaque correlation references (`operation_ref`, `analysis_ref`); optional advisory delivery hints; and unknown-but-preserved extensions. The carrying envelope's aggregate SHALL name the raised notification itself as `notification:<uuid>`. A consumer asking for a typed read of an envelope answering a different fact SHALL be refused, never half-interpreted.

#### Scenario: A fully populated notification travels typed inside a real envelope

- **WHEN** a payload carrying every member, including hints, both correlation references and preserved extensions, is bound into a parsed minimal envelope and the envelope is re-serialized and reparsed
- **THEN** the envelope's event type equals the payload's registered type, no field is dropped on the wire, and the payload comes back equal to what was sent

#### Scenario: A minimal notification without optional members is valid

- **WHEN** a payload carries only the required members
- **THEN** it deserializes, serializes without any omitted member reappearing, and round-trips losslessly

#### Scenario: A mismatched typed read is refused

- **WHEN** an envelope whose event type names another fact is asked for a `platform.notification.raised.v1` payload
- **THEN** the read fails naming the expected event type

### Requirement: The notification class taxonomy is versioned and extensible

The payload's `class` SHALL be drawn from a taxonomy registry whose known set at registry version 1 is exactly `operation_completed`, `operation_failed`, `analysis_ready`, `backup_outcome`, `watch_triggered`, `archive_imported`. A class token outside the known set SHALL be readable-and-preserved verbatim, never rejected, and SHALL re-serialize byte-identically. Token grammar SHALL be snake_case segments of at most 32 bytes. The payload SHALL carry `class_registry_version`, an integer greater than zero, identifying which vocabulary the producer used, so a consumer can tell preservation from recognition. A registry version of zero SHALL be refused by the Rust layer while the JSON Schema layer still accepts it, matching the generated-schema lower-bound rule.

#### Scenario: Every known class parses to its named variant

- **WHEN** each of the six version-1 class tokens is deserialized
- **THEN** it maps to its named variant and re-serializes to the same token

#### Scenario: An unknown class token is preserved, not rejected

- **WHEN** a payload carries a well-formed class token this build does not know
- **THEN** the Rust type accepts it, reports it as unrecognized, preserves the exact token, and a full payload round trip re-emits it unchanged

#### Scenario: A malformed class token stops processing

- **WHEN** a class token violates the grammar (uppercase, over-length, or empty)
- **THEN** both the JSON Schema layer and the Rust layer reject the payload

#### Scenario: A zero registry version is refused below the schema

- **WHEN** a payload declares `class_registry_version` 0
- **THEN** the Rust layer rejects it while the JSON Schema layer accepts it, per the lower-bound rule

### Requirement: Recipients use the shared tenancy grammar

The recipient SHALL be carried as a closed tenant reference spelled `user:<canonical UUID>`. Any other owner kind or a non-canonical UUID SHALL be rejected. Correlation references (`operation_ref`, `analysis_ref`) SHALL be open-vocabulary `<kind>:<local_id>` pointers, opaque to this crate: the crate fixes their carrier grammar, never their referent kinds, and both are optional.

#### Scenario: A non-user recipient kind is refused

- **WHEN** the recipient is spelled with an owner kind other than `user`
- **THEN** both the JSON Schema layer and the Rust layer reject the payload

#### Scenario: Correlation references stay opaque

- **WHEN** a payload correlates through `operation:<uuid>` and `analysis:<uuid>` references
- **THEN** both round trip unchanged, with no kind-level interpretation imposed by this crate

### Requirement: Delivery hints are advisory wire facts

The payload MAY carry a `priority_hint` from the closed vocabulary `low`, `normal`, `high`, and a `quiet_hours` hint expressed as two offsets from UTC midnight in seconds, each within 0..=86_399 and unequal to each other, wrap-around permitted when the window crosses midnight. Both members are advisory: absence means the consumer applies its own defaults, and no enforcement behaviour is specified by this contract. Equal start and end offsets SHALL be refused by the Rust layer while the JSON Schema layer accepts them; an out-of-range offset SHALL be refused by both layers.

#### Scenario: Hints absent means consumer defaults

- **WHEN** a valid payload omits `priority_hint` and `quiet_hours`
- **THEN** it parses with both hints absent and re-serializes without them

#### Scenario: A degenerate quiet-hours window is refused below the schema

- **WHEN** `quiet_hours` carries equal start and end offsets
- **THEN** the Rust layer rejects it while the JSON Schema layer accepts it, because property inequality is beyond draft 2020-12

#### Scenario: An out-of-range quiet-hours offset is refused everywhere

- **WHEN** either offset exceeds 86_399 seconds
- **THEN** both the JSON Schema layer and the Rust layer reject the payload

### Requirement: Producers author nothing through the extension channel

A producer constructing a notification payload SHALL leave `extensions` empty: every value a producer intends a consumer to read is a typed field of this contract, and the channel exists only for relaying unknown keys from producers built later. The published schema SHALL advertise the preserve policy for additive evolution.

#### Scenario: A constructed payload serializes no extension keys

- **WHEN** a producer builds a payload without touching `extensions`
- **THEN** the serialized form carries no member beyond the documented fields

#### Scenario: Unknown additive members from a later producer survive a relay

- **WHEN** a payload carrying an undocumented member is parsed and re-serialized
- **THEN** the member survives byte-identically under the preserve policy
