# Design: add-social-source-contracts

## Context

The repository generates JSON Schema from canonical Rust types via `contractsc` (ADR-0001). A new contract family therefore means: a crate, registry entries, `contracts.toml` declarations, fixtures with expectations, and regenerated artifacts — all under a gate that lints field names, timestamp vocabulary, and fixture secrecy. `docs/ARCHITECTURE.md` S7 sketches the semantics; `AGENTS.md` adds the Instagram/Threads authority rule and the X bookmark-timestamp rule. Producers are three repositories; the consumer is Knowledge.

## Goals / Non-Goals

Goals: one crate holding every social-source wire type; zero new identifier grammar; full gate compliance; fixtures that prove both compatibility directions.

Non-Goals: provider clients, OAuth, HTTP, Knowledge consumers, Ratatoskr-side collection management (no current consumer changeset needs it), media bytes.

## Decisions

### D1: No new identifier grammar

`SocialSourceId` is produced by the existing `uuid_newtype!` macro (canonical lowercase hyphenated UUID; ADR-0007 clause 1), exactly like `DocumentId`. Provider post ids and folder ids reuse [`EntityLocalId`](../../../crates/identifiers/src/entity.rs) — already "opaque, case-sensitive, provider-minted". Author handles get a small validated string (`^[A-Za-z0-9._]{1,64}$`) because no external specification fixes a handle grammar across X/Instagram/Threads (ADR-0007 clause 3 does not apply); it is an attribute, not a domain pointer. The sync checkpoint cursor gets a validated-string type bounded to printable ASCII without control characters: cursors are provider-defined opaque continuation tokens (base64, JSON fragments); any tighter grammar would break on provider evolution. Neither is an identifier of a Ratatoskr record, so clause 2's `<kind>:<local_id>` rule does not apply.

### D2: Open tokens vs closed enums

`Platform`, `SocialMediaKind` and `SocialRelationKind` are open validated strings (snake_case token grammar shared with event-type segments): new platforms, media kinds and relation kinds must not break a running consumer, and nothing dangerous can be misread from an unrecognized token rendered generically — a consumer skips an unknown relation or renders an unknown media kind generically while keeping the record. `AcquisitionMethod`, `SavedAuthority`, `CaptureCompleteness`, `UpstreamAvailability` are closed enums rejected at parse, following the `OperationStatus` precedent (`DOMAIN.md` invariant 6, "rejected explicitly"): misreading acquisition or authority is precisely the Instagram-bookmarks bug `AGENTS.md` forbids, and guessing retention from an unknown availability state is worse than stopping. Alternative considered — `EntityKind`-style enums with an `Other(String)` arm — rejected because a named "other" arm invites consumers to write a fallback branch that silently guesses semantics.

### D3: Snapshot shape is flat; capture facts live beside record facts

One struct `SocialSourceSnapshot` carries record facts (identity, author, text, media, relations, folders, digests) beside capture facts (acquisition, authority, completeness, availability, checkpoint, warnings), mirroring `OperationSnapshot`. Events carry the whole snapshot — state-carried transfer makes at-least-once redelivery idempotent, as with `OperationProgressed`. Two payloads (`SocialSourceCaptured`, `SocialSourceUpdated`) because the captured/updated distinction is the fact Knowledge reacts to (new index entry vs re-index), even though both carry the same body type.

### D4: One cross-field invariant, enforced by a hand-written `Deserialize`

Partial captures must explain themselves: `completeness = partial` requires at least one warning. Like the operation snapshot's invariants I1–I5 this is cross-field, so serde has no hook; the public struct derives only `Serialize` and parses through a private mirror struct. Deliberately **not** invariants: `published_at <= captured_at` (provider clocks skew; enforcing would reject honest data), and `authoritative_platform_state => official_api` (an X data export also carries authoritative platform state).

### D5: Fixtures cannot contain URLs or handles-with-@ — by design

The secret/PII scanner bans `https://` and `@name` everywhere under `fixtures/**` (`tools/contractsc/src/secrets.rs`), and weakening it is not on the table. Therefore committed valid fixtures omit the optional `permalink` and use handle spellings without the `@` prefix (the canonical form providers expose as screen names). Field coverage that fixtures cannot express — permalink, display names, checkpoints containing URL-ish characters — is covered instead by the Rust-level drift-guard round-trip test that constructs a snapshot carrying every field, in the style of the operation snapshot's test O-2.

### D6: Governance metadata choices

- `[services].known` gains `ratatoskr-instagram` and `ratatoskr-threads`; producers/consumers lists then name real deployables.
- Contract owner is `ratatoskr-x`: the metadata model allows one accountable owner, X is the first producer and the family's origin, and joint governance across the three social repos runs through the workspace changeset either way.
- `[lint].timestamp_property_names` gains `published_at` (authority `provider_authored` — never fabricated; the X bookmark rule lives here) and `captured_at` (authority `observed`). Each gets a `[[contract.field]]` entry, which L4 requires.
- Entity kinds used by fixtures (`social_source`, `user`, `operation`, `event`) are all already in `[entity_kinds].known`; no widening.
- Privacy class is `user_content` for all three contracts (post text, alt text, author names); classification `internal` (service-to-service).

### D7: Compatibility fixtures without a fake history

Ratatoskr is in development and these contracts are born at v1; there is no older major to freeze. The backward direction (`new-consumer-old-producer/`) holds the minimal first-day shape (every optional member omitted) rather than inventing a `v0` artifact; the forward direction (`old-consumer-new-producer/`) holds a payload carrying unknown additive members that this build must preserve. Fixture names say what they are instead of borrowing a version number that never existed.

## Risks / Trade-offs

- [Closed enums force a consumer upgrade when a producer adds a variant] → That is the intent for authority/lifecycle semantics; expansion is additive at the wire level and classified by `cargo contracts compat`, and replay remains possible after upgrade.
- [Flat snapshot grows with every future capture fact] → Extensions preserve additive growth; a genuinely second axis would justify splitting the struct then, with a compatibility report, not before.
- [Permalink validation is a lower bound (HTTPS + no whitespace)] → Full URL syntax is producer-side validation; the schema documents the bound honestly.
- [Fixture-less permalink coverage] → Mitigated by D5's drift-guard test asserting every documented member appears on the wire.

## Migration Plan

Purely additive: new crate, new schemas, new fixtures; no existing type or artifact changes except registry/metadata/doc status. Rollback is reverting the commit; nothing consumes these contracts yet. No database exists behind them (development status), so there is no data migration.

## Open Questions

None.
