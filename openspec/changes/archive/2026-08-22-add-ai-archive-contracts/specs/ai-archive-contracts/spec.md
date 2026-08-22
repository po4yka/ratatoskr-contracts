# AI-archive contracts

## Purpose

Defines the wire behaviour of normalized AI-provider archives published by `ratatoskr-chatgpt` and `ratatoskr-claude` and consumed by `ratatoskr-knowledge`: immutable export evidence, project/conversation/message graph nodes stamped with the parser that produced them, a shared content-part grammar that preserves unknown provider records, evidence-based completeness reporting, and the `ai_archive.archive.imported` / `ai_archive.conversation.added` / `ai_archive.conversation.updated` events that carry them.

## ADDED Requirements

### Requirement: An import names its evidence and its authority

An AI-archive import SHALL carry a Ratatoskr-side `ai_archive_id` as a bare canonical lowercase UUID, an open `provider` token, the owner as a `TenantRef`, the raw provider export as a `BlobRef`, the instant of import as an observed timestamp, and the parser name/version stamps. The raw-export `BlobRef` SHALL be the immutable evidence from which every node in the import was parsed; no contract in this capability SHALL carry export bytes.

#### Scenario: one import is one stable identity

- **WHEN** a producer emits an imported event and later conversation events for the same import
- **THEN** all payloads carry the same `ai_archive_id`, and it parses as a canonical lowercase UUID

#### Scenario: the raw export is referenced, never embedded

- **WHEN** a consumer reads any payload in this capability
- **THEN** the export bytes are reachable only through the carried `BlobRef`, and the payload contains no base64 or byte array anywhere

#### Scenario: a non-canonical instant is refused

- **WHEN** a payload carries `imported_at` spelled `2026-08-17T12:00:00+02:00`
- **THEN** deserialization rejects it because only canonical UTC instants parse

### Requirement: Provider identity is explicit on every node

Every project, conversation and message node SHALL carry the open `provider` token and, where the export supplies one, the provider-minted external id as an opaque local identity. A consumer SHALL be able to deduplicate conversations using `provider` plus the external conversation id without contacting any provider. The provider vocabulary SHALL be open: an unrecognized-but-well-formed provider token SHALL be preserved verbatim, never rejected.

#### Scenario: a future provider does not break a running consumer

- **WHEN** a snapshot arrives whose provider token this build has never seen
- **THEN** every record parses, and re-emitting reproduces the unknown token byte-identically

#### Scenario: external ids survive round trip

- **WHEN** a conversation carries provider external ids for itself and its messages
- **THEN** serialization and parsing return those values byte-identical, including case

### Requirement: Conversations are graphs, not only lists

A message SHALL carry an optional parent-message reference naming another message in the same conversation, so branches, regenerated answers and edited histories survive normalization. Messages inside a conversation SHALL travel in provider presentation order. A conversation SHALL optionally reference the project it belongs to.

#### Scenario: a regenerated answer keeps its parent

- **WHEN** a conversation contains two assistant answers whose parent is the same user prompt
- **THEN** both answers carry that prompt's message id as their parent reference, and the graph parses back identically

#### Scenario: a linear conversation needs no graph edges

- **WHEN** every message of a conversation omits the parent reference
- **THEN** the conversation parses with presentation order as the only structure

### Requirement: Every node records which parser produced it

A project, conversation and message node SHALL carry parser name and version stamps identifying the parser build that normalized it. Stamps SHALL be opaque bounded tokens: consumers MAY compare them for staleness but MUST NOT parse them for semantics.

#### Scenario: a mixed-history import shows its seams

- **WHEN** an import re-parsed older conversations with a newer parser build is emitted
- **THEN** nodes parsed by different builds carry different version stamps, and each round-trips unchanged

### Requirement: One content-part grammar serves every provider

Message content SHALL be a sequence of typed parts drawn from one shared grammar: text, markdown, image, asset, citation, tool call and tool result. Both providers SHALL be modeled by this grammar plus extension points, never by two divergent schemas. An image part SHALL reference bytes through a `BlobRef`; an asset part SHALL reference stored files, artifacts or canvas-like objects through an asset kind token and a `BlobRef`. No part SHALL carry file bytes.

#### Scenario: the same grammar carries both providers

- **WHEN** a ChatGPT-shaped message with text and tool parts and a Claude-shaped message with markdown and artifact parts are emitted
- **THEN** both parse against the same content-part type with no provider-specific schema anywhere in this capability

#### Scenario: assets travel by reference

- **WHEN** a message references an uploaded file and a generated image
- **THEN** each part carries an asset-kind token and a `BlobRef`, and the payload contains no file bytes

#### Scenario: a tool call and its outcome stay linked

- **WHEN** an export records a tool invocation and its output
- **THEN** the call part and the result part carry matching tool-call identifiers, and both round-trip losslessly

### Requirement: Unknown content parts are preserved, never discarded

A part this build does not recognize SHALL parse into the unknown-part channel carrying the original JSON value verbatim, and SHALL re-serialize byte-identically through normalization and re-export. No known-part validation failure may silently convert a part into a discarded field.

#### Scenario: a future provider part survives

- **WHEN** a message carries a part whose discriminator this build does not know
- **THEN** the message parses, the unknown part lands intact, and re-emission reproduces its original members exactly

#### Scenario: a malformed known part is not quietly demoted

- **WHEN** a part declares the text discriminator but omits the text member
- **THEN** deserialization fails with a named error instead of preserving the object as unknown

### Requirement: Completeness is declared with verifiable evidence

An import SHALL carry a completeness report stating the closed completeness vocabulary (`complete`, `conversations_complete`, `structurally_partial`, `assets_partial`, `unknown`, `failed_validation`), counted totals for conversations and messages, and structured gaps. Every completeness state other than `complete` SHALL carry at least one gap naming what is missing. The reported conversation and gap counts SHALL equal the counts computable from the payload. Completeness SHALL NOT be inferred merely from every known file having parsed.

#### Scenario: an incomplete import explains itself

- **WHEN** a payload declares `structurally_partial` with an empty gap list
- **THEN** the Rust layer rejects it, naming the invariant

#### Scenario: counts cannot drift from the tree

- **WHEN** a snapshot declares conversation or gap counts that disagree with the nodes it carries
- **THEN** deserialization rejects it with a count-mismatch error

#### Scenario: complete imports may still warn

- **WHEN** an import is complete but a non-blocking parse warning was recorded
- **THEN** the snapshot parses, and the warning survives a round trip without reducing completeness

#### Scenario: an unknown completeness state stops processing

- **WHEN** a payload carries a completeness value outside the closed vocabulary
- **THEN** both layers reject it rather than letting a consumer guess whether the archive is whole

### Requirement: Imported and conversation events are facts inside the common envelope

`ai_archive.archive.imported.v1` SHALL mean one provider export finished importing as evidence; its payload SHALL carry the import head (identity, owner, evidence reference, timing, stamps, completeness report). `ai_archive.conversation.added.v1` and `ai_archive.conversation.updated.v1` SHALL mean a conversation entered the index or changed; both payloads SHALL carry the whole conversation graph plus the owning import's identity, so at-least-once redelivery is idempotent and no earlier event is needed to interpret a later one. All three payloads SHALL implement the envelope crate's payload contract and travel only inside an envelope whose `event_type` matches the payload. Event actions SHALL be past tense.

#### Scenario: the real payloads travel inside real envelopes

- **WHEN** each of the three payloads is set into an event envelope and read back through `payload_as`
- **THEN** each typed payload equals its original, each envelope carries the matching `event_type`, and no member is dropped

#### Scenario: a conversation event stands alone

- **WHEN** a consumer receives an updated event with no prior events from that import
- **THEN** the payload alone suffices to index the conversation

#### Scenario: a mismatched event type cannot be read as an archive payload

- **WHEN** `payload_as` requests an archive payload from an envelope carrying a different event type
- **THEN** the read fails with a payload-type error

### Requirement: Old and new producers interoperate without coordination

A consumer built against this version SHALL accept a snapshot or event payload carrying additive members it does not know, preserving them verbatim on re-emission, and SHALL accept the minimal first-day shape that omits every optional member.

#### Scenario: a newer producer adds an optional member

- **WHEN** a payload contains a member this build has never heard of
- **THEN** the payload parses, the member lands in the preserved extension map, and re-emission loses nothing

#### Scenario: the minimal first-day payload still parses

- **WHEN** a payload carries only the members required on day one
- **THEN** every optional member defaults to absent and the payload parses
