//! [`NotesContractError`]: every way a notes event invariant can fail.

/// Every way a cross-field rule of a notes event can fail.
///
/// A *parse* error type: it is never serialized onto the wire. Field-level rejection happens in
/// the identifier and string types; only the cross-field rules of [`crate::FileChanged`] surface
/// here.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NotesContractError {
    /// A rename did not name the old path, so a consumer cannot move what it indexed under it.
    #[error("change_kind `renamed` requires previous_path")]
    RenameWithoutPreviousPath,
    /// A change other than a rename named an old path.
    #[error("change_kind `{change_kind}` forbids previous_path; only `renamed` carries one")]
    PreviousPathWithoutRename {
        /// The wire word of the change kind that carried the path.
        change_kind: &'static str,
    },
    /// A deleted file carried a blob, although it has no bytes to name.
    #[error("change_kind `deleted` forbids a blob")]
    BlobOnDeletedFile,
    /// A created, updated or renamed file carried no blob for the bytes of its revision.
    #[error("change_kind `{change_kind}` requires a blob")]
    MissingBlob {
        /// The wire word of the change kind that lacked the blob.
        change_kind: &'static str,
    },
    /// The blob names other bytes than the revision: its digest is not the SHA-256 that
    /// `revision` holds.
    #[error("blob digest does not equal the revision")]
    BlobDigestMismatch,
    /// The blob names the bytes of the revision, so its media type is the media type of the file;
    /// the two differed.
    #[error("blob media_type does not equal media_type")]
    BlobMediaTypeMismatch,
    /// The blob belongs to another service than `ratatoskr-notes`, which alone stores note bytes.
    #[error("blob owner_service must be `ratatoskr-notes`, found `{owner_service}`")]
    ForeignBlobOwner {
        /// The owner the blob named.
        owner_service: String,
    },
    /// The connection sequence was zero; it starts at 1 for each connection.
    #[error("connection_sequence must be at least 1")]
    ConnectionSequenceZero,
}
