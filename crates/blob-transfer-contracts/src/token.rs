//! The opaque resumption token: [`UploadResumptionToken`].

ratatoskr_identifiers::wire_string_newtype! {
    /// Server-issued handle naming one upload session. Required on every chunk receipt, status
    /// and finalize call after opening.
    ///
    /// Opaque by contract: a client compares tokens, stores them and presents them back, and
    /// never parses one. The issuing service mints it unguessable — at least 128 bits of entropy
    /// — because whoever holds the token drives the session; expiry is the server's decision,
    /// carried beside the token as `expires_at`, and an expired token is refused with
    /// `blob_transfer.session_expired`, distinct from `blob_transfer.session_unknown`.
    pub struct UploadResumptionToken {
        pattern  = r"^rst_[0-9a-v]{24,64}$",
        max_len  = 68,
        examples = ["rst_0v8k4a2j9pm1d7n5tp3es6uabfhij1cm4nop5qrs"],
    }
}
