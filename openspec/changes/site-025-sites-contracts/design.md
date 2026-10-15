## Context

The behaviour is defined by the workspace store (change `add-site-publications`, specs
`site-manifest`, `site-publications`, `site-release-privacy` and `site-content-delivery`, design D2,
D4, D6 and D9). The notes types in `crates/notes-contracts` are the model: closed vocabularies,
validated strings, unknown fields refused, cross-field rules in `validate()`.

## Decisions

- **Where the files are.** Rule R3 puts a non-event schema at
  `schemas/json-schema/<family>/<id suffix>.v1.schema.json`, so the rows `sites.site_manifest`,
  `sites.release` and `sites.page` give `schemas/json-schema/sites/{site-manifest,release,page}.v1`.
  The command schema is under `commands/` and the two event schemas under
  `schemas/events/<event type>.schema.json`. One row registers one message type (R9, R10).
- **Fixture paths.** The manifest fixtures are TOML files with the names of workspace task 3.1,
  because the manifest is a TOML file; rule R13 reads JSON only. Rule R13 puts every JSON fixture
  under `valid/`, `invalid/` or `compat/`, so the release and page fixtures of workspace task 3.3
  are `fixtures/sites/release/valid/minimal.json` and
  `fixtures/sites/page/valid/{with-callout,with-wikilink,with-embed}.json`, and the manifest also
  has `fixtures/sites/site-manifest/valid/minimal.json` for the shared fixture tests. The minimal
  release lists the three pages, and each `page_sha256` is the SHA-256 of that page fixture's bytes.
- **A site id.** 3 to 40 lower-case ASCII letters, digits and `-`, the same grammar as the `site:`
  form of `WriteOrigin` in `ratatoskr-notes-contracts`. A publication id has the same grammar,
  because the owner writes it in the manifest and in the path of `PUT .../publications/{id}`.
- **The release names no id of its own.** The release id is the SHA-256 of the RFC 8785 form of the
  release document (design D4), so the document cannot hold it.
- **The page body.** `ast` is an mdast-compatible tree: each node has `type`, and text nodes carry
  `value` as mdast does. The node set is closed. It adds `callout`, `wikiLink` (a resolved slug) and
  `embed` (a resolved page or asset), and it has no raw HTML node: raw HTML of a note reaches only
  the sanitised `html` field, so a site that renders the tree never inserts markup.
- **Unknown fields are refused** (`unknown_policy = "reject"`). A release is public, and it must not
  carry a member that the privacy filter did not produce.
