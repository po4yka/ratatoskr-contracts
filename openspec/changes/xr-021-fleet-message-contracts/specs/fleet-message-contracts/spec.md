## Purpose

Defines the behaviour of the typed messages, builders and validators that the XR-021 fleet fixes add to the contracts workspace, as observable inside this repository.

## ADDED Requirements

### Requirement: Operation reports are validated against the status invariants

`OperationReported::validate` SHALL reject `failed` without an error, `succeeded` with an error, and `partially_succeeded` with neither a warning nor an error.

#### Scenario: A partial report without a diagnostic is rejected
- **WHEN** a report with status `partially_succeeded`, no warnings and no error is validated
- **THEN** validation fails with `PartialWithoutDiagnostic`

### Requirement: The AI-archive receipt binding has one definition

The `platform_receipt` module SHALL export the receipt path, media type, header names, capability path, capability name and incomplete-import warning, and SHALL accept a capability document only for the named service and only when it lists the receipt capability.

#### Scenario: A capability document for another service is rejected
- **WHEN** a capability document names `claude` and is checked for `chatgpt`
- **THEN** the check answers false

### Requirement: Conversation content digests are verified

`AiConversation::verify_content_digest` SHALL fail with `ContentDigestMismatch` when a message was edited after the digest was computed, and the added and updated payload validators SHALL call it.

#### Scenario: An edited message is detected
- **WHEN** a message body changes and the conversation digest is not recomputed
- **THEN** `AiConversationAdded::validate` fails

### Requirement: Channel digest manifests have stable canonical bytes

`ChannelDigestManifest::to_canonical_bytes` SHALL emit compact, key-sorted JSON, and `from_canonical_bytes` SHALL reject bytes that are not the canonical form of the value they parse to.

#### Scenario: Reordered bytes are an integrity failure
- **WHEN** a manifest serialized with its keys in another order is parsed with `from_canonical_bytes`
- **THEN** the call fails with `ManifestIntegrity`

### Requirement: A content capture command carries exactly one source

`ContentCaptureRequested` SHALL accept a payload with exactly one of a bounded http or https `url` and a sha256 `blob`, and SHALL reject both and neither.

#### Scenario: A payload with both sources is rejected
- **WHEN** a payload carries a `url` and a `blob`
- **THEN** decoding fails

### Requirement: Social capture reports have one construction

The social capture report builders SHALL produce the queued, preserved and unavailable reports with the documented codes and retryability, and `report_envelope` SHALL wrap them in the canonical event envelope.

#### Scenario: A deleted post is a non-retryable failure
- **WHEN** `unavailable_report` is called with `Deleted`
- **THEN** the report is `failed` with code `social.source.deleted` and `retryable = false`

### Requirement: The live message registry matches the fleet table

`contracts.toml` SHALL list the producers and consumers of each message added by this change as the fleet table states, and `platform.operation_reported` SHALL list exactly the services that emit it.

#### Scenario: The registry rows are checked
- **WHEN** the registry test loads `contracts.toml`
- **THEN** each documented row matches its (type, kind, producers, consumers)
