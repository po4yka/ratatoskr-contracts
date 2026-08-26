# Proposal: define-block-kind-extension-procedure

## Why

`ARCHITECTURE.md` S6.1 says version one of the Document IR is "the shared intersection" and that more block kinds are added only when both sides need them, but nothing records who proposes a kind, who must accept it, what evidence the proposal carries, or how versions move. Without a written procedure every extractor-side need becomes an ad-hoc contracts negotiation, and the first candidate kinds (PDF page provenance, transcript timing, degraded-extraction markers) are exactly the cases where extractor and knowledge have so far chosen service-private carriers instead.

A coordination check against both consuming repositories on 2026-08-26 (extractor main `b2bf875`, knowledge main `e35de67`) found no landed work that demands a new kind: the direct PDF adapter deliberately emits one plain paragraph per page and its archived design states block refinement needs no contract change; the YouTube transcript adapter carries segment timing in an extractor-owned sidecar under a spec requirement that published blocks contain no timing fields; degraded browser-worker outcomes terminate runs without a document event. Knowledge today matches blocks only through catch-all arms and never deserializes contract types. This change therefore delivers the procedure only and invents no speculative kinds.

## What Changes

- Add ADR-0010, "Document IR block-kind extension procedure", recording: the proposer (ratatoskr-extractor), the acceptor (ratatoskr-knowledge), the evidence a proposal must carry (a landed producer conversion path, a named knowledge consumption site, round-trip fixture documents, invalid fixtures with declared rejection layers, compatibility classification, digest interaction), the unknown-kind rejection stance readers take at both layers, the additive-only-within-major-1 rule, and the rollout order between the repositories.
- Point `ARCHITECTURE.md` S6.1's intersection sentence and S6.2's provenance sentence at ADR-0010 so the procedure is discoverable from the architecture document.
- Point the canonical `DocumentBlock` doc comment at ADR-0010, which changes generated schema description bytes; regenerate all artifacts deterministically.
- Add one invalid fixture, `fixtures/content/document/invalid/unknown-kind.json`, pinning the procedure's loud-rejection premise: a `"kind"` discriminant this build does not know is rejected by the JSON Schema layer and the serde layer. The fixture pins existing behaviour; it changes no wire shape.
- Record, in the ADR's follow-up, the two coordination findings that are real but out of scope here: knowledge's three field-exhaustive match patterns (`context.rs:125`, `context.rs:129`, `search.rs:48`) that make any future attribute addition to `heading`/`paragraph` compile-breaking for knowledge until they gain `..`, and extractor `docs/ARCHITECTURE.md` S8.2 prose promising "page-level provenance" the landed implementation deliberately does not deliver.

Not in scope: new block kinds, new attributes on existing kinds, layout or visual IR, table/image deep structures. None has landed cross-repository demand.

## Capabilities

### New Capabilities

None. The procedure is repository governance recorded as an ADR; the cross-repository facts it cites — version one is the shared intersection, single-service needs stay service-private — already live in the ratatoskr-workspace store spec `document-ir`, which this change cites rather than restates.

### Modified Capabilities

None. No wire behaviour changes; the fixture pins rejection behaviour that both layers already exhibit.

(skip_specs is set because no spec-level behaviour changes: documentation plus one evidence fixture.)

## Impact

- `docs/adr/0010-document-ir-block-kind-extension-procedure.md` (new) and `docs/adr/README.md` (index entry).
- `docs/ARCHITECTURE.md` S6.1/S6.2 (two pointer sentences).
- `crates/document-contracts/src/document.rs` doc comment on `DocumentBlock`; regenerated `schemas/json-schema/content/document.v1.schema.json` and matching TypeScript declaration.
- `fixtures/content/document/invalid/unknown-kind.json` plus an entry in `fixtures/invalid-expectations.toml`.
- No producer or consumer repository changes in this change. Future kind proposals will name their own changesets in those repositories per the procedure.
