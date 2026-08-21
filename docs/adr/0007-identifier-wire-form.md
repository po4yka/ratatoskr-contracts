# ADR-0007: Identifier wire form — bare typed UUIDs and qualified references

> Status: Accepted  
> Last reviewed: 2026-08-21
> Supersedes the identifier bullet in [ADR-0002](0002-event-naming-and-major-version-strategy.md) §Consequences, which stated the reconciliation for one field and left the general rule implicit.
> Amended when milestone 5 replaced the old string `BlobRef` with a structured reference. `BlobRef` is no longer part of the identifier rule.

## Context

`ARCHITECTURE.md` S5.1 declares typed UUID newtypes, `pub struct CorrelationId(pub Uuid);` among them. Four lines of `ARCHITECTURE.md` S5.2's normative envelope example are namespaced strings — `"correlation_id": "operation:018f…"`, `"aggregate_id": "document:018f…"`, `"causation_id": "event:018f…"`, `"tenant_id": "user:018f…"` — while `"event_id"` in the same example is a **bare** UUID. `README.md` shows `"aggregate_id": "x-post:123"`, whose local part is not a UUID at all.

Three documents, three shapes, and no stated rule. ADR-0002 reconciled them for `correlation_id` only, and recorded that the reading "needs explicit sign-off before any producer ships". This ADR is that sign-off, and it states the rule rather than the instance.

A defect was found while writing it. `EntityRef::parse` accepted an uppercase-UUID local part — `event:018F0000-…` matches the published `pattern` — while `EntityRef::as_uuid` and `EventId::try_from(&EntityRef)` accept only the canonical lowercase form. One event therefore had two unequal wire references, and the `causation_id` → `EventId` join returned a misleading `NotAUuid` for a reference to a real event.

## Drivers

1. Every documented example must be simultaneously legal. A rule that forbids one of them is not a reconciliation, it is a change nobody agreed.
2. A pointer's referent kind must be readable from the value alone, because pointer fields are polymorphic and consumers route on them.
3. `event_id` is the at-least-once deduplication key (`ARCHITECTURE.md` S15.6). A dedup key wants to be 16 bytes, a `uuid` column and a `format: uuid` schema, not a string that must be split first.
4. `ARCHITECTURE.md` S5.1: "provider IDs remain opaque strings or bounded numeric wrappers".
5. One identity has one spelling. The repository already applies this to instants, to `TenantRef` and to every typed UUID.
6. Narrowing an identifier's accepted value set is free before the first release and breaking after it.

## Options

### Option A — uniform tagging (chosen for pointers only)

Every identifier carries `<kind>:`, Stripe/TypeID style.

Against: it contradicts `ARCHITECTURE.md` S5.2's bare `event_id`, which is an authority-1 documented example. `event:<uuid>` cannot carry `format: uuid`, cannot be a `uuid` column without a parse step, and cannot be handed to a dedup cache as 16 bytes. It also drags an equality-normalisation obligation onto the hottest path in the system.

### Option B — uniformly bare typed newtypes

Every identifier is a bare UUID in a typed newtype, taking S5.1 literally.

Against: the local part is not always a UUID. `README.md`'s `x-post:123` and `vault/docs/ARCHITECTURE.md`'s `"repository_id": "github:123456"` are both non-UUID. A bare polymorphic field would be an untyped opaque string with no kind and no routing.

### Option C — split by what the field does (chosen)

Own identity is bare and typed; a pointer is qualified.

For: it is the only reading under which S5.2's bare `event_id`, S5.2's four prefixed references, and `README.md`'s `x-post:123` are all legal at once. S5.1 is satisfied as written — `CorrelationId(pub Uuid)` exists with exactly that shape — it is simply not the type of the envelope's `correlation_id` slot.

Against: it is a rule with three clauses rather than one, so it must be written down, which is what this ADR does.

## Decision

**The rule.** It governs the wire fields of the contracts in **this** repository. It is descriptive of what ships. It is not asserted over identifier fields owned by other repositories, and the counterexamples in §Rejected readings are recorded rather than legislated away.

1. A field carrying the record's **own** identity is a bare canonical lowercase-hyphenated UUID in a typed newtype: `event_id` (`EventId`), `operation_id` (`OperationId`).
2. A field **pointing at** another Ratatoskr domain record is the self-describing string `<kind>:<local_id>`: `aggregate_id`, `correlation_id`, `causation_id`, `tenant_id`. The kind vocabulary is open (`EntityRef`, `EntityKind::Other` preserves an unknown kind) unless authorization requires it closed, in which case the kind set is closed and an unknown kind is rejected (`TenantRef`; ADR-0002 point 8).
3. A handle to a **non-domain external system** keeps that system's own grammar in its own validated newtype: `ErrorEnvelope.trace_id` is bare 32-hex because W3C Trace Context fixes that spelling. Clause 3 applies only where an external specification already fixes the spelling. It is not a general exemption from clause 2.

The predicate for clauses 1 and 2 is **is-a versus points-at**, not "does the kind vary today". `tenant_id` is tagged because it points at a `User` record, exactly as `aggregate_id` points at an aggregate; its closed kind set is an authorization decision (ADR-0002 point 8) and is orthogonal to whether it is tagged. The phrase "whose kind is not fixed by the schema" is struck from the crate documentation, because it was the only reason `tenant_id` looked like an exception.

**One spelling per UUID reference.** `EntityRef::parse` rejects a local part that is a UUID spelled any way other than the canonical lowercase hyphenated one, with the named error `IdentifierError::NonCanonicalUuid`. The rule fires on UUID shape alone: a provider slug and a numeric id are untouched and stay case-sensitive and fully opaque.

**Equality.** `EntityRef` equality is octet equality of the rendered `<kind>:<local_id>`. The kind is lowercase by grammar. The local part is case-**sensitive** and is never case-folded, because provider identities are case-significant (`github/AGENTS.md`: "case normalization must not collapse distinct provider identities"). The canonical-UUID rule removes a second spelling; it never merges two identities.

**Not decided here, deliberately.** `event_id` and `operation_id` stay bare. `tenant_id` stays tagged. The self-correlation convention — a root event carrying `correlation_id: event:<event_id>` — stays; `EventId::try_from(&EntityRef)` is the bridge, and the canonicality rule is what makes that bridge total. `correlation` is **not** added to `EntityKind::KNOWN` or to `contracts.toml [entity_kinds].known`: a correlation scope is not a Ratatoskr entity, no fixture carries `correlation:`, and `[entity_kinds].known` governs fixtures under a documented one-token-one-fixture discipline that an unused token would break. `CorrelationId::as_entity_ref()` therefore widens into `EntityKind::Other("correlation")`, which is the open-vocabulary branch working as designed.

## Consequences

- The envelope shape does not change. No fixture byte changes except the one new invalid fixture.
- `EntityRef::parse` accepts strictly less than before. That is the whole point, and it is why this lands before the first release.
- `EntityRef::new` takes an already-validated `EntityLocalId` and does not re-run the canonicality rule. The wire boundary is `parse` — serde routes every deserialization through it — and in-crate callers build the local part from `uuid::Uuid::to_string()`, which is canonical by construction. The rule cannot be moved into `EntityLocalId::parse`, which is generated by `wire_string_newtype!`; adding a hook to that macro for one call site would cost more than it closes.
- The published `pattern` still accepts `event:018F…`. This is the third instance of ADR-0001's recorded lower-bound consequence, alongside the cross-field invariants and one-spelling-per-instant; ADR-0001 §Consequences now names it. The gap is auditable per fixture: `fixtures/core/event-envelope/invalid/aggregate-id-uppercase-uuid.json` declares `rejected_by = ["serde"]`, which asserts in both directions that the schema accepts it and the Rust type does not. ADR-0001 explicitly refuses to close such a gap with generated `allOf`/`if-then` blocks, and this rule is conditional on the value's shape, so a draft 2020-12 encoding would be exactly that.
- Cross-language clients generated from the schema before milestone 8 will accept an uppercase-UUID reference that a Rust consumer rejects. This is ADR-0001 §Consequences' known gap ("Consumers that are not written in Rust get the schema, not the invariants, until milestone 8"), and this rule joins the list of what falls in it.

## Security / privacy

`SECURITY.md` requires security review for identity fields, parser changes and compatibility rules. This decision is all three, and this section is that review's outcome.

- **Narrowing an identity parser.** `EntityRef::parse` now rejects a value it previously accepted. The rejection is explicit and named, never silent and never a fold: a folding parser would rewrite bytes a producer sent, and a relay re-emitting a preserved envelope would then change an upstream identity. Fail-closed is the correct posture for an identity field, and it is the posture `canonical_uuid`, `TenantRef` and `WireTimestamp` already take.
- **Loss of representability.** A provider whose external identity is an uppercase UUID cannot be referenced. No provider named anywhere in the workspace mints one. If one appears, relaxing the rule is a *widening* — "relaxing validation without changing existing meaning" is backward-compatible under `AGENTS.md` — whereas adding the rule after a release would be breaking. The asymmetry is why the narrow rule ships now.
- **Disclosure through the kind token.** `EntityKind` is open and producer-controlled, so a kind token reaches every broker, log sink and, through `ErrorEnvelope.correlation_id`, a user-visible support code. Two things bound it. The grammar bounds a token to 32 characters of `^[a-z][a-z0-9_-]{0,31}$`, which is the same bound ADR-0002 §Security relies on for event names, and the same reasoning applies: a token has nowhere to hide a tenant or user identity. What a token *does* disclose is the bounded-context vocabulary, which event names already publish. Producers must not encode identity or user content in a kind token; the fixture gate (`contracts.toml [entity_kinds].known`) makes every token this repository ships a reviewed one.
- No fixture carries production or personal data; the new fixture reuses the reserved synthetic UUID block.

## Compatibility / migration

Nothing is published, so there is nothing to migrate. The change is **breaking** in the abstract — the accepted value set narrows — and free in fact, because no producer exists and every in-repository construction path already emits the canonical form.

`EntityRef::PATTERN`, `EntityKind::PATTERN` and `EntityLocalId::PATTERN` are unchanged, so `contractsc compat` reports no structural change. That is a case the classifier cannot see, and it is recorded here for the same reason ADR-0002 point 7 records semantic reinterpretation: a green `compat` run is a structural result, never a semantic guarantee.

Superseding this ADR after the first release requires a new envelope major.

## Validation

- I-1, I-2: a UUID local part, a provider local part, and a `BlobRef`-shaped local part containing further colons all parse.
- I-3: `event:018F…`, `document:018f…-A`, `user:018F…` and `x-post:018F…` are rejected with `NonCanonicalUuid`, and each is first asserted to match the published `pattern`, so the test proves *why* the Rust layer must catch it. The companion case proves `x-post:123`, `x-post:AbC-123`, `repository:OWNER.Name`, `blob:sha256:<lowercase>` and a string one hex digit short of a UUID all survive unfolded.
- I-12, I-14: the preserved branch for an unknown `EntityKind`, and the rejected branch for `TenantRef`.
- Q-2: property test over both published patterns, stated as both branches — every generated pair either parses and round-trips byte-exactly, or is a non-canonical UUID and is rejected. `EventId::parse` is the canonicality oracle, so there is no second copy of the pattern to drift.
- Q-3: every macro-generated newtype rejects everything outside its published pattern.
- F-1 … F-6, the fixture gate: `fixtures/core/event-envelope/invalid/aggregate-id-uppercase-uuid.json` is rejected by `serde` and accepted by the schema, exactly as `fixtures/invalid-expectations.toml` declares — the gate asserts both directions, so a fixture that became invalid for a different reason would fail the build.

Command surface: `cargo test --workspace --locked`, `cargo contracts check`, `cargo contracts generate`, `git diff --exit-code`.

## Rejected readings

Each of these is a real citation that the rule does not satisfy. They are recorded so the next reader does not have to rediscover them.

- **`ErrorEnvelope.trace_id` is a bare pointer.** It points at a distributed trace and is 32 bare hex characters. Under a two-clause is-a/points-at rule it would have to become `trace:4bf9…`, which would break W3C Trace Context interoperability. This is why clause 3 exists and why the rule is three clauses, not two.
- **`vault/docs/ARCHITECTURE.md`'s manifest.** The same signed manifest that supplies the strongest supporting citation — `"repository_id": "github:123456"`, a lowercase kind, a colon, a non-UUID local part, byte-compatible with `EntityRef::PATTERN` and authored without sight of this implementation — also carries `"target_id": "018f0000-0000-7000-8000-000000000001"`, a bare-UUID pointer at a separate vault record on the very next line. Vault's manifest is not a contract in this repository and this rule does not bind it.
- **`platform/README.md`'s capture response.** `{"operation_id": "018f...", "status": "accepted"}` is a bare identifier in a client-facing body. Read as a pointer it would be tagged; read as the minimal view of the operation it stays bare. Reasonable readers differ, which is precisely why the rule is stated descriptively for this repository's fields and not as a mechanical predicate applied everywhere.
- **`INTERFACES.md`'s "Public IDs are namespaced and stable".** Under the ordinary reading of "namespaced" this would require a kind prefix on `event_id`, contradicting `ARCHITECTURE.md` S5.2. `AGENTS.md` principle 4 glosses the phrase in its own next sentence — "Do not expose database sequence IDs as global identities" — which a bare UUID satisfies. This is a documentation decision, taken explicitly and not smuggled: `INTERFACES.md` is reworded to say what it governs.

## Follow-up

- Milestone 8 cross-language clients must carry the canonicality rule in generated validation, or the generated client's `README` must state that it is a lower bound. Decide it in the milestone-8 ADR.
- If a provider that mints uppercase UUIDs is onboarded, relax the rule additively rather than working around it in the producer.
