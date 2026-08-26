# Design: add-social-analysis-completed-event

## Context

`social-analysis-intake` is the ratified workspace boundary: captured and updated SocialSource
facts are analysis requests, and completed analyses return by social identity plus content digest.
The contracts repository already owns the social identity, digest, tenant, timestamp, event
envelope, generators, and compatibility fixtures.

## Decisions

### D1: The payload stays in the social-contracts crate

The payload describes completion of analysis for a `SocialSourceId` and reuses the social source
digest semantics. It belongs beside the captured, updated, and removed social boundary facts,
rather than creating a general analysis crate before another consumer needs one.

### D2: Linkage fields only

The payload contains `owner`, `social_source_id`, `content_digest`, and `completed_at`. Detailed
analysis output and Knowledge run identity remain Knowledge-owned and are retrieved through that
bounded context. Source services may persist the linkage tuple but not a foreign identifier.

### D3: Additive first-version contract

This is a new event family and does not change existing payloads. Unknown additive extensions are
preserved by the existing extensions mechanism.
