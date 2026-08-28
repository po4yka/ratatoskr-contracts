## 1. Crate and shared values

- [x] 1.1 RED: add `crates/channel-digest-contracts/tests/values.rs` with compiling tests for canonical channel usernames, owner/run/subscription/manifest/result references, language, closed-open UTC windows, and trigger vocabularies; run the focused test and confirm it fails because the crate/types do not exist.
- [x] 1.2 GREEN: add `ratatoskr-channel-digest-contracts` to the Cargo workspace with the minimal existing identifier/envelope/schema dependencies and implement the bounded values/validators; run the focused test through `build-gate -- cargo test --locked -p ratatoskr-channel-digest-contracts --test values` and confirm it passes.
- [x] 1.3 RED: extend `values.rs` with invalid username, equal/reversed window, scheduled-occurrence, count, reference, and length tables; run it and confirm at least one invalid cross-field value is accepted.
- [x] 1.4 GREEN: add hand-written semantic validation and closed error codes without echoing input values; rerun `values` and confirm every invalid table case is refused.

## 2. Subscription and run commands

- [x] 2.1 RED: add `commands.rs::subscription_command_is_owner_scoped_replay_safe_and_content_free`; assert the exact command type, required owner/operation/idempotency/desired-state fields, locator/reference exclusivity, and forbidden credential/body members, then run it and confirm the payload is absent.
- [x] 2.2 GREEN: implement `ChannelDigestSubscriptionSetRequested` on the canonical command envelope and make the focused command test pass.
- [x] 2.3 RED: add `commands.rs::run_command_encodes_exact_trigger_and_window_semantics`; cover on-demand and scheduled valid examples plus missing/foreign occurrence and unbounded selector failures, then run it and confirm the payload is absent or accepts an invalid case.
- [x] 2.4 GREEN: implement `ChannelDigestRunRequested` and its closed trigger/window invariants; rerun the focused command tests and confirm they pass.

## 3. Knowledge recap facts

- [x] 3.1 RED: add `recap.rs::request_names_immutable_manifest_without_source_content`; assert owner/run/operation, manifest digest/reference, counts, language, and window while proving source bodies, provider IDs, URLs, prompts, and credentials are structurally absent; run it and confirm the request type is absent.
- [x] 3.2 GREEN: implement `KnowledgeChannelDigestRecapRequested` on the command envelope and make the focused request test pass.
- [x] 3.3 RED: add completion/failure tests covering complete/partial count consistency, stable result linkage, coverage/warnings, empty-window exclusion, safe closed failure classes, and mutual exclusivity; run them and confirm the facts are absent or an invalid state passes.
- [x] 3.4 GREEN: implement `KnowledgeChannelDigestRecapCompleted`, `KnowledgeChannelDigestRecapFailed`, their closed outcomes, and semantic validators; rerun the recap tests and confirm they pass.

## 4. Registry, generated artifacts, and compatibility

- [x] 4.1 RED: add `tools/contractsc/tests/channel_digest.rs` asserting the new service/crate/root types, exact subjects, producer/consumer sets, authority/units/privacy declarations, and generated outputs; run it and confirm `contracts.toml` lacks the registrations.
- [x] 4.2 GREEN: register the service, entity kinds where required, contracts, fields, and justified waivers in `contracts.toml`; update the generator registry and run the focused registry test until it passes.
- [x] 4.3 RED: add positive, negative, privacy, round-trip, and two-direction compatibility fixtures for every command/fact; run `cargo contracts check` and confirm it fails because fixtures/generated artifacts are missing.
- [x] 4.4 GREEN: add fixtures, generate deterministic JSON Schemas and TypeScript declarations from the Rust authority, and rerun `cargo contracts check` until generation, lint, secret scan, provenance, and compatibility pass.

## 5. Verification and publication

- [x] 5.1 Update contract README/API documentation and the consumer pin instructions; no failing test applies to prose, so verify every named subject/type/path against generated artifacts and run `git diff --check`.
- [x] 5.2 Run the exact `DEVELOPMENT.md` gate through the machine build gate where compiler-backed, plus `openspec validate add-channel-digest-contracts --strict`; inspect public API and generated diffs and record the observed results before publication.
- [x] 5.3 Publish/merge Contracts before any consumer or producer pin changes, then verify the remote commit and hosted checks; this is rollout evidence, not a substitute for the local gate.
