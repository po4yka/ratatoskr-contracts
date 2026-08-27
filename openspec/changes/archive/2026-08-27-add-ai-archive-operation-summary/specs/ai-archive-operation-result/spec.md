## ADDED Requirements

### Requirement: AI archive terminal result summary is typed and bounded

An operation result MAY contain `ai_archive_import_summary` only when its `result_kind` is
`ai_archive.import`. The summary SHALL expose only the archive id, provider, evidence-based
completeness class, and aggregate conversation, message, asset, gap and warning counts.

#### Scenario: A complete archive import result is represented without content

- **WHEN** an import producer reports a complete AI archive result
- **THEN** the typed summary contains the matching archive id, `complete`, and zero gaps
- **AND THEN** it contains no archive content, title, path, provider account identifier, detailed
  diagnostic, or credential

### Requirement: Summary identity is bound to its result target

When `ai_archive_import_summary` is present, `target` SHALL be an `ai_archive` reference for the
same archive id held by the summary.

#### Scenario: Mismatched target is rejected

- **WHEN** a payload gives the summary a different archive id from its result target
- **THEN** typed deserialization rejects the payload

### Requirement: Missing evidence remains explicitly absent

The summary field SHALL remain optional during producer rollout. Consumers SHALL not interpret a
missing summary as a complete or gap-free archive import.

#### Scenario: Older producer result remains valid

- **WHEN** an `ai_archive.import` result has no summary
- **THEN** the operation snapshot remains valid with no completeness assertion
