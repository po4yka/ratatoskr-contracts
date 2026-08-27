## Context

The active workspace change `add-social-browser-capture-contract` owns the fleet boundary. This
repository already supplies `BrowserExtension`, `ExplicitUserCapture`, and social availability
vocabularies, but lacks both the canonical command envelope that gives Platform a typed request and
the exact client-visible outcome taxonomy.

## Goals / Non-Goals

**Goals:**

- Give every inter-service command one envelope contract while keeping event and command semantics
  distinct.
- Reuse established provenance types while closing only provider routing and outcome codes.
- Generate and validate Rust, JSON Schema, fixtures, and TypeScript from one canonical source.

**Non-Goals:**

- HTTP route design, provider URL canonicalization, source persistence, or client rendering.

## Decisions

### D1: Add one canonical command envelope beside the event envelope

Commands are requests and events are facts, so `CommandEnvelope` and `EventEnvelope` remain
separate types. The command envelope mirrors the proven event-envelope invariants where they apply:
a typed UUID command identity, validated command type, issue time, producer, aggregate, correlation
and optional causation/tenant references, a fixed schema major, object payload, and preserved
additive extensions. `CommandPayload` supplies the same typed set/payload access as `EventPayload`.

`CommandId` is added to the shared identifiers crate so the command's own identity cannot be
mistaken for an operation or event. Idempotency remains a domain concern in the payload: the envelope
identity deduplicates delivery while the request idempotency key deduplicates user intent.

### D2: Bind the social request to that envelope

`social.capture.requested.v1` is a command, not a `social.source.*` fact: the former asks an owner
to acquire a permalink and the latter says a normalized source exists. Reusing the facts would make
unavailable attempts look like preserved content.

### D3: Reuse existing provenance vocabularies

The command references existing `BrowserExtension` and `ExplicitUserCapture` values instead of
parallel enums. This retains their established authority meaning and avoids vocabulary drift.

### D4: A closed social taxonomy maps onto, but does not weaken, `ErrorCode`

`SocialCaptureOutcomeCode` lives with the social domain, has exactly three variants, and maps each
variant to the general `ErrorCode` carried by `ErrorEnvelope` or `WarningEnvelope`. The shared
`ErrorCode` remains deliberately open for other domains; it must not pretend to reject unknown
social spellings. Consumers that need social semantics parse the closed type first, while messages
remain safe display strings rather than protocol fields.

### D5: Commands are a first-class generated family

The generic registry can render any root type, but it only has semantic metadata checks for events.
Registering `social.capture.requested.v1` as a generic schema would lose the type-to-payload binding
that makes a command truthful. `contractsc` therefore gains a `commands` family and a
`[contract.command]` declaration mirroring the existing event registration: the command type major
must match the contract major and the registered payload must implement `CommandPayload` with the
same static type. The common `CommandEnvelope` remains a separate core root schema because its
payload is intentionally polymorphic.

## Risks / Trade-offs

- [A downstream consumer has not adopted the command] → Platform capability-gates that provider
  until its consumer is deployed.
- [A future command drifts from the common shape] → each command implements `CommandPayload`, and
  registry/schema tests cover envelope-to-payload binding rather than duplicated command wrappers.
- [A command is generated without a payload binding] → the `commands` metadata family checks its
  `CommandPayload::COMMAND_TYPE` before the registry can generate artifacts.
- [The command becomes a dumping ground for provider detail] → schema permits only the declared
  minimal fields and rejects session/page data by absence.

## Migration Plan

Publish this additive contract and generated artifacts first. Platform and social services adopt it
next; browser support is enabled only after those consumers are verified. Existing social facts are
not replaced, and no version negotiation or parallel major is introduced.
