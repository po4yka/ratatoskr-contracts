# Design: add-notification-contracts

## Context

The repository is Rust-first (ADR-0001): canonical shape lives in `crates/*/src/**.rs`, `schemas/` and `generated/typescript/` are written by `cargo contracts generate`, and `contracts.toml` is governance metadata validated against reality by rule set R1-R12 in `tools/contractsc/src/metadata.rs`. Eight contract crates exist; the newest (`backup-contracts`) is the structural template this change mirrors: a payload type implementing `EventPayload`, hand-written checked `Deserialize` where cross-field rules exist, closed enums marked `#[non_exhaustive]`, an open-preservation enum pattern already proven by `EntityKind`, and fixtures whose rejections are declared per layer in `fixtures/invalid-expectations.toml`.

Discovery is explicit: `tools/contractsc/src/registry.rs` enumerates root types in `root_types!` and event types in `event_payload_types()`; adding a crate means touching both, plus workspace membership and a `[[contract]]` row, which is what makes the generator able to see the type at all.

## Goals / Non-Goals

**Goals:**

- One documented bus surface (`platform.notification.raised.v1`) that four producer services can emit and Telegram's sender can consume without either importing the other's types.
- A class taxonomy that grows without breaking deployed consumers: unknown tokens preserved, registry version explicit.
- Wire shapes that reuse the existing identifier grammar end to end — no new identifier conventions invented here.
- Advisory hints carried honestly as hints: no enforcement semantics on the wire.

**Non-Goals:**

- Delivery guarantees beyond at-least-once; dedupe and preference filtering belong to Telegram.
- Email/webhook channels, preference storage, UI rendering.
- A command envelope: none exists yet in this repository, and introducing one is a separate decision.

## Decisions

### D1: One event type for all classes, not one per class

Six-plus classes could each be an event type (`platform.notification.analysis_ready.v1`, ...), but that forces every new class through a schema registration, fixture suite and consumer release even when only the label changed, and it scatters the addressing/hint grammar across many schemas. One type with a versioned taxonomy keeps the notification *envelope* stable while the vocabulary evolves — the same trade `EntityKind` made for aggregate kinds. The class remains machine-actionable (closed known set + preserved unknowns), so consumers still branch precisely.

Alternative rejected: separate event types per source fact (operation.progressed etc.) with Telegram subscribing to everything — that couples Telegram's subscription list to every producing context and gives it no single documented surface to honor preferences "through".

### D2: Fact naming — `platform.notification.raised.v1`

`AGENTS.md` principle 9: events represent completed facts. By the time a producer emits this, the judgment "this user should be told" is complete; whether a message is actually sent is Telegram's decision (preferences, quiet hours, dedupe), so a command name ("requested", "send") would overstate the wire's authority. `raised` is past tense and passes `EventType::action_looks_past_tense()`. Bounded context `platform`: the notification bus is platform-level shared infrastructure, like `platform.operation.*`.

Aggregate identity: the raised notification itself, `notification:<uuid>`. This differs from operation events (which aggregate on the operation) because the notification is the thing consumers acknowledge/dedupe/suppress; its identity must not be borrowed from the causing fact.

### D3: Taxonomy = open-preservation enum + explicit registry version

`NotificationClass` mirrors `EntityKind` exactly: token pattern `^[a-z][a-z0-9_-]{0,31}$`, six known variants, `Other(String)` preservation, `is_known()`. Preservation (not rejection) is correct here because a class drives *presentation routing*, not state transitions: an unknown class must reach a human inbox rather than be dropped or mis-filed under a default.

The open enum alone cannot tell a consumer "recognized" from "preserved", so the payload carries `class_registry_version` (u32, floor 1). Bumping it is mandatory when the known set grows; consumers built against version 1 then know that a version-2 producer may emit tokens they will hold as `Other`. This is data-driven versioning on the wire, not a second envelope major — the payload major stays 1 because the shape does not change when the vocabulary grows.

### D4: Addressing and correlation reuse existing types verbatim

- `recipient: TenantRef` — tenancy is closed to `user:` by design (`SECURITY.md` identity rule); a notification has exactly one human recipient today, and widening tenant kinds is a reviewed schema-pattern change, not this crate's decision.
- `operation_ref`, `analysis_ref: Option<EntityRef>` — open-vocabulary pointers, opaque to this crate per the request. Named fields, not a map: two of them are precisely what producers need today (operations and knowledge analyses), names survive review, and the lint rejects vague containers. Everything else routes through the envelope's `correlation_id`/`causation_id`, which are required anyway.
- Redundancy note: when a notification belongs to an operation, `operation_ref` should equal the envelope's `correlation_id`. The duplication is deliberate — the payload stays self-contained for consumers that process bodies without envelope context — and is documented, not enforced (cross-layer rules are unenforceable here).

### D5: Advisory hints

- `priority_hint: Option<NotificationPriority>` — closed enum `low|normal|high`, mirroring `BackupPriorityHint`'s precedent: a guessed priority silently reorders delivery, so unknown stops processing; adding a level later is additive. Absent = Telegram's default ordering. Enforcement stays in Telegram; the field is a producer-asserted urgency judgment.
- `quiet_hours: Option<QuietHoursHint>` — `{start_offset_seconds, end_offset_seconds}`, each 0..=86_399, unequal, wrap-around allowed (start > end crosses midnight). Seconds-since-UTC-midnight satisfies the unit-suffix lint, avoids inventing a time-of-day string format, and keeps timezone interpretation where the knowledge lives: Telegram holds preference/timezone storage, the producer only states the window it was told about or infers. Equal bounds are ambiguous (empty day vs full day), hence refused; a producer that knows nothing omits the member.

Both hints sit outside any cross-field coupling: they constrain nothing else, so `NotificationRaised` needs no mirror-based `Deserialize` *for their sake*; `QuietHoursHint` validates internally via a private-field constructor plus checked conversion, and its manual `JsonSchema` publishes `minimum`/`maximum` so out-of-range offsets fail at both layers (only the inequality is Rust-only).

### D6: `NotificationId` lives in `ratatoskr-identifiers`

Every typed own-identity UUID lives there (`DocumentId`, `OperationId`, ...); splitting one out would fork the grammar. Additive expansion of that crate's public API requires re-blessing `compat/api/ratatoskr-identifiers.txt` via `cargo contracts api-write` and creating the new crate's baseline — mechanical, reviewed through the baseline diff.

### D7: Privacy classification `user_content`

Title/message are user-facing summaries about a specific user's data and are delivered onward to external infrastructure (Telegram); `boundary_metadata` would understate the retention/redaction design work owed to that flow. Carrier safety comes free from `SafeMessage` (no control characters, 1024-char ceiling), which structurally blocks forged log lines and stack traces in notification text. Producers are documented to keep message text summary-level; secrets and raw provider content travel by blob reference elsewhere, never here (`AGENTS.md` principle 8).

## Risks / Trade-offs

- [Single event type concentrates traffic] → Mitigation if needed later: partitioning by class at the bus layer is Telegram/platform territory; the wire shape does not preclude it.
- [Unknown-class preservation lets typos through silently] → Same answer as `[entity_kinds].known`: fixtures are checked against a known-token allowlist in `contracts.toml`; a typo'd class in a committed fixture fails the build while the wire type stays open.
- [`title`/`message` could leak sensitive detail to a third party] → `SafeMessage` bounds the carrier; classification `user_content` forces the retention conversation; docs require summary-level text; secret scan runs over all fixtures.
- [Registry version could drift from the known set] → The version constant and the enum live in one module with one test asserting the pairing; a bump task is spelled out in tasks.md for future expansions.

## Migration Plan

Purely additive: no existing schema changes, no consumer migrates. Rollout order is expand-only — land the contract, then producers adopt at will, then Telegram consumes. Rollback is "stop emitting"; nothing persisted depends on the type yet. Compatibility evidence: family is additive by construction; `cargo contracts compat` identity baseline recorded like the backup-policy change did.

## Open Questions

None blocking. Timezone-aware quiet hours, digest suppression windows, and per-channel fan-out are Telegram-side behaviours this contract deliberately cannot see.

## Compatibility evidence

Recorded at apply time (task 5.2):

- `cargo contracts compat schemas/events/platform.notification.raised.v1.schema.json <same>` reports `compatible: no contract difference` - the identity baseline for an additive-by-construction family member; there is no prior schema to diff against.
- `cargo contracts api-check` reports every contract crate unchanged against `compat/api/` after the 5.1 re-bless: `ratatoskr-identifiers` gained only `NotificationId` and its impls (additive), and `ratatoskr-notification-contracts` froze its first baseline.
- Producers: `ratatoskr-knowledge`, `ratatoskr-github`, `ratatoskr-vault`, `ratatoskr-x`. Consumer: `ratatoskr-telegram`. Rollout is expand-only per design.md Migration Plan.
