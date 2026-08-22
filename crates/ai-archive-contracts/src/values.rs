//! Human-authored text value newtypes.

use ratatoskr_identifiers::wire_string_newtype;

wire_string_newtype! {
    /// A provider- or user-authored single-line title of a project or conversation.
    ///
    /// Control characters are banned so a title cannot forge log lines; every other Unicode is
    /// content. Not machine-parsed; consumers never branch on it.
    pub struct AiTitle {
        pattern  = r"^[^\x00-\x1f\x7f]{1,256}$",
        max_len  = 256,
        examples = ["Rust ownership notes"],
    }
}

wire_string_newtype! {
    /// Multi-line normalized text: project instructions and descriptions.
    ///
    /// Line breaks are content and survive verbatim (`\n`, `\r\n`, `\t`); every other C0
    /// control and DEL is banned so text cannot smuggle terminal escapes into a renderer. The
    /// upper bound lives in `MAX_LEN` rather than the pattern: a bounded repetition that large
    /// does not compile, and the newtype already rejects over-long input before the pattern
    /// runs.
    pub struct AiText {
        pattern  = r"^[^\x00-\x08\x0b\x0c\x0e-\x1f\x7f]+$",
        max_len  = 65536,
        examples = ["first line\nsecond line"],
    }
}

wire_string_newtype! {
    /// A display name of one stored asset file, as the export named it.
    ///
    /// A single path segment, never a path: no slash separates directories here, because this
    /// contract names a stored blob by reference and must never carry a storage location. The
    /// first character is alphanumeric or an underscore, so no accepted spelling collides with a
    /// relative path token such as `.` or `..`; separators and control characters are refused.
    pub struct AiFileName {
        pattern  = r"^[A-Za-z0-9_][^/\\\x00-\x1f\x7f]{0,254}$",
        max_len  = 255,
        examples = ["quarterly-report.pdf", "screenshot 2026-08-01.png"],
    }
}
