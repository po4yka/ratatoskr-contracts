# ADR-0002: Event naming and major-version strategy

> Status: Accepted  
> Last reviewed: 2026-08-18

## Context

`README.md`, `AGENTS.md` and `ARCHITECTURE.md` S9.1 all give the event-name grammar `<bounded_context>.<aggregate>.<action>.v<major>`, and all three list examples. None of them states the alphabet, whether `v0` is legal, or what forces a major bump.

The load-bearing gap is different: `ARCHITECTURE.md` S5.2 shows an envelope that carries **both** a `schema_version` field and an event type ending in `.v<major>`, and never says what each one versions. Two version axes in one object with no stated meaning is a compatibility hazard, because a consumer will key on one of them and be wrong.

## Drivers

- Names that a machine can check, so a malformed event type cannot reach a topic.
- Two version axes that do not silently conflate.
- Expand / migrate / contract (`README.md`, `AGENTS.md`, `ARCHITECTURE.md` S10) with a mechanical classification of what is breaking.
- Forward compatibility for a consumer built today against a producer released later.

## Options

The options below are recorded for the `schema_version` question, because that is the consequential one.

### Option A — `schema_version` mirrors the `event_type` major

The field repeats the payload major that the name already carries.

For: the redundancy is checkable, and a v2 payload is visibly v2 in two places.

Against: the envelope then has no version axis of its own, so an envelope major bump would need a new required field, and adding a required field is itself a breaking change.

### Option B — `schema_version` is the envelope major (chosen)

For: it is not redundant; it gives the envelope the one thing it otherwise cannot express; and it lets a consumer refuse an envelope whose structure it cannot parse, instead of half-interpreting it.

Against: the field is the constant `1` in every v1 message, and a reader who assumed option A will be surprised.

## Decision

1. `event_type` is carried on the wire as a validated `EventType`, never as a bare `String`.

2. **Grammar.** Three snake_case segments, each matching `^[a-z][a-z0-9_]{0,31}$`, then `.v<major>` where `<major>` matches `[1-9][0-9]{0,3}`. Maximum 128 UTF-8 bytes. `v0` and leading zeros are unparseable: there are no draft contracts on the wire. Segments use snake_case with no hyphens, because every example in `README.md` and S9.1 is snake_case. **Entity kinds** use a different, hyphen-permitting alphabet, because `README.md` shows the kind `x-post`. Keeping the two alphabets distinct is honest to the documents; inventing a union of them is not.

3. **The canonical form is byte-exact.** Parsing an event type and then rendering it returns the input, or the input was rejected.

4. **The action segment is past tense** (`AGENTS.md` principle 9: "Events represent completed facts … Do not name a request as if it already happened"). This is enforced by `cargo contracts check` over *registered* event types, and **not** by `EventType::parse`. A consumer must not fail to read a producer's event because of English grammar, and grammatical tense is not decidable in general. Governance belongs in the repository, not in the runtime parser.

5. **Two independent version axes.**
   - The `.v<major>` suffix of `event_type` versions the **payload** contract. Source: `ARCHITECTURE.md` S5.2, "Payloads are versioned independently through `event_type` major versions."
   - The envelope's `schema_version` versions the **envelope** itself. It is `1` today, and `EnvelopeSchemaVersion` refuses any other value at parse time.
   - A new payload major mints a new `event_type`. The old one keeps flowing until the contract phase.
   - A new envelope major is a system-wide event and requires a workspace changeset.

6. **Renaming an event type is a breaking change** (`AGENTS.md`: "Do not rename existing event types for style"). `EventType::with_major` is the only sanctioned mutation, and it changes the version suffix only.

7. **What forces a new major.** This restates `ARCHITECTURE.md` S9.2 and `AGENTS.md` in the form the `contractsc compat` classifier checks: a required property added, removed or renamed; an optional property becoming required; a narrowed `type`, `format`, `pattern`, `const`, enumeration set or numeric bound; an enumeration variant added **or** removed, because consumers of these enumerations are exhaustive (see point 8); `additionalProperties` going from permitted to forbidden; and `x-ratatoskr-unknown-policy` going from `preserve` to `reject`.

   There is one rule no tool can see: S9.2's "semantic reinterpretation requires a new version even if JSON shape is unchanged". Two things catch it. First, the mandatory `authority` and `unit` entries in `contracts.toml`: changing either is a visible diff in a reviewed governance file. Second, human review, which is where `AGENTS.md` puts the obligation. A green `compat` run is a structural result and never a semantic guarantee.

8. **Unknown-field and unknown-variant policy, per type**, because that policy is what decides whether a change is additive or breaking:
   - Envelope, error, warning, snapshot and result **fields** are *preserved* into an `extensions` map (`README.md`: "consumers ignore unknown additive fields"), so adding an optional field is additive.
   - `EntityKind` is *preserved* in an `Other` variant, so a bounded context added in milestones 5–7 does not break a consumer built today.
   - `OperationStatus`, `EnvelopeSchemaVersion` and the kind of `TenantRef` are *rejected explicitly*, because a consumer that guesses at a lifecycle state, at an envelope structure or at a data owner is worse than one that stops. Adding an `OperationStatus` variant is therefore a major bump.

   Together these are the two branches of `DOMAIN.md` invariant 6, chosen per type and stated rather than left to chance.

9. `vN` and `vN+1` coexist. Nothing in milestones 1–4 removes anything.

## Consequences

- `schema_version` carries no per-message information in v1. That is correct for a structural version, and it is the price of option B.
- Under either reading of the two options, every milestone 1–4 artifact and fixture is byte-identical, because every major is 1. No committed byte depends on this decision today. Reversing it costs one constant, one error variant and one fixture — but only until the first release. After a release, a consumer that keyed on the field is wrong either way, so the reading must be confirmed before then.
- `CorrelationId(Uuid)` from `ARCHITECTURE.md` S5.1 survives as a mintable identity, while the envelope's `correlation_id` slot is an `EntityRef`, so `operation:`, `command:` and future kinds fit without an envelope major bump. This reconciles S5.1 with S5.2 and with `README.md`'s non-UUID `x-post:123`, and it is the most consequential reading in this work. It needs explicit sign-off before any producer ships.
- Command contracts (`ARCHITECTURE.md` S5.3, outside milestones 1–4) will need a parallel `CommandType` whose third segment is imperative. `EntityKind` is already open, so `command:` references need no change here.
- The past-tense rule is a repository gate, so a producer in another repository can emit a present-tense name and this repository will parse it. That is deliberate: reading a producer's event must not depend on English grammar.

## Security / privacy

Event names are public identifiers. They must not encode tenant or user identity, and the grammar gives them nowhere to hide one: three fixed segments and a version.

The closed kind set of `TenantRef` is recorded here as the "rejected explicitly" case. The reason is authorization, not tidiness: a consumer that cannot understand who owns the data must not process the record. `SECURITY.md` requires security review for identity fields, and this is that review's outcome.

## Compatibility / migration

Nothing is published, so there is nothing to migrate. Every rule above is enforced from the first commit, so no contract can be created that violates it. The compatibility fixtures under `fixtures/**/compat/` freeze both directions from day one: `old-consumer-new-producer/` proves this build accepts and re-emits what a later producer sends, and `new-consumer-old-producer/` proves this build still reads the first shape.

## Validation

Test identifiers refer to the test matrix in the implementation specification:

- E-2, E-3, E-4: every documented example parses, and a malformed name is rejected for a named reason.
- E-10, E-11: a future envelope major is refused at parse, while an unknown field within the current major is tolerated. The two axes behave differently, exactly as this ADR says.
- E-12, E-13: `family()` is stable across payload majors, and `with_major` changes the version only.
- I-12, I-14, O-5: the preserved branch for `EntityKind`, and the rejected branch for `TenantRef` and `OperationStatus`.
- L-7, M-8: registered event actions are past tense, and the registered `event_type` agrees with the payload type's own constant and with the declared major.
- B-1 … B-18: each rule in point 7 is exercised against a real committed schema, including the fail-closed case for a keyword no rule models.
- Q-6: property test — well-formed names round-trip with the right major, malformed names are rejected.

## Follow-up

- Confirm the `schema_version` reading before the first release. It is free to reverse today and breaking afterwards.
- Add `CommandType`, with the imperative-segment rule, at whichever milestone introduces commands.
- ADR-0004 formalises point 8 across the whole repository.
