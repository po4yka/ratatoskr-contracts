# social-analysis-completed-event Specification

## Purpose
TBD - created by archiving change add-social-analysis-completed-event. Update Purpose after archive.

## Requirements

### Requirement: Social analysis completion has one typed linkage fact

`knowledge.analysis.completed.v1` SHALL use `SocialSourceAnalysisCompleted` from
`ratatoskr-social-contracts`. The payload SHALL carry the owner, `social_source_id`,
`content_digest`, and `completed_at`, plus preserved extensions. It SHALL NOT carry model output,
prompt material, raw responses, or a Knowledge-private run identifier.

#### Scenario: a source service links a completed result without a foreign key

- **WHEN** a source service receives a valid completion fact
- **THEN** it can match its local record using the payload's `social_source_id` and `content_digest` without writing a Knowledge identifier

#### Scenario: completion remains private by reference

- **WHEN** the completion fact is serialized into an event envelope
- **THEN** it contains no model output, raw response, prompt text, or Knowledge-private run identifier
