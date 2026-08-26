# Proposal: add-social-analysis-completed-event

## Why

The ratified workspace `social-analysis-intake` boundary requires producers to link completed
Knowledge analyses through `(social_source_id, content_digest)`, but its completion fact has no
published payload. A source service therefore cannot validate or persist a completion without
inventing a private wire shape.

## What Changes

- Add the typed `SocialSourceAnalysisCompleted` event payload to
  `ratatoskr-social-contracts`, with event type `knowledge.analysis.completed.v1`.
- Carry only `owner`, `social_source_id`, `content_digest`, `completed_at`, and preserved
  extensions. The event carries neither model output nor a Knowledge-private run identifier.
- Register the payload, generate its JSON Schema and TypeScript mirror, and provide valid,
  invalid, and compatibility fixtures.

## Impact

The producer is `ratatoskr-knowledge`; source services including `ratatoskr-instagram` consume
the event only for observational linkage. This implements the published workspace boundary and
does not alter Knowledge internals or source ownership.
