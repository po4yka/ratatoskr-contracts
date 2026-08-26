# ADR-0010: Document IR block-kind extension procedure

> Status: Accepted
> Last reviewed: 2026-08-26

## Context

`ARCHITECTURE.md` S6.1 states that version one of the Document IR is "the shared intersection used by Extractor and Knowledge" and that more block kinds are added only when both sides need them. The store spec `document-ir` in `ratatoskr-workspace` owns the cross-repository rule behind that sentence: anything a single service needs and no other reads stays inside that service. Neither document records how the gate operates: who proposes a kind, who must accept it, what evidence the proposal carries, or in what order the repositories may adopt it.

The canonical shape is a two-variant `#[non_exhaustive]` enum (`heading`, `paragraph`) serialized with `tag = "kind"` and `deny_unknown_fields`, and the generated JSON Schema closes the `kind` vocabulary the same way. Readers therefore reject an unknown kind discriminant at both layers instead of preserving it, which `fixtures/content/document/invalid/unknown-kind.json` pins as of this ADR. Blocks carry no preservation channel; ADR-0008's `extensions` ruling is envelope-scoped and does not reach into `DocumentBlock`.

A coordination check against both sides of the contract on 2026-08-26 found no landed work that demands a new kind, which is why this ADR ships only the procedure:

- ratatoskr-extractor main (`b2bf875`): the direct PDF adapter walks pages but emits one plain paragraph per page and its archived design records that trade-off as deliberate, noting block refinement "can land later without contract changes because blocks stay ordered Paragraphs"; the YouTube transcript adapter keeps segment timing in an extractor-owned sidecar under a spec requirement that published blocks contain no timing fields; degraded browser-worker outcomes terminate runs through typed failure classes and resolution-step rows without producing a document event.
- ratatoskr-knowledge main (`e35de67`, pinned to contracts rev `d56c689`): both block-matching sites end in catch-all arms, nothing deserializes contract types at runtime, so new kinds compile cleanly and degrade silently — unknown-kind content is dropped from LLM context, search text and citation eligibility.

## Drivers

1. `ARCHITECTURE.md` S6.1/S6.2 promise a two-sided gate for kinds and provenance spans but name no procedure, so every extractor-side need restarts a negotiation from zero.
2. The store spec `document-ir`: version one is the shared intersection; single-service needs stay service-private.
3. `AGENTS.md` review requirements: every contract change names owning context, producers, consumers, field authority, compatibility classification, rollout order, and evidence.
4. Development status is binding: one version only — no v2, no parallel majors, no deprecation windows. Only the repository owner changes that status.
5. Readers reject unknown kinds and unknown fields loudly at both layers (fixture-pinned), so adoption order between repositories is not optional politeness; it is the only safe sequence.

## Options

### Option A — leave the gate unwritten; negotiate per change

Against: driver 1. The first disagreement has no tiebreaker, and the sidecar/PDF history shows the practical outcome: needs get parked in service-private carriers rather than negotiated, or forced through without consumer evidence.

### Option B — encode the procedure as a local OpenSpec capability spec

Against: process text is not system behaviour; `openspec/specs/` here starts empty by policy and grows from behaviour changes, and the cross-repository facts already live in the store spec, which this repository cites rather than restates.

### Option C — one ADR with pointers from S6.1/S6.2 and the canonical type (chosen)

For: matches how this repository already records governance decisions (ADR-0007 identifier grammar, ADR-0008 extensions rule), keeps the decision reviewable and supersable, and makes the gate discoverable from both the architecture document and the generated schema description.

## Decision

1. **Proposer: ratatoskr-extractor.** Only a landed extraction path can produce honest evidence of demand. A proposal is filed as a workspace changeset naming this repository, per `AGENTS.md`'s cross-repository workflow.

2. **Acceptor: ratatoskr-knowledge, with a real veto.** S6.1's intersection rule means a kind knowledge does not render is content knowledge silently loses — dropped from context, search text, embeddings and citation eligibility by its catch-all arms. Knowledge accepts by naming its consumption site, not by acknowledging receipt.

3. **Evidence bundle.** A kind or attribute proposal carries all six, and none is waivable:
   - the landed producer conversion path (code and tests on an extractor branch, deterministic conversion, budgets stated);
   - the named knowledge consumption site (file references plus what rendering, indexing or citation decision reads the value);
   - valid fixture documents exercising every new discriminant and attribute through parse–serialize–parse identity;
   - invalid fixtures declaring the rejecting layer in `fixtures/invalid-expectations.toml`;
   - compatibility classification produced with `cargo contracts compat`, plus the re-blessed `compat/api/ratatoskr-document-contracts.txt` baseline, since appending a variant changes the crate's public API surface;
   - a digest statement: `content_digest` covers the canonical JSON of `blocks` alone, so kinds and attributes live inside hashed bytes — newly produced documents digest differently from the day the producer emits the extension, previously stored documents keep the digests computed at their extraction, and any reordering or retitling moves digests by construction.

4. **Version movement.** Within payload major 1 the only permitted moves are additive ones: appending variants to the `#[non_exhaustive]` enum and adding optional fields with documented authority, unit, nullability and timestamp semantics, passing the repository lint vocabularies. A second major is out of reach while development status holds — there is no v2 to propose against until the owner changes that status.

5. **Adoption order: contracts, then consumers, then the producer.** Because readers reject unknown kinds and fields at both layers, the only safe sequence is:
   1. this repository lands the extension with fixtures and regenerated artifacts;
   2. every consuming repository repins, extends its match arms, and demonstrates consumption — for attribute additions this includes making its patterns non-exhaustive-ready (`..`) where they currently bind every field;
   3. the producing repository repins last and starts emitting.
   As of knowledge's pin `d56c689` the field-exhaustive sites are `context.rs` lines 125 and 129 and `search.rs` line 48; those citations are volatile facts owned by the knowledge changeset, and the binding rule is the pattern-readiness requirement, not the line numbers.

6. **Standing refusals.** Layout or visual IR, table and image deep structures, and provider-specific block shapes stay out absent landed two-sided demand. Single-sided demand is refused to service-private carriers — the transcript timing sidecar is the precedent, not a grievance. Vague names (`status`, `data`, `metadata`, `timestamp`) are rejected by the repository lint before review sees them.

7. **Amendment.** This procedure changes by superseding ADR, never by silent edit, matching `docs/adr/README.md`.

## Consequences

- The next kind proposal is mechanical: file the workspace changeset, attach the six-item bundle, land in the decision-5 order. Nothing about today's wire shape changes.
- The loud-rejection premise is now executable fact rather than prose: the invalid fixture fails the gate if either layer ever stops refusing unknown kinds, which is the earliest possible warning that this ADR's reasoning has rotted.
- The generated schema description for `content.document` points here, so a consumer reading only generated artifacts still meets the governance rule.
- Proposals get slightly heavier and negotiations disappear; that trade is the point.

## Security / privacy

A new kind is a new container for arbitrary content and inherits the repository's classification duties: the proposal states privacy classification, marks user-content-bearing fields for retention and redaction design, and passes the secret/PII scan over its fixtures automatically through the gate. Kinds that would carry raw provider payloads face the same bar as every other contract — references over bodies unless a consumer demonstrably needs content.

## Compatibility / migration

Nothing is published, so this repository migrates nothing. The decision-5 order replaces migration windows while development status forbids parallel majors: a fleet cannot run two shapes of major 1 side by side, so each extension lands as one contracts revision that consumers adopt before producers flip. Reverting an accepted-but-unconsumed extension is a normal additive reverse under the same order; removing one after consumers depend on it is breaking and waits for the owner's version-status decision.

## Validation

- `fixtures/content/document/invalid/unknown-kind.json` with its `invalid-expectations.toml` entry: rejected by `json_schema` and `serde` for containing `unknown variant`, accepted by no layer beyond them, asserted in both directions by `cargo contracts check` and `cargo test --workspace`.
- The gate's determinism and drift steps prove the regenerated schema description bytes are exactly the generator's output.
- CI's `compatibility` job holds the public-API baseline against silent variant additions: a kind landed without re-blessing `compat/api/ratatoskr-document-contracts.txt` fails there.

## Follow-up

1. **Knowledge pattern readiness precedes any attribute addition.** The three field-exhaustive match sites named in decision 5 must gain `..` (or knowledge adopts variant-level tolerance) in a knowledge changeset before any proposal adds an optional attribute to `heading` or `paragraph`. A coordinated decision on variant-level `#[non_exhaustive]` belongs to that same change: it would make future attribute additions compile-neutral for consumers but breaks exhaustive construction sites, including knowledge's own fixtures, so it is deliberately not taken here.
2. **Extractor documentation drift.** Extractor's `docs/ARCHITECTURE.md` section 8.2 promises "page-aware text blocks" and "page-level provenance"; the landed PDF adapter deliberately delivers neither, and the prose predates the implementation by five days. Fixing that text is an extractor-docs change, cited here so the discrepancy is on record rather than folklore.
3. **Known future candidate.** Transcript timing at the boundary is the named deferral from the YouTube transcript design ("if Knowledge ever needs timing at the boundary, that is a coordinated contracts changeset"). It arrives through this procedure when knowledge names the use — not before.
