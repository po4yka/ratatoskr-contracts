# Proposal: add-blob-transfer-contracts

## Why

Upload-capable clients (mobile plan item 8, export-agent plan item 7) must deliver large local files to receiving services' blob stores (AI-archive receipts, extractor's blob store), but no shared transfer discipline exists: without one, every receiving service invents its own chunking, resumability and digest dialect. The legacy monolith accepted uploads synchronously in one process and left no reference to follow. The routing prerequisite is settled: platform ADR-0015 decides uploads traverse edge proxying under a `transfer` route class to per-service receipts, which fixes where this protocol lives on the wire.

## What Changes

- New contract crate `ratatoskr-blob-transfer-contracts` defining the transfer protocol as document contracts in a new `transfer` family — six one-root contracts, one per message shape:
  - upload session initiation carrying the declared size, media type and SHA-256 digest (digest-first; algorithm negotiation fixed to SHA-256 at version 1);
  - chunk addressing by index over a declared fixed chunk size, with ordering, gap and last-chunk rules decided once here;
  - server-issued opaque resumability tokens and a resume status view of received chunks;
  - finalize outcome verifying the streamed digest against the declared digest, with an explicit terminal mismatch variant beside the success variant;
  - per-chunk retry and idempotency semantics (identical replay succeeds once; divergent bytes conflict);
  - an error taxonomy mapped onto `error-contracts` codes;
  - the canonical HTTP binding documented in prose while every wire type stays transport-honest.
- Out of scope: server-side storage placement, quarantine policies, multipart-form alternatives, bus events.

## Capabilities

### New Capabilities

- `blob-transfer`: the chunked, resumable, digest-first upload transfer protocol shared by upload-capable clients and receiving services' blob-receipt endpoints.

### Modified Capabilities

- None.

## Impact

- **Code:** new `crates/blob-transfer-contracts`; workspace manifests; `tools/contractsc` registry, pins and fixture renderer arms; regenerated `schemas/json-schema/transfer/*` and `generated/typescript/`.
- **Contracts registry:** six `[[contract]]` rows (one per message shape under `transfer.`) plus vocabulary additions (`expires_at` timestamp name; `ratatoskr-mobile`, `ratatoskr-export-agent` services).
- **Consumers:** `ratatoskr-mobile`, `ratatoskr-export-agent` (uploaders), `ratatoskr-chatgpt`, `ratatoskr-claude`, `ratatoskr-extractor` (receipts) — cited for review; no service code changes here.
- **Cross-repository context:** cites workspace store spec `blob-references` (the finalize success outcome yields exactly the `BlobRef` that spec defines) and platform ADR-0015 (the HTTP binding's route class and claim headers).
