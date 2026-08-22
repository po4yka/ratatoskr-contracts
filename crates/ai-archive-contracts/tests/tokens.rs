//! Open tokens and value newtypes (`AiProvider`, parser stamps, titles and text).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_ai_archive_contracts::{
    AiFileName, AiProvider, AiText, AiTitle, ParserName, ParserVersion,
};

/// The provider vocabulary is open: known tokens parse, unknown-but-grammatical tokens are
/// preserved verbatim, and anything outside the grammar is refused.
#[test]
fn provider_is_an_open_validated_token() {
    for known in ["chatgpt", "claude"] {
        let parsed = AiProvider::parse(known).expect("a known provider parses");
        assert_eq!(parsed.as_str(), known);
    }

    let future = AiProvider::parse("gemini_workspace").expect("a grammatical token parses");
    assert_eq!(future.as_str(), "gemini_workspace");

    for rejected in ["", "ChatGPT", "open ai", "chat-gpt-4"] {
        assert!(
            AiProvider::parse(rejected).is_err(),
            "{rejected} must not parse as an AiProvider"
        );
    }
}

/// Parser stamps are opaque bounded tokens; nothing may parse them for semantics.
#[test]
fn parser_stamps_are_opaque_bounded_tokens() {
    let name = ParserName::parse("chatgpt_export").expect("a parser name parses");
    assert_eq!(name.as_str(), "chatgpt_export");
    assert!(ParserName::parse("Claude Export").is_err());
    assert!(ParserName::parse("").is_err());

    for version in ["1.4.2", "2026.08.1", "9f83a1c0"] {
        let parsed = ParserVersion::parse(version).expect("a parser version parses");
        assert_eq!(parsed.as_str(), version);
    }
    for rejected in ["1 4 2", "", "v1\t3"] {
        assert!(
            ParserVersion::parse(rejected).is_err(),
            "{rejected} must not parse as a ParserVersion"
        );
    }
}

/// Titles are single-line control-free human text.
#[test]
fn titles_reject_control_characters() {
    let title = AiTitle::parse("Rust ownership notes").expect("a title parses");
    assert_eq!(title.as_str(), "Rust ownership notes");
    assert!(AiTitle::parse("bad\u{0007}bell").is_err());
    assert!(AiTitle::parse("").is_err());
}

/// Body text keeps line breaks verbatim but bans the other C0 controls and DEL.
#[test]
fn text_preserves_line_breaks_and_rejects_other_controls() {
    let text = AiText::parse("first line\nsecond line").expect("multi-line text parses");
    let wire = serde_json::to_string(&text).unwrap();
    let decoded: AiText = serde_json::from_str(&wire).expect("and deserializes");
    assert_eq!(decoded.as_str(), "first line\nsecond line");

    assert!(AiText::parse("tab\tstays").is_ok());
    assert!(AiText::parse("vertical\u{000b}tab").is_err());
    assert!(AiText::parse("delete\u{007f}char").is_err());
}

/// File names are bounded single-segment display names, never paths.
#[test]
fn file_names_are_not_paths() {
    let name = AiFileName::parse("quarterly-report.pdf").expect("a file name parses");
    assert_eq!(name.as_str(), "quarterly-report.pdf");

    for rejected in ["/etc/passwd", "a/b.txt", "..", ""] {
        assert!(
            AiFileName::parse(rejected).is_err(),
            "{rejected} must not parse as an AiFileName"
        );
    }
}
