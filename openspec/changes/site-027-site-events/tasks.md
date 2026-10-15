## 1. Site event (workspace SITE-027 tasks 3.1 and 3.2)

- [x] 1.1 RED: add `crates/sites-contracts/tests/site_event.rs` with `release_published_refuses_page_list`, `release_published_refuses_source_ref` and `every_event_type_has_a_valid_fixture`, and the fixtures under `fixtures/sites/site-event/`. They MUST fail because the schema file does not exist.
- [ ] 1.2 GREEN: add `SiteEvent` and its payloads, the row and the expectations, regenerate, and update the pinned counts. 1.1 passes.

## 2. Draft change message (workspace SITE-027 tasks 3.3 and 3.4)

- [ ] 2.1 RED: add `live_message_types.rs::sites_draft_changed_is_registered`. It MUST fail because the type is not registered.
- [ ] 2.2 GREEN: add `DraftChanged`, its row and fixtures, regenerate, and update the pinned counts. 2.1 passes.
