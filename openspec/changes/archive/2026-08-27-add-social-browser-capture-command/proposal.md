## Why

Platform and the browser extension need one typed command to deliver an explicit X, Instagram, or
Threads permalink to its owning social service. Existing social contracts describe normalized facts,
not the capture request, so current clients can only send an indistinguishable generic extractor
URL.

## What Changes

- Add the canonical command-envelope contract and bind `social.capture.requested.v1` to it for
  Platform-to-owner explicit social capture with closed provider and provenance vocabulary.
- Add closed typed social operation outcome codes for unavailable/deleted sources and a linked-
  article extraction failure warning, each mapping explicitly to the shared error envelope code.
- Extend contract metadata and generation with a first-class `commands` family, so the registry
  verifies `CommandPayload::COMMAND_TYPE` instead of treating a command as a generic schema.
- Register schemas, generated TypeScript declarations, valid/invalid fixtures, and compatibility
  metadata for the new command.

## Capabilities

### New Capabilities

- `social-browser-capture-command`: versioned command and outcome vocabulary implementing the
  active workspace change `add-social-browser-capture-contract`.

### Modified Capabilities

None.

## Impact

- Canonical sources: `ratatoskr-event-envelope`, `ratatoskr-identifiers`,
  `ratatoskr-social-contracts`, `contracts.toml`, fixtures, generated schemas, and TypeScript
  output.
- Producers: Platform. Consumers: Ratatoskr X, Instagram, Threads, and Browser Extension through
  Platform's public API projection.
- The command is additive and must land before Platform routing or any social owner consumer.
