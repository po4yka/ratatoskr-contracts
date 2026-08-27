## Why

GitHub Catalog and Telegram currently have no shared wire vocabulary for repository previews, confirmed modes, or component-level partial results. Without one, either side would have to duplicate an unstable JSON shape and Telegram could accidentally turn an ambiguous response into fabricated success.

## What Changes

- Extend `ratatoskr-github-contracts` with strict first-version request/response types for repository preview and repository action execution.
- Preserve stable provider identity separately from the mutable `owner/name` alias and canonical URL.
- Define the closed `metadata`, `track`, and `star` mode vocabulary and closed component outcomes for metadata, provider star, and desired-backup-policy steps.
- Encode safe refusal/failure codes without provider error text, credentials, private payloads, or Telegram callback data.
- Generate and compatibility-check JSON Schema and TypeScript artifacts through the existing contract pipeline.
- Implement the shared workspace capability `github-repository-interaction`; no event subject is added because Platform's existing authenticated streaming gateway carries this bounded request/response API.

## Capabilities

### New Capabilities

- `github-repository-interaction-contracts`: Strict Rust and generated wire contracts for repository preview, confirmed actions, and truthful component outcomes.

### Modified Capabilities

None.

## Impact

- `crates/github-contracts`, its API baseline, schemas/generated TypeScript, and contract validation fixtures.
- Consumers merge after this repository: `ratatoskr-github` first, then `ratatoskr-telegram`; the coordinating fleet spec is `github-repository-interaction` in the workspace changeset of the same task.
- Additive first-version contract only; no new production dependency and no breaking replacement of repository-analysis types.
