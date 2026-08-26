# blob-transfer Specification

## Purpose

One chunked, resumable, digest-first transfer discipline for every upload-capable client delivering bytes to a receiving service's blob store, so that no receiving service invents its own wire dialect. The contract fixes the message shapes, the chunk-addressing rules, the resumability-token discipline, the finalize verification and the error taxonomy; receiving services keep ownership of storage placement and receipts. Routing context: platform ADR-0015 proxies these calls through edge under the `transfer` class; produced blobs become addressable per the workspace store spec `blob-references`.

## ADDED Requirements

### Requirement: Sessions open digest-first

An upload session SHALL be initiated by a declaration carrying the total byte size, the media type, the digest algorithm (`sha256` at version 1) and the lowercase-hex SHA-256 digest of the whole payload, before any payload byte is transferred, and a receiving service SHALL be able to refuse the session on the declaration alone.

#### Scenario: declaration precedes bytes

- **WHEN** a client opens a session with a well-formed declaration
- **THEN** the opened-session answer names the server-issued resumption token, echoes the negotiated chunking, and carries an expiry instant, and no payload byte has been sent

#### Scenario: malformed digest stops processing

- **WHEN** a declaration carries a digest that is not 64 lowercase hexadecimal characters
- **THEN** deserialization rejects the document in every layer that validates it

### Requirement: Chunks are addressed by index over a declared fixed size

Chunk addressing SHALL be by zero-based index against the chunk size declared at initiation; every chunk except the last SHALL be exactly that size, the last SHALL be the remainder, the chunk count SHALL follow from the declared total, and a chunk outside the derived range SHALL be invalid.

#### Scenario: index arithmetic is total

- **WHEN** a session declares a total size and a chunk size within the protocol bounds
- **WHEN** any participant asks how many chunks the transfer has and what length chunk N must have
- **THEN** the answers follow deterministically from the declaration alone

#### Scenario: out-of-range chunk is refused

- **WHEN** a chunk arrives whose index equals the derived chunk count
- **THEN** the reference semantics reject it with the out-of-range outcome and the session state is unchanged

### Requirement: Resumption is driven by an opaque server-issued token

A receiving service SHALL issue an opaque resumption token at session opening, the token SHALL be presented on every chunk receipt, status and finalize call, and the protocol SHALL distinguish an unknown or expired token from every other failure.

#### Scenario: token grammar is enforced at the boundary

- **WHEN** a message carries a resumption token violating the published token grammar
- **THEN** deserialization rejects the message rather than preserving it

### Requirement: An interrupted transfer resumes from a status view

A client that lost its connection SHALL be able to ask for the session's received-chunk view, and the view SHALL identify exactly which indices are recorded so the remaining chunks can be sent without duplication or gaps being guessed.

#### Scenario: resume after interruption sends only what is missing

- **WHEN** a transfer recorded chunks 0, 1 and 3 and the client queries status
- **THEN** the status answer lists exactly the received indices and the count of missing chunks, and re-sending chunk 1 afterwards changes nothing

### Requirement: Chunk replay is idempotent and divergence is a conflict

Re-receiving a chunk whose bytes hash to the digest already recorded for that index SHALL succeed once more without changing session state, and receiving different bytes for an already-recorded index SHALL be refused as a conflict without corrupting the recorded chunk.

#### Scenario: identical replay succeeds

- **WHEN** the same chunk bytes are delivered twice for one index
- **THEN** both deliveries answer success and the second leaves the recorded state unchanged

#### Scenario: divergent bytes conflict

- **WHEN** bytes with a different digest arrive for an already-recorded index
- **THEN** the delivery is refused with the conflict outcome, the originally recorded chunk survives, and the session remains usable

### Requirement: Finalize verifies the streamed digest with an explicit outcome

Finalization SHALL be possible only when every derived chunk index is recorded, SHALL compute the digest over the streamed bytes itself, SHALL yield a stored outcome carrying the blob reference fields when the streamed digest equals the declared digest, and SHALL yield an explicit digest-mismatch outcome otherwise; an incomplete session SHALL be refused with a dedicated code while staying open.

#### Scenario: matching digest finalizes to a blob reference

- **WHEN** every chunk is recorded and the streamed digest equals the declared digest
- **THEN** the outcome is the stored variant carrying the content identity a `BlobRef` needs, and the session is terminal

#### Scenario: mismatched digest is a truthful terminal outcome

- **WHEN** every chunk is recorded but the streamed digest differs from the declared digest
- **THEN** the outcome is the mismatch variant carrying expected and computed digests, and the session is terminal-failed

#### Scenario: premature finalize refuses without closing

- **WHEN** finalize is called while chunk indices remain unrecorded
- **THEN** the call is refused with the incomplete code and the session stays open for further chunks

### Requirement: Failures speak the shared error taxonomy

Every protocol failure SHALL map onto a stable `blob_transfer.`-namespaced code consumable through the shared error envelope, with an explicit retriable-or-not classification per code.

#### Scenario: codes branch, messages do not

- **WHEN** a consumer inspects a protocol failure
- **THEN** it branches on the stable code within the `blob_transfer.` namespace and never on the human-readable message

### Requirement: Transport honesty with one canonical HTTP binding

The protocol types SHALL remain transport-honest — no HTTP status codes, methods, headers or URLs inside any wire type — while the canonical HTTP binding (endpoints, methods, status usage) is documented normatively beside them.

#### Scenario: types survive a non-HTTP transport unchanged

- **WHEN** the message shapes are serialized over a transport other than HTTP
- **THEN** no field of any type references HTTP semantics, and the canonical binding stands as documentation only
