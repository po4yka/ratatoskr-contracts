## 1. Site manifest (workspace SITE-025 tasks 3.1 and 3.2)

- [x] 1.1 RED: add `fixtures/sites/site-manifest/{valid-minimal,valid-all-capabilities,invalid-unknown-key,invalid-unknown-capability,invalid-two-hostnames,invalid-site-id}.toml` and `tools/contractsc/tests/sites_fixtures.rs::site_manifest_fixtures_match_their_names`. It MUST fail on its first assertion, because `sites/site-manifest.v1` does not exist.
- [x] 1.2 GREEN: add the crate `ratatoskr-sites-contracts` with `SiteManifest` and `SiteCapability`, the row `sites.site_manifest`, `ratatoskr-sites` in the known services, the registry entries and the canonical renderer, run `cargo contracts generate`, and update the pinned counts. 1.1 passes.

## 2. Release and page (workspace SITE-025 tasks 3.3 and 3.4)

- [x] 2.1 RED: add `fixtures/sites/release/valid/minimal.json`, `fixtures/sites/page/valid/{with-callout,with-wikilink,with-embed}.json` and `sites_fixtures.rs::release_and_page_fixtures_round_trip`. It MUST fail because `sites/release.v1` and `sites/page.v1` do not exist.
- [x] 2.2 GREEN: add `Release` and `Page` with the tree nodes, their rows, registry entries and renderers, regenerate, and update the pinned counts. 2.1 passes, and `fixtures/content/document` does not change.

## 3. Messages (workspace SITE-025 tasks 3.5 and 3.6)

- [x] 3.1 RED: add `tools/contractsc/tests/live_message_types.rs::sites_message_types_are_registered`. It MUST fail because `sites.release.build_requested.v1` is not registered.
- [x] 3.2 GREEN: add `ReleaseBuildRequested`, `ReleasePublished` and `SiteApprovalChanged` with their rows and fixtures, add `ratatoskr-sites` to the consumers of `notes.file.changed.v1` and `notes.vault.disconnected.v1`, regenerate, and update the pinned counts. 3.1 passes and `cargo test --workspace` passes.
- [x] 3.3 Document the three messages in the `README.md` subject table. No test: documentation.
- [x] 3.4 Bless the public API baseline of `ratatoskr-sites-contracts` with `cargo contracts api-write`. No test: a generated baseline; `cargo contracts api-check` verifies it.

## 4. Follow-ups (workspace SITE-025 tasks 3.8 to 3.10)

- [x] 4.1 RED: `live_message_types.rs::operation_reports_list_exactly_the_services_that_emit_them` expects `ratatoskr-notes` and `ratatoskr-sites` among the producers. It MUST fail because the row does not name them.
- [x] 4.2 GREEN: add `ratatoskr-notes` and `ratatoskr-sites` to the producers of `platform.operation_reported`. 4.1 passes.
- [x] 4.3 Add `sites_fixtures.rs::release_fixture_names_each_page_fixture_by_its_sha256`. No failing test first: the fixtures already agree. A changed byte in a page fixture makes it fail.
