# social-source-contracts Specification

## Purpose
Defines the wire behaviour of normalized social-source records published by `ratatoskr-x`, `ratatoskr-instagram` and `ratatoskr-threads` and consumed by `ratatoskr-knowledge`: record and author identity, saved-state authority, media by reference, native folder membership, capture completeness with checkpoints, and the `social.source.captured` / `social.source.updated` events that carry them.

## Requirements

### Requirement: A snapshot carries its own identity and points at provider identity explicitly

A social-source snapshot SHALL carry a Ratatoskr-side `social_source_id` as a bare canonical lowercase UUID, a `platform` token, and the provider-minted external post id as an opaque local identity. The permalink, when present, SHALL be an absolute HTTPS URL. A consumer SHALL be able to deduplicate records using `platform` plus `external_post_id` without contacting any provider.

#### Scenario: one capture is one stable identity

- **WHEN** a producer emits a captured event and later an updated event for the same source
- **THEN** both payloads carry the same `social_source_id`, and both parse as canonical UUIDs

#### Scenario: a non-canonical UUID is refused

- **WHEN** a payload carries a `social_source_id` in any spelling other than the canonical lowercase hyphenated form
- **THEN** deserialization fails with a pattern error naming the type

#### Scenario: provider identity survives round trip

- **WHEN** a snapshot whose `external_post_id` is a provider slug such as a numeric string is serialized and parsed again
- **THEN** the value returns byte-identical

### Requirement: Saved-state authority is explicit and closed

Every snapshot SHALL state how it was acquired (`AcquisitionMethod`) and what authority the saved-state claim has (`SavedAuthority`). Both vocabularies SHALL be closed: an unknown value SHALL be rejected at parse, never guessed at. An Instagram or Threads explicit capture SHALL be representable only as `explicit_user_capture`, never as authoritative platform membership; `authoritative_platform_state` SHALL be reachable only where the platform itself exposes saved state through a supported channel.

#### Scenario: an unknown acquisition method stops processing

- **WHEN** a payload carries `"acquisition": "carrier_pigeon"`
- **THEN** both the JSON Schema layer and the Rust layer reject it with an unknown-variant error

#### Scenario: an explicit capture is never authoritative platform state

- **WHEN** an Instagram capture obtained through a share-style flow is emitted
- **THEN** its `saved_authority` is `explicit_user_capture`, and no field in the payload asserts membership in a provider-native saved list

### Requirement: Media travels by reference, never by bytes

A media item SHALL describe itself with a media-kind token, a `BlobRef` naming owner service, digest, media type and byte length, and optional alt text. No contract in this capability SHALL carry image, video or other media bytes.

#### Scenario: a media item resolves through a blob reference

- **WHEN** a consumer reads a snapshot with two media items
- **THEN** each carries a `BlobRef` resolvable under the workspace blob-reference spec, and the payload contains no base64 or byte array anywhere

#### Scenario: an unknown media kind is preserved

- **WHEN** a future producer emits a media item whose kind token this build does not know
- **THEN** the snapshot parses, and re-emitting it reproduces the unknown token verbatim

### Requirement: Native folder membership is distinct from capture authority

A snapshot MAY list memberships in provider-native folders, each carrying the provider folder id and optionally the provider-authored folder name. Folder membership SHALL NOT imply or require any particular `saved_authority`, and a snapshot with no folders SHALL be valid.

#### Scenario: bookmark folders ride along without changing authority

- **WHEN** an X bookmark snapshot lists two native folders obtained through the supported API
- **THEN** the memberships carry provider folder ids, `saved_authority` remains `authoritative_platform_state`, and the record round-trips losslessly

#### Scenario: a folder-less capture is complete

- **WHEN** an Instagram capture carries no folder memberships
- **THEN** the snapshot parses with an empty folder list

### Requirement: A quote, reply or repost names its target post

A relation SHALL name its kind (quote, reply, repost) and the target post's provider external id on the same platform. A snapshot MAY carry zero or more relations.

#### Scenario: a quoted post is addressable

- **WHEN** a snapshot declares one quote relation
- **THEN** the relation carries the quoted post's external id and parses back identically

### Requirement: Capture completeness is declared, and partial captures explain themselves

A snapshot SHALL declare whether the capture is `complete` or `partial`. A partial capture SHALL carry at least one warning identifying what was not captured. A complete capture MAY still carry warnings that did not reduce completeness.

#### Scenario: a partial capture must warn

- **WHEN** a payload declares `"completeness": "partial"` with an empty warning list
- **THEN** the Rust layer rejects it, naming the invariant

#### Scenario: a partial capture with evidence is accepted

- **WHEN** a media download failed during capture and the payload declares partial completeness with that warning
- **THEN** the snapshot parses and the warning survives a round trip

### Requirement: A checkpoint references where a sync may resume

A snapshot MAY carry one opaque sync-checkpoint cursor produced by the capturing service's sync run. The cursor SHALL be treated as opaque by consumers: bounded printable text with no control characters, never interpreted, never rewritten.

#### Scenario: a checkpoint survives every hop

- **WHEN** a snapshot carrying a checkpoint cursor is serialized, parsed, and re-serialized
- **THEN** the cursor returns byte-identical, including characters outside any provider grammar this repository knows

#### Scenario: a control character cannot hide in a checkpoint

- **WHEN** a payload carries a checkpoint containing a newline
- **THEN** deserialization rejects it

### Requirement: Upstream availability is observed, not assumed

A snapshot SHALL carry the observed upstream availability of the source (`available`, `unavailable`, or `deleted_upstream`) as a closed vocabulary. An unavailable or deleted source MAY still carry a complete capture of what was obtained before it went away.

#### Scenario: a deleted post keeps its captured content

- **WHEN** a source was fully captured and later observed deleted upstream
- **THEN** `upstream_availability` is `deleted_upstream`, `completeness` may remain `complete`, and the text and media references are untouched

#### Scenario: an unknown availability state is refused

- **WHEN** a payload carries `"upstream_availability": "shadowbanned"`
- **THEN** both layers reject it rather than letting a consumer guess a retention policy

### Requirement: Captured and updated events are facts inside the common envelope

`social.source.captured.v1` SHALL mean a source became part of a user's library; `social.source.updated.v1` SHALL mean an existing source's normalized record changed. Both payloads SHALL carry the whole snapshot (state-carried transfer), implement the envelope crate's payload contract, and travel only inside an envelope whose `event_type` matches the payload. Event actions SHALL be past tense.

#### Scenario: the real payload travels inside a real envelope

- **WHEN** a captured payload is set into an event envelope and read back through `payload_as`
- **THEN** the typed payload equals the original, the envelope's `event_type` is `social.source.captured.v1`, and no member of the payload is dropped

#### Scenario: an updated event re-publishes the full record

- **WHEN** a producer emits an updated event after a refresh
- **THEN** the payload alone is sufficient to index the source without consulting any earlier event

#### Scenario: a mismatched event type cannot be read as a social payload

- **WHEN** `payload_as` requests a social payload from an envelope carrying a different event type
- **THEN** the read fails with a payload-type error

### Requirement: Old and new producers interoperate without coordination

A consumer built against this version SHALL accept a snapshot or event payload carrying additive members it does not know, preserving them verbatim on re-emission, and SHALL accept the minimal first-day shape that omits every optional member.

#### Scenario: a newer producer adds an optional member

- **WHEN** a payload contains a top-level member this build has never heard of
- **THEN** the payload parses, the member lands in the preserved extension map, and re-emission loses nothing

#### Scenario: the minimal first-day payload still parses

- **WHEN** a payload carries only the members that were required on day one
- **THEN** every optional member defaults to absent and the payload parses

### Requirement: An absent author means unknown authorship, never no author

A snapshot MAY omit `author` entirely. Absence SHALL mean that the producing service could not observe any author account for the source; it SHALL NOT be interpreted as a claim that the source has no author. A snapshot that omits the author SHALL otherwise be a complete record: parsing it SHALL succeed, and re-emitting it SHALL keep the member absent rather than substituting a placeholder.

#### Scenario: an authorless snapshot parses losslessly

- **WHEN** a payload carries a well-formed snapshot with every required member present but `author` omitted
- **THEN** deserialization succeeds with no author value, and re-emitting the parsed snapshot reproduces the payload without introducing an author or dropping any other member

#### Scenario: an old payload with an author still parses

- **WHEN** a producer built before this change emits a snapshot whose `author` is fully populated
- **THEN** the current type parses it identically to before

### Requirement: A removed fact states that a library stopped holding a source

`social.source.removed.v1` SHALL mean the user's Ratatoskr library no longer holds the source named by `social_source_id`. The payload SHALL carry the library identity, the owner whose library dropped it, a removal reason from the closed vocabulary (`user_requested`, `retention_policy`), and the producer-clock instant of removal. The fact SHALL NOT assert anything about upstream availability, and an unknown reason value SHALL be refused at parse.

#### Scenario: a privacy deletion is expressible

- **WHEN** a producer removes a source because its user asked for deletion
- **THEN** it publishes `social.source.removed.v1` carrying that source's identity, the owner, `reason = "user_requested"`, and a removal instant, inside an envelope whose event type matches the payload

#### Scenario: an unknown removal reason stops processing

- **WHEN** a payload carries `"reason": "cache_eviction"`
- **THEN** both the JSON Schema layer and the Rust layer reject it with an unknown-variant error
