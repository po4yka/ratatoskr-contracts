//! The protocol's reference semantics: addressing arithmetic, transitions, resume, mismatch,
//! idempotency. Tasks 2.1–2.10 of change `add-blob-transfer-contracts`.

#![allow(
    clippy::expect_used,
    reason = "a failed expectation in a test binary is the reporting mechanism"
)]
#![allow(
    clippy::panic,
    reason = "a panicking assertion in a test binary is the reporting mechanism"
)]

use ratatoskr_blob_transfer_contracts::{
    CHUNK_SIZE_MAX_BYTES, CHUNK_SIZE_MIN_BYTES, ChunkRecordOutcome, MAX_CHUNK_COUNT, UploadPlan,
    UploadSession, UploadSessionRequest,
};
use ratatoskr_identifiers::{BlobOwner, ContentDigest, DigestAlgorithm, DigestHex, MediaType};

const OWNER: &str = "ratatoskr-extractor";
const TOKEN: &str = "rst_000000000000000000000000";
const DECLARED_HEX: &str = "1111111111111111111111111111111111111111111111111111111111111111";

fn digest_hex(hex: &str) -> DigestHex {
    DigestHex::parse(hex).expect("test hex satisfies the grammar")
}

/// Heap-built filler bytes of exactly `len` elements; no oversized array literal lands on the
/// stack.
fn payload(byte: u8, len: u32) -> Vec<u8> {
    core::iter::repeat_n(byte, usize::try_from(len).expect("fits")).collect()
}

fn request(declared_size_bytes: u64, chunk_size_bytes: u32) -> UploadSessionRequest {
    UploadSessionRequest {
        declared_size_bytes,
        media_type: MediaType::parse("application/pdf").expect("grammar holds"),
        digest: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            hex: digest_hex(DECLARED_HEX),
        },
        chunk_size_bytes,
        extensions: ratatoskr_identifiers::Extensions::new(),
    }
}

fn open_session(declared_size_bytes: u64, chunk_size_bytes: u32) -> UploadSession {
    UploadSession::new(&request(declared_size_bytes, chunk_size_bytes))
        .expect("the declaration is inside bounds")
}

/// 2.1 / 2.2 — addressing arithmetic is total and refused out of range.
#[test]
fn chunk_plan_derives_count_and_lengths_totally() {
    let exact = UploadPlan::new(131_072, 65_536).expect("inside bounds");
    assert_eq!(exact.expected_chunk_count(), 2);
    assert_eq!(exact.chunk_len(0), Some(65_536));
    assert_eq!(exact.chunk_len(1), Some(65_536));

    let short_tail = UploadPlan::new(100_000, 65_536).expect("inside bounds");
    assert_eq!(short_tail.expected_chunk_count(), 2);
    assert_eq!(short_tail.chunk_len(1), Some(34_464));

    let single_short = UploadPlan::new(60_000, 65_536).expect("inside bounds");
    assert_eq!(single_short.expected_chunk_count(), 1);
    assert_eq!(single_short.chunk_len(0), Some(60_000));

    assert_eq!(short_tail.chunk_len(2), None, "past the derived range");
    assert_eq!(short_tail.chunk_len(u32::MAX), None);

    assert!(matches!(
        UploadPlan::new(0, CHUNK_SIZE_MIN_BYTES),
        Err(ratatoskr_blob_transfer_contracts::TransferContractError::ZeroDeclaredSizeBytes)
    ));
    assert!(matches!(
        UploadPlan::new(1_000_000, CHUNK_SIZE_MIN_BYTES - 1),
        Err(ratatoskr_blob_transfer_contracts::TransferContractError::ChunkSizeBelowMinimum { .. })
    ));
    assert!(matches!(
        UploadPlan::new(1_000_000, CHUNK_SIZE_MAX_BYTES + 1),
        Err(ratatoskr_blob_transfer_contracts::TransferContractError::ChunkSizeAboveMaximum { .. })
    ));
    assert!(matches!(
        UploadPlan::new(
            u64::from(MAX_CHUNK_COUNT + 1) * u64::from(CHUNK_SIZE_MIN_BYTES),
            CHUNK_SIZE_MIN_BYTES,
        ),
        Err(ratatoskr_blob_transfer_contracts::TransferContractError::TooManyChunks { .. })
    ));
}

/// 2.3 / 2.4 — transitions: record, refused shapes leave no trace, premature finalize stays open,
/// complete path seals to a stored outcome.
#[test]
fn session_transitions_follow_the_protocol() {
    let mut session = open_session(100_000, 65_536);

    assert_eq!(
        session
            .record(0, &payload(0xAB, 65_536))
            .expect("first chunk"),
        ChunkRecordOutcome::Recorded
    );
    assert!(matches!(
        session.record(1, &payload(0xAB, 12)),
        Err(
            ratatoskr_blob_transfer_contracts::TransferFailure::ChunkLengthInvalid {
                expected_bytes: 34_464,
                got_bytes: 12,
                ..
            }
        )
    ));
    assert!(matches!(
        session.record(2, &payload(0xAB, 65_536)),
        Err(
            ratatoskr_blob_transfer_contracts::TransferFailure::ChunkIndexOutOfRange {
                count: 2,
                ..
            }
        )
    ));

    assert!(matches!(
        session.finalize(
            BlobOwner::parse(OWNER).expect("owner"),
            digest_hex(DECLARED_HEX)
        ),
        Err(
            ratatoskr_blob_transfer_contracts::TransferFailure::FinalizeIncomplete {
                missing_chunks_count: 1,
            }
        )
    ));
    assert_eq!(
        session
            .status_view(
                &ratatoskr_blob_transfer_contracts::UploadResumptionToken::parse(TOKEN)
                    .expect("token")
            )
            .missing_chunks_count,
        1,
        "a premature finalize leaves the session open"
    );

    let _ = session
        .record(1, &payload(0xCD, 34_464))
        .expect("final chunk");
    let outcome = session
        .finalize(
            BlobOwner::parse(OWNER).expect("owner"),
            digest_hex(DECLARED_HEX),
        )
        .expect("complete and matching");
    let ratatoskr_blob_transfer_contracts::UploadCompletionOutcome::Stored { blob_ref, .. } =
        outcome
    else {
        panic!("a matching stream stores");
    };
    assert_eq!(blob_ref.owner_service.as_str(), OWNER);
    assert_eq!(blob_ref.digest.hex.as_str(), DECLARED_HEX);
    assert_eq!(blob_ref.digest.algorithm, DigestAlgorithm::Sha256);
    assert_eq!(blob_ref.media_type.as_str(), "application/pdf");
    assert_eq!(blob_ref.length_bytes, 100_000);
}

/// 2.5 / 2.6 — resume: the status view names exactly what survived, replays change nothing, and
/// the transfer completes through the gaps.
#[test]
fn resume_after_interruption_sends_only_missing_chunks() {
    let tail = 100_u32;
    let total = 3 * 65_536 + u64::from(tail);
    let mut session = open_session(total, 65_536);
    for index in [0_u32, 1, 3] {
        let len = if index == 3 { tail } else { 65_536 };
        let _ = session.record(index, &payload(0xAB, len));
    }

    let token =
        ratatoskr_blob_transfer_contracts::UploadResumptionToken::parse(TOKEN).expect("token");
    let view = session.status_view(&token);
    assert_eq!(
        view.session_state,
        ratatoskr_blob_transfer_contracts::UploadSessionState::Open
    );
    assert_eq!(view.received_chunks, vec![0, 1, 3]);
    assert_eq!(view.received_chunks_count, 3);
    assert_eq!(view.missing_chunks_count, 1);

    assert_eq!(
        session.record(1, &payload(0xAB, 65_536)).expect("replay"),
        ChunkRecordOutcome::IdempotentReplay,
        "an interrupted client may redeliver a recorded chunk"
    );
    assert_eq!(
        session.status_view(&token).received_chunks,
        vec![0, 1, 3],
        "a replay changes nothing"
    );

    let _ = session
        .record(2, &payload(0xAB, 65_536))
        .expect("gap filled");
    let outcome = session
        .finalize(
            BlobOwner::parse(OWNER).expect("owner"),
            digest_hex(DECLARED_HEX),
        )
        .expect("complete");
    assert!(matches!(
        outcome,
        ratatoskr_blob_transfer_contracts::UploadCompletionOutcome::Stored { .. }
    ));
}

/// 2.7 / 2.8 — a divergent stream is a truthful terminal mismatch carrying both digests, and the
/// session answers nothing afterwards.
#[test]
fn digest_mismatch_is_a_truthful_terminal_outcome() {
    let mut session = open_session(65_536, 65_536);
    let _ = session
        .record(0, &payload(0xAB, 65_536))
        .expect("only chunk");

    let computed = "2222222222222222222222222222222222222222222222222222222222222222";
    let outcome = session
        .finalize(
            BlobOwner::parse(OWNER).expect("owner"),
            digest_hex(computed),
        )
        .expect("mismatch is an answer, not a refusal");
    let ratatoskr_blob_transfer_contracts::UploadCompletionOutcome::DigestMismatch {
        declared_sha256_hex,
        computed_sha256_hex,
        ..
    } = outcome
    else {
        panic!("a divergent stream must not store");
    };
    assert_eq!(declared_sha256_hex.as_str(), DECLARED_HEX);
    assert_eq!(computed_sha256_hex.as_str(), computed);
}

/// 2.9 / 2.10 — idempotent replay succeeds once more; divergence conflicts without corrupting the
/// recorded chunk; the session stays usable afterwards.
#[test]
fn identical_chunk_replay_is_idempotent_and_divergence_conflicts() {
    let mut session = open_session(200_000, 65_536);
    let _ = session.record(0, &payload(0xAB, 65_536)).expect("record");

    assert_eq!(
        session
            .record(0, &payload(0xAB, 65_536))
            .expect("identical replay"),
        ChunkRecordOutcome::IdempotentReplay
    );
    assert!(matches!(
        session.record(0, &payload(0xCD, 65_536)),
        Err(ratatoskr_blob_transfer_contracts::TransferFailure::ChunkConflict { index: 0 })
    ));
    assert_eq!(
        session
            .record(0, &payload(0xAB, 65_536))
            .expect("original survives"),
        ChunkRecordOutcome::IdempotentReplay,
        "the conflict did not corrupt the recorded chunk"
    );

    let _ = session.record(1, &vec![0xEF; 65_536]).expect("rest");
    let _ = session.record(2, &vec![0xEF; 65_536]).expect("rest");
    let _ = session
        .record(3, &vec![0xEF; 3_392])
        .expect("short last chunk");
    let outcome = session
        .finalize(
            BlobOwner::parse(OWNER).expect("owner"),
            digest_hex(DECLARED_HEX),
        )
        .expect("session stayed usable through the conflict");
    assert!(matches!(
        outcome,
        ratatoskr_blob_transfer_contracts::UploadCompletionOutcome::Stored { .. }
    ));
}
