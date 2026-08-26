## ADDED Requirements

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
