# Design: define-block-kind-extension-procedure

## Context

Version one of the Document IR is a two-variant `#[non_exhaustive]` enum (`heading`, `paragraph`) serialized with `tag = "kind"` and `deny_unknown_fields`; the generated JSON Schema closes the `kind` enum the same way. The content digest is SHA-256 over the canonical JSON of `blocks` alone. Two repositories sit on either side: ratatoskr-extractor produces, ratatoskr-knowledge consumes. The coordination evidence cited in the proposal was collected on 2026-08-26 against extractor main `b2bf875` and knowledge main `e35de67`.

Constraints that shape the design: the workspace store spec `document-ir` already owns the cross-repository rule ("version one is the shared intersection"; "anything a single service needs and no other reads stays inside that service"), so this repository records only its own governance on top of that rule, citing it rather than restating it. Development status is binding: one version only, no v2, no deprecation windows; only the repository owner changes that status.

## Goals / Non-Goals

**Goals:**

- One written procedure that makes the next kind proposal mechanical: named proposer, named acceptor, enumerated evidence, fixed rollout order, explicit rejection stance for readers.
- Pin the loud-rejection premise with an executable fixture so the procedure's compatibility reasoning rests on a tested fact, not prose.
- Keep every generated artifact reproducible after the doc-comment pointer change.

**Non-Goals:**

- No new kinds or attributes; none has landed demand (see proposal).
- No change to knowledge's match patterns or to any consumer repository in this change.
- No decision on variant-level `#[non_exhaustive]`; it is recorded as a follow-up because it would break knowledge's fixture construction sites and deserves its own coordinated change.

## Decisions

1. **ADR, not a spec delta and not an ARCHITECTURE section.** The procedure governs how this repository's documents evolve — process, not wire behaviour — and `openspec/specs/` starts empty by policy; behaviour that more than one repository sees belongs in the workspace store, which already holds the intersection rule. An ADR is where this repository has recorded exactly such decisions before (ADR-0007 identifier grammar, ADR-0008 extensions ruling). ARCHITECTURE.md S6.1 keeps its sentence but cites the ADR, mirroring how ADR-0008 corrected S5.5 without moving the decision into S5.5. Alternative considered: a local capability spec under `openspec/specs/` — rejected because process text is not system behaviour and bulk spec growth here is explicitly forbidden.

2. **Proposer = producer, acceptor = consumer, both named repositories.** Extractor proposes because only a landed extraction path can produce honest evidence; knowledge accepts because S6.1's intersection rule gives it a veto: a kind knowledge does not render is content knowledge silently loses (its catch-all arms drop unknown kinds from context, search text and citation eligibility). Alternatives: owner-proposes (too far from the need), any-service-proposes (would let service-private shapes leak into the shared intersection, which the store spec forbids).

3. **Evidence list is concrete and checkable.** A proposal must carry: the landed producer conversion path (file references, not plans); the named knowledge consumption site (file and what rendering/indexing/citation decision reads the kind); valid fixtures round-tripping parse–serialize–parse identity; invalid fixtures declaring the rejecting layer per `fixtures/invalid-expectations.toml`; compatibility classification via `cargo contracts compat`; a digest statement (kinds are part of `blocks`, so adding or reordering kinds moves `content_digest`); and a knowledge acceptance note. This mirrors what AGENTS.md already demands of every contract change, specialized to block kinds.

4. **Unknown kinds are rejected loudly at both layers, and the procedure says so.** This is the load-bearing premise: a producer cannot emit a kind ahead of consumer adoption without breaking readers, which is why the evidence bar exists and why rollout order matters. It is pinned by the new invalid fixture (`rejected_by = ["json_schema", "serde"]`, error contains `unknown variant`). Preservation channels (`extensions`) exist on envelopes, not on blocks, per ADR-0008's authoring-channel rule.

5. **Versions move additive-only within major 1; major 2 is out of reach while dev status holds.** New kinds append to the `#[non_exhaustive]` enum; new attributes on existing kinds are optional fields with documented authority, unit and nullability. Both are additive payload-major-1 changes. A second major requires an owner decision against the binding development-status rules, which currently forbid it entirely — the procedure states this rather than planning around it.

6. **Rollout order: contracts first, then consumers learn, then producers emit.** Because readers reject unknown kinds, the only safe order is: (1) contracts lands kind + fixtures + regenerated artifacts; (2) each consuming repository repins, extends its match arms, and demonstrates consumption; (3) the producing repository repins last and emits. Attribute additions additionally require the consuming repository's field-exhaustive patterns to gain `..` at step 2 — knowledge currently has three such sites, recorded verbatim in the ADR so the next proposal names them instead of rediscovering them.

7. **Doc-comment pointer on `DocumentBlock`, accepting regeneration churn.** ADR-0008 set the precedent: the generated description should point at the governing decision so the schema self-documents its governance. Cost is one regenerated schema pair verified byte-deterministic by the existing gate.

## Risks / Trade-offs

- [The fixture could drift from the real serde/schema message] → the expectations entry pins the substring `unknown variant`, produced by both layers today; the gate fails if either layer stops rejecting, which is exactly the tripwire wanted.
- [Procedure could rot if the first real proposal contradicts it] → the ADR carries an explicit amendment clause: amend by superseding ADR, never by silent edit, matching `docs/adr/README.md`.
- [Naming knowledge's three pattern sites hard-codes another repository's line numbers into this repo's docs] → the ADR names the sites as of their pinned rev and marks them volatile facts owned by the knowledge changeset; the procedure depends on the *rule* (patterns must be non-exhaustive-ready), not on those line numbers.

## Migration Plan

Documentation lands with the branch; nothing deploys. Rollback is reverting the branch. Generated artifacts are regenerated inside the change and proven deterministic by the gate, so no follow-up repair exists.

## Open Questions

None. The coordination gate resolved the only scope question (procedure only vs. procedure plus kinds).
