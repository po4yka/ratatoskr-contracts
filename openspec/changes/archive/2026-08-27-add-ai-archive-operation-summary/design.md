## Context

`OperationResultRef` is the contract boundary that Platform persists and clients consume. It
previously named only a generic result target. AI archive producers already own the authoritative
import head and evidence-based completeness report, but export-agent needs a bounded projection of
that result without receiving archive content or parser diagnostics.

The active workspace change
`archive-operation-completeness-projection` names the producer, Platform and export-agent rollout
order. This change owns only the wire type and generated artifacts.

## Decisions

- Add `ai_archive_import_summary` as an optional typed field of `OperationResultRef`; never use
  `extensions` for this producer-authored data.
- Reuse `AiArchiveOperationSummary`, whose fields are the immutable Ratatoskr archive id, provider,
  completeness class, and aggregate counts. It deliberately carries no content, names, paths,
  external/provider identifiers, detailed warnings or gaps, or credentials.
- Validate the field's association at deserialization and when a producer constructs a result:
  it is legal only for `result_kind = ai_archive.import`, its target must be an `ai_archive` entity,
  and that target must equal `ai_archive_import_summary.ai_archive_id`.
- The field is optional for rollout. Its absence means no completeness conclusion; it never means
  complete.

## Consequences

The change is additive on the wire. Older producers omit the field and new consumers retain an
unverified result. Older generic relays preserve the unknown typed member through their existing
extension channel, but no new producer authors it there.

The generated JSON Schema expresses the field shape. Rust validation expresses the cross-field
kind/target association that schema cannot state cleanly. Fixtures cover both layers accordingly.

## Rollout and rollback

1. Publish this contract and generated artifacts.
2. Update Platform's generic operation projection to retain the field unchanged.
3. Update ChatGPT and Claude terminal import reporters to author valid summaries.
4. Update export-agent to consume the field as authoritative backend evidence.

Rollback stops producer emission. Existing consumers show the result as unverified; no schema
migration, new API version, or compatibility route is involved.
