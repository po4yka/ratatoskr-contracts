//! Human-authored text and provider-attribute newtypes.

use ratatoskr_identifiers::wire_string_newtype;

wire_string_newtype! {
    /// A provider handle (screen name) in its bare form, without the `@` presentation prefix.
    ///
    /// No external specification fixes one handle grammar across X, Instagram and Threads, so
    /// this is a bounded attribute alphabet rather than an ADR-0007 clause-3 external grammar:
    /// alphanumerics, dots and underscores. Handles are single-line tokens; the `@` is added by
    /// whichever surface renders them.
    pub struct AuthorHandle {
        pattern  = r"^[A-Za-z0-9._]{1,64}$",
        max_len  = 64,
        examples = ["example_user", "provider.slug.01"],
    }
}

wire_string_newtype! {
    /// A provider-authored or user-authored display name.
    ///
    /// Single-line human text: control characters are banned so a name cannot forge log lines,
    /// but any other Unicode is content. Not machine-parsed; consumers never branch on it.
    pub struct DisplayName {
        pattern  = r"^[^\x00-\x1f\x7f]{1,256}$",
        max_len  = 256,
        examples = ["Example User"],
    }
}

wire_string_newtype! {
    /// Normalized user-authored text: a post body, a caption, media alt text.
    ///
    /// Line breaks are content and survive verbatim (`\n`, `\r\n`, `\t`); every other C0
    /// control and DEL is banned so a post cannot smuggle terminal escapes into a renderer.
    /// The upper bound lives in `MAX_LEN` rather than the pattern: a bounded repetition that
    /// large does not compile, and the newtype already rejects over-long input before the
    /// pattern runs.
    pub struct PostText {
        pattern  = r"^[^\x00-\x08\x0b\x0c\x0e-\x1f\x7f]+$",
        max_len  = 65536,
        examples = ["first line\nsecond line"],
    }
}

wire_string_newtype! {
    /// The canonical HTTPS permalink of a source on its platform.
    ///
    /// A deliberate lower bound: absolute `https://`, no whitespace, no control characters.
    /// Full URL syntax validation belongs to the producer that minted the link; this contract
    /// only guarantees the value is unambiguous to render and store. `http://` is refused —
    /// every supported platform serves permalinks over TLS.
    pub struct PostPermalink {
        pattern  = r"^https://[!-~]{1,2000}$",
        max_len  = 2048,
        examples = ["https://x.com/example_user/status/1234567890"],
    }
}
