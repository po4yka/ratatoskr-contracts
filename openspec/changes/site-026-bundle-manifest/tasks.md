## 1. Bundle manifest (workspace SITE-026 tasks 3.1 to 3.3)

- [x] 1.1 RED: add `crates/sites-contracts/tests/bundle_manifest.rs` with `a_manifest_without_a_file_digest_is_refused`, `a_path_with_a_parent_segment_is_refused` and `a_path_under_the_forwarding_prefix_is_refused`, and their fixtures under `fixtures/sites/bundle-manifest/invalid/`. They MUST fail because the schema file does not exist.
- [x] 1.2 GREEN: add `BundleManifest`, its row, its valid fixture and the expectations, run `cargo contracts generate`, and update the pinned counts. 1.1 passes, and `tools/contractsc/tests/fixtures.rs` passes.
- [x] 1.3 Add `a_valid_manifest_round_trips` and `a_manifest_with_the_reserved_sdk_path_is_valid`. No failing test first: the type and the path rule land in 1.2. A mutation of each makes it fail.

## 2. Deploy responses (workspace SITE-026 task 3.4)

- [x] 2.1 RED: add the fixtures under `fixtures/sites/deploy-stored/` and `fixtures/sites/deploy-activated/` and `sites_fixtures.rs::deploy_responses_round_trip`. It MUST fail because `sites/deploy-stored.v1` and `sites/deploy-activated.v1` do not exist.
- [x] 2.2 GREEN: add `DeployStored` and `DeployActivated` (the commit is the deploy id), their rows, registry entries and renderers, regenerate, update the pinned counts, and document the manifest and the two routes in `README.md`. 2.1 passes.
