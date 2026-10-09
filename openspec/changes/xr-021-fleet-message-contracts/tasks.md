## 1. Operation report validation (S06 D3)

- [x] 1.1 Add `crates/operation-contracts/tests/reported.rs` tests `a_partially_succeeded_report_without_warning_or_error_fails_validation`, `a_failed_report_without_error_fails_validation`, `a_succeeded_report_with_error_fails_validation` and `a_partial_report_with_one_warning_validates` against a signature-only `validate` that returns `Ok(())`; the three rejection tests MUST fail on their assertions.
- [x] 1.2 Implement `OperationReported::validate` for invariants I2, I3 and I4 and document them on the type; the four tests pass.

## 2. AI-archive receipt binding (S06 D1, D2, D3)

- [x] 2.1 Add `crates/ai-archive-contracts/tests/platform_receipt.rs` tests `binding_constants_are_the_documented_literals`, `capability_document_round_trips_and_names_the_service`, `a_document_for_another_service_or_without_the_capability_is_rejected` and `incomplete_import_warning_is_a_valid_warning_envelope` against a scaffold with empty constants and stub bodies; each MUST fail on an assertion.
- [x] 2.2 Implement `platform_receipt` and the crate-doc section on the loopback binding; the four tests pass.

## 3. Conversation digest, tombstone and erasure documentation (S07, S12)

- [x] 3.1 Add `crates/ai-archive-contracts/tests/graph_nodes.rs::conversation_digest_is_sha256_of_canonical_messages_json` against stubs that return `Ok(())`; it MUST fail because an edited message is not detected.
- [x] 3.2 Implement `compute_content_digest`, `verify_content_digest` and `ContentDigestMismatch`, call them from the added and updated validators and correct stale fixture digests; the test passes.
- [x] 3.3 Document `AiArchiveTombstone`, `AccountErasureRequested` and `AccountErasureAcknowledged` and add the `user-requested-archive.json` fixture. This cannot start from a failing test (documentation and a fixture); `cargo contracts check` reports drift until `cargo contracts generate` runs, which is its red and green.

## 4. Channel digest manifest (S08)

- [ ] 4.1 Add `crates/channel-digest-contracts/tests/manifest.rs` tests `canonical_bytes_are_key_sorted_compact_and_stable`, `from_canonical_bytes_rejects_reordered_or_pretty_bytes_as_integrity`, `validate_rejects_unsorted_sources` and `manifest_path_and_header_constants_match_the_documented_values` against signature-only stubs; each MUST fail on an assertion.
- [ ] 4.2 Implement `manifest.rs`, the error variants, the `sha2` dependency, the `contracts.toml` row and `docs/CHANNEL_DIGEST_MANIFEST.md`; generate and bless the API baseline; the tests pass.

## 5. Read-API views (S08)

- [ ] 5.1 Add `crates/channel-digest-contracts/tests/views.rs` tests `result_view_round_trips_todays_completed_and_failed_json`, `result_summary_rejects_unknown_outcome` and `subscription_page_round_trips` against empty view types; each MUST fail on an assertion.
- [ ] 5.2 Implement `views.rs`, the rows `channel_digest.subscription_view` and `channel_digest.result_view`, generate and bless; the tests pass.

## 6. Schedule registration command (S08)

- [ ] 6.1 Add `crates/operation-contracts/tests/schedule_registration.rs` tests `registration_payload_round_trips_the_github_and_channel_digest_examples` and `registration_rejects_bad_label_and_non_object_payload` against a type whose validation accepts everything; the rejection test MUST fail on its assertion.
- [ ] 6.2 Implement `schedule_registration.rs`, the row `platform.schedule_registration_requested` with its vague-field waiver, fixtures, generate and bless; the tests pass.

## 7. Content capture command (S11)

- [ ] 7.1 Add `crates/document-contracts/tests/capture_command.rs` with the seven tests of the work order against an empty payload type so the assertions fail rather than the import.
- [ ] 7.2 Implement `capture.rs`, the registry entry, the `contracts.toml` row and fixtures with their expectations, generate and bless; the tests pass.

## 8. Document extracted event (S09)

- [ ] 8.1 Add `crates/document-contracts/tests/events.rs::document_extracted_round_trips_and_requires_extractor_owned_blob` and the envelope composition fixture case; the blob-owner assertion MUST fail against a `validate` that accepts any owner.
- [ ] 8.2 Implement `ContentDocumentExtracted`, its registry entry and `contracts.toml` row, generate and bless; the test passes.

## 9. Vault backup policy command (S09)

- [ ] 9.1 Add `crates/backup-contracts/tests/commands.rs::apply_requested_round_trips_a_desired_policy` and an invalid fixture against an empty payload; it MUST fail on an assertion.
- [ ] 9.2 Implement `commands.rs`, the export, the registry entry and the row, generate and bless; the test passes.

## 10. Social capture reports and captured_at fixtures (S10 CD1, CD4)

- [ ] 10.1 Add `crates/social-contracts/tests/capture_report.rs` comparing the builders with four golden fixtures byte for byte against builders that return a bare failed report; it MUST fail on an assertion.
- [ ] 10.2 Implement `capture_report.rs`, the dependency, `ReportConstruction` and the four fixtures; regenerate the baseline; the test passes.
- [ ] 10.3 Add the two `captured_at` fixtures and their expectation row. These pin existing behaviour of the timestamp type, so no test can fail first.

## 11. Registry corrections (S01)

- [ ] 11.1 Add `tools/contractsc/tests/live_message_types.rs` asserting the fleet table rows, the `platform.operation_reported` producers and the `Request fact:` summary prefix; it MUST fail on the producers list and the summary.
- [ ] 11.2 Edit `contracts.toml` and regenerate; the test passes.

## 12. Final gate

- [ ] 12.1 Run the full gate of `DEVELOPMENT.md`, `cargo deny check` and `openspec validate --all --strict`; every step passes on the committed tree. Verification only, no test to write.
