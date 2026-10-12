# Deletion acknowledgement and stream erasure

> Status: Proposed  
> Owner: `ratatoskr-knowledge` (acknowledgement producer), `ratatoskr-platform` (erasure consumer)  
> Source of the wire shapes: `crates/operation-contracts/src/deletion_ack.rs` and `crates/operation-contracts/src/stream_erasure.rs`  
> Changeset: XR-021 round 5, CONTRACTS.md R5-11 and R5-12

Two additive messages close two privacy gaps of the events stream. The stream `ratatoskr_events` drops its oldest message at 7 days or 1 GiB, so a deletion fact that Knowledge did not consume in time would vanish without a trace, and the content-bearing facts of a deleted conversation would stay in the stream until it drops them. This document states what each message means, who may send it and what it must never carry. The shapes themselves are the Rust types; the generated schemas are `schemas/events/knowledge.deletion.acknowledged.v1.schema.json` and `schemas/json-schema/commands/platform-stream-erasure-requested.v1.schema.json`.

## `knowledge.deletion.acknowledged.v1` (event)

Knowledge answers a deletion fact with this event: `ai_archive.subject.tombstoned.v1` from `ratatoskr-chatgpt` or `ratatoskr-claude`, and `social.source.removed.v1` from `ratatoskr-x`, `ratatoskr-instagram` or `ratatoskr-threads`. The producer re-sends its deletion fact until the acknowledgement arrives, so the acknowledgement is what ends the re-sending.

| Member | Meaning |
| --- | --- |
| `acknowledged_event_id` | The `event_id` of the deletion fact that is answered. |
| `family` | `ai_archive` or `social`. |
| `owner` | The owner of the deleted data, `user:<uuid>`. |
| `subject` | The subject the deletion fact named, as an opaque reference such as `ai_conversation:<uuid>` or `social_source:<uuid>`. |
| `outcome` | `applied` or `refused`. |
| `refusal` | `foreign_subject`. Present exactly when `outcome` is `refused`; `validate()` and every deserialization enforce it. |
| `acknowledged_at` | The instant Knowledge decided the outcome, according to the Knowledge clock. |

Producer: `ratatoskr-knowledge`. Consumers: `ratatoskr-chatgpt`, `ratatoskr-claude`, `ratatoskr-x`, `ratatoskr-instagram` and `ratatoskr-threads`, each on its own durable `ratatoskr_<service>_deletion_acks`. Subject `evt.knowledge.deletion.acknowledged.v1`.

Envelope rules: `producer` is `ratatoskr-knowledge`, `event_id` is the outbox id, `aggregate_id` equals `payload.subject`, `correlation_id` is copied from the acknowledged envelope, `causation_id` is `event:<acknowledged_event_id>` and `tenant_id` equals `payload.owner`.

`applied` covers a deletion that was carried out and a repeat of one that had been. `refused` means Knowledge deleted nothing, because the subject belongs to another owner than the one the fact names; the data stays in Knowledge and an operator must look. A refusal also ends the re-sending.

An acknowledgement names no content. It carries ids, the owner and a subject reference, never text, titles, file names or blob digests, because every one of the five producers receives every acknowledgement. A consumer matches it to the outbox row whose `event_id` equals `acknowledged_event_id` and whose tenant equals `owner`; an acknowledgement that matches no row is acknowledged and ignored, and a duplicate changes nothing.

## `platform.stream_erasure.requested.v1` (command)

A producer asks Platform Edge to erase single messages it published from `ratatoskr_events`. Edge is the only identity allowed to erase a stream message, and no producer holds a stream-management permission.

| Member | Meaning |
| --- | --- |
| `request_ref` | The deletion request this erasure belongs to. Its kind is `privacy_request`. |
| `owner` | The owner whose published messages are erased, `user:<uuid>`. |
| `stream_sequences` | The stream sequence numbers the broker returned in the publish acknowledgement of each message to erase: 1 to 500 entries, each at least 1 and at most 2^53 - 1, strictly ascending (so unique). |

Producers: `ratatoskr-chatgpt` and `ratatoskr-claude`. Consumer: `ratatoskr-platform`, on the durable `platform_stream_erasure` that Edge creates itself. Subject `cmd.platform.stream_erasure.requested.v1`.

Envelope rules: `producer` is `ratatoskr-chatgpt` or `ratatoskr-claude`, `command_id` is the outbox id, `aggregate_id` equals `payload.request_ref`, `correlation_id` is the deletion request's and `tenant_id` equals `payload.owner`.

The command can only name messages the producer itself published. Edge reads each sequence and erases the message only when its subject is one of the seven content-bearing `evt.ai_archive.*` subjects (never `evt.ai_archive.subject.tombstoned.v1`), the producer in the stored envelope equals the command's producer and the tenant in the stored envelope equals `owner`. Any other message is counted as refused and left alone. The contract cannot express this check; it is Platform's rule, and it is why the payload carries sequences and an owner rather than a producer.

Sequences are `u64` in Rust and exact there, but a JavaScript or TypeScript consumer reads a JSON number as a double. A sequence above 2^53 - 1 would be rounded into the number of a different message, so the contract refuses it instead of preserving it. Real stream sequences are nowhere near that bound.
