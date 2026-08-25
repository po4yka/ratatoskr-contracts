# backup-policy Specification

## Purpose
Defines the wire agreement through which GitHub states which repositories must be preserved and at what depth (the versioned desired-backup-policy document) and Vault answers with auditable acknowledgment events, including the drift rules that decide what a policy version covers when the repository catalog changes between versions.

## Requirements

### Requirement: Policy versions are monotonic and start above zero

The desired-backup-policy document SHALL carry a `policy_version` that is an integer greater than zero, and each successive published policy for the same estate SHALL have a strictly greater `policy_version` than the one it replaces. A candidate version equal to or lower than the previous version SHALL be detectably invalid, as shall a version of zero.

#### Scenario: A strictly increasing successor is valid

- **WHEN** a policy with `policy_version` 4 is validated against a previously applied version of 3
- **THEN** the succession check passes

#### Scenario: An equal or lower version is refused

- **WHEN** a candidate `policy_version` equals or is lower than the previously applied version
- **THEN** the succession check fails with a distinct non-increasing error

#### Scenario: A zero version never parses

- **WHEN** a policy document whose `policy_version` is 0 is deserialized
- **THEN** the Rust layer rejects it while the JSON Schema layer still accepts it, matching the lower-bound rule for generated schemas

### Requirement: Each policy names repositories explicitly

Every per-repository entry SHALL identify its repository using the shared pointer grammar (`repository:<canonical UUID>`), and SHALL carry a mirror cadence class from a closed vocabulary, a priority hint from a closed vocabulary, an optional size hint expressed in bytes, and explicit exclusions. Two entries in one policy SHALL NOT name the same repository.

#### Scenario: A full entry survives a serialization round trip

- **WHEN** a committed fixture carrying cadence class, priority hint, size hint in bytes and exclusions is serialized and deserialized
- **THEN** every field returns unchanged and no field is dropped

#### Scenario: A duplicated repository entry is refused

- **WHEN** a policy lists two entries naming the same `repository:<uuid>` reference
- **THEN** the Rust layer rejects it; the JSON Schema layer accepts it because array-of-uniques is beyond draft 2020-12

#### Scenario: An unknown cadence class stops processing

- **WHEN** an entry carries a cadence class outside the closed vocabulary
- **THEN** both the schema layer and the Rust type reject the document

### Requirement: Policy generation metadata is attributed

The policy document SHALL name the producing deployable service and the instant the document was produced. The production instant SHALL use the canonical UTC representation, be required on the wire, and be documented as producer-asserted.

#### Scenario: A non-canonical production instant is refused

- **WHEN** `produced_at` carries fractional seconds that render away (`...:00.000Z`)
- **THEN** the Rust layer rejects the spelling while the schema pattern permits the fraction

### Requirement: Vault acknowledges each policy version over the canonical envelope

Vault SHALL answer a received policy version with exactly one registered event type, `vault.backup_policy.acknowledged.v1`, carried inside the canonical event envelope. The payload SHALL state the acknowledged policy version, an outcome of `accepted` or `rejected`, machine-actionable rejection reasons when the outcome is rejected, and the last policy version Vault had fully applied before this decision. The aggregate identifier of the acknowledgment SHALL name the policy itself as `backup_policy:<version>`.

#### Scenario: An accepted acknowledgment travels typed inside a real envelope

- **WHEN** an accepted payload is bound into a parsed minimal envelope and the envelope is re-serialized and reparsed
- **THEN** the event type matches the payload's registered type, no field is dropped on the wire, and the payload comes back equal to what was sent

#### Scenario: Rejection always explains itself

- **WHEN** an outcome of `rejected` is deserialized with an empty reason list
- **THEN** the Rust layer rejects it; an `accepted` outcome with any reason is rejected the same way

#### Scenario: A reason's repository reference appears only when the reason needs one

- **WHEN** a rejection reason of code `repository_unknown_in_catalog` omits its repository reference, or another code carries one
- **THEN** the Rust layer rejects the mismatch in either direction

#### Scenario: A zero acknowledged version never parses

- **WHEN** an acknowledgment naming `acknowledged_policy_version` 0 is deserialized
- **THEN** the Rust layer rejects it while the JSON Schema layer accepts it, matching the lower-bound rule

#### Scenario: Acceptance implies forward progress

- **WHEN** an `accepted` outcome names an `acknowledged_policy_version` at or below `last_applied_policy_version`
- **THEN** the Rust layer rejects the payload, because accepting an already-superseded version is indistinguishable from a replay

#### Scenario: A stale acknowledgment stays representable

- **WHEN** Vault re-answers an old version it has already superseded
- **THEN** the payload may say `rejected` with code `policy_version_not_monotonic` while `last_applied_policy_version` exceeds the acknowledged version, and the payload parses

### Requirement: Exclusion expressions are carrier-safe

Every exclusion expression SHALL be a non-empty string of at most 256 bytes containing no control characters, regardless of which scope it targets.

#### Scenario: A control character in an expression is refused

- **WHEN** an exclusion expression carries a C0 control character or DEL
- **THEN** the Rust layer rejects the document while the schema layer accepts it, because character-class bans beyond length are below draft 2020-12's expressive power here

### Requirement: Drift between policy and catalog follows stated rules

The contract SHALL express mid-version drift as pure decision rules: a catalog repository not named by the current policy version SHALL be treated as out of scope until a future version names it; a policy entry naming a repository absent from the catalog SHALL surface as a reportable drift rather than being silently skipped; and an exclusion SHALL only ever narrow the mirrored set.

#### Scenario: Uncovered catalog repositories are enumerable

- **WHEN** the catalog holds a repository that no entry of the current policy names
- **THEN** computing coverage against that catalog lists the repository as not covered

#### Scenario: Entries pointing outside the catalog are enumerable

- **WHEN** an entry names a repository reference missing from the catalog snapshot
- **THEN** computing drift against that snapshot reports the entry's reference as unknown to the catalog

#### Scenario: An exclusion never widens scope

- **WHEN** an entry's exclusions are applied to the entry's mirrored set
- **THEN** the result is a subset of the set before exclusions were applied
