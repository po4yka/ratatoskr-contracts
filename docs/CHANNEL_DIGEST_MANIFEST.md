# Channel digest manifest

> Status: Proposed  
> Owner: `ratatoskr-channel-digests` (producer), `ratatoskr-knowledge` (consumer)  
> Source of the wire shape: `crates/channel-digest-contracts/src/manifest.rs`  
> Changeset: XR-021, CONTRACTS.md section S08

A digest run selects immutable source revisions of public Telegram channel posts. `ratatoskr-channel-digests` stores that selection as one manifest and serves the stored bytes over loopback HTTP. `ratatoskr-knowledge` reads the manifest to build a recap, and trusts it only after it has checked the bytes against the digest the recap request carries.

This document describes the HTTP surface. The shape of the manifest, its canonical bytes and its validation rules are the Rust types; the generated schema is `schemas/json-schema/channel_digest/manifest.v1.schema.json`.

## Canonical bytes

The manifest is a closed, digest-verified artifact. Its canonical bytes are compact UTF-8 JSON with no trailing newline in which every object's keys appear in lexicographic order. `ChannelDigestManifest::to_canonical_bytes` produces them and refuses an invalid manifest; `ChannelDigestManifest::from_canonical_bytes` decodes them and refuses any byte sequence that is not the canonical rendering of the value it contains (reordered keys, pretty printing, a trailing newline, duplicate keys, an explicit `null`). The digest of a manifest is the SHA-256 of its canonical bytes, in lowercase hexadecimal (`sha256_hex`).

The digest service stores the canonical bytes and answers every read with exactly those bytes. It never serializes the stored value again, so the digest a consumer computes over a response is the digest the producer computed over the stored artifact.

## Read route

`GET /v1/manifests/{manifest_id}` on the digest service's API listener, `127.0.0.1:8098`. `{manifest_id}` is the bare UUID of the manifest reference; `manifest_path(&ChannelDigestManifestRef)` builds the path, so there is no manifest-reference header.

| Request header | Value |
| --- | --- |
| `Authorization` | `Bearer <service secret>`, the digest service's `RATATOSKR__AUTH__SERVICE_SECRET` |
| `x-ratatoskr-owner-id` | the bare lowercase UUID of the owning user, not `user:<uuid>` |
| `x-ratatoskr-digest-run-id` | the bare UUID of the digest run the caller expects |
| `x-ratatoskr-manifest-digest` | the 64 lowercase hexadecimal characters of the SHA-256 the caller expects |

| Status | Meaning |
| --- | --- |
| `200` | The body is exactly the stored canonical manifest. `content-type: application/json` and `cache-control: no-store`. |
| `401` | The bearer is missing or wrong, or the owner header is not a UUID. |
| `404` | Empty body. The manifest is absent, belongs to another owner, or a claim differs: the run id is not the manifest's run, or the digest is not the stored SHA-256. These cases are indistinguishable on purpose. |

## Readiness

`GET /ready` on the API listener requires the bearer only, with no owner header. It answers `200` when the database answers `select 1` and `503` otherwise, and carries `cache-control: no-store`. `GET /live` is not served on the API listener (`404`); the operator plane on ports 9469 and 9470 is unchanged.
