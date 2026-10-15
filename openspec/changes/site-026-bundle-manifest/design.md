## Context

The behaviour is defined by the workspace store (change `add-site-gateway-hosting`, spec
`site-bundles`, design D4). The SITE-025 types in `crates/sites-contracts` are the model: `SiteId`,
validated strings, unknown members refused, list rules in `validate()`.

## Decisions

- **The schema is generated.** ADR-0001 makes the Rust type the only source of a schema, so the
  type that workspace task 3.4 names lands with the schema in task 3.2. The tests of task 3.3 then
  cannot fail first; they are pin tests with a mutation proof.
- **A bundle path.** Relative, segments separated by `/`, no empty, `.` or `..` segment, no
  backslash and no control character, at most 1024 bytes, and no first segment `_r`, which the
  gateway reserves for forwarding. `.ratatoskr/` is accepted. NFC is the producer's duty, as for
  `VaultPath`.
- **The files.** Sorted by path, each path once; the entry document is one of them. A size is at
  most 2^53 - 1, as `ReleaseAsset.size_bytes`.
- **The repository** is the GitHub `owner/name` of the site repository, and the commit is the full
  40-hex SHA-1 commit id.
