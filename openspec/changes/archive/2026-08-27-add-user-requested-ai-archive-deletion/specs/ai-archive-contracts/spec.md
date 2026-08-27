## ADDED Requirements

### Requirement: Owner-requested deletion is an authoritative tombstone reason

`AiArchiveTombstoneReason` SHALL accept and preserve exactly `user_requested` in addition to its existing provider, compliance, and reconciliation reasons. The token SHALL mean that the authenticated owner explicitly requested Ratatoskr-held deletion; it SHALL NOT claim provider-side deletion or grant revocation. The existing `ai_archive.subject.tombstoned.v1` payload shape and event type SHALL remain unchanged.

#### Scenario: User-requested conversation tombstone round-trips

- **WHEN** a tombstone fixture names one conversation, carries `reason = "user_requested"`, and supplies immutable non-sensitive deletion evidence
- **THEN** the typed payload and generated schema accept it, round-trip it without loss, and retain the existing v1 event type

#### Scenario: Existing reasons remain valid

- **WHEN** fixtures use `provider_deletion_event`, `compliance_event`, or `reconciliation_policy`
- **THEN** every existing fixture continues to validate and serialize identically

#### Scenario: Unknown reason remains invalid

- **WHEN** a tombstone carries a reason outside the closed declared vocabulary
- **THEN** typed deserialization rejects it rather than treating an unknown authority as valid deletion evidence
