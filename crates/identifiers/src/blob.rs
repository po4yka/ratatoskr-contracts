//! The opaque, content-addressed blob handle.

use crate::wire_string_newtype;

wire_string_newtype! {
    /// Opaque, content-addressed handle to stored bytes.
    ///
    /// Carries no bucket, host, filesystem path, signed URL or expiry: `ARCHITECTURE.md` S14
    /// forbids all of them ("Blob references are opaque and do not expose filesystem paths or
    /// signed storage URLs"). Resolving a handle to bytes is a service concern and needs separate
    /// authorization.
    ///
    /// This tightens S5.1's `BlobRef(pub String)`; S5.1's own requirement list says identifier
    /// string formats are "stable and validated", which a public `String` cannot guarantee.
    /// `INTERFACES.md`'s richer blob description (hash, size, media type, authorization metadata)
    /// is a descriptor, not a handle; the descriptor arrives with Document IR in m5.
    ///
    /// The two-colon shape is deliberate: parsed as an `EntityRef` it is kind `blob`, local id
    /// `sha256:<hex>`.
    pub struct BlobRef {
        pattern  = r"^blob:sha256:[0-9a-f]{64}$",
        max_len  = 76,
        examples = ["blob:sha256:0000000000000000000000000000000000000000000000000000000000000000"],
    }
}

impl BlobRef {
    /// The 64-character lowercase hex digest.
    #[must_use]
    pub fn sha256_hex(&self) -> &str {
        const PREFIX: &str = "blob:sha256:";
        self.as_str().get(PREFIX.len()..).unwrap_or("")
    }
}
