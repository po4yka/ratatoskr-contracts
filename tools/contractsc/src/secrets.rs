//! The fixture secret and PII scanner (`ARCHITECTURE.md` S12, `SECURITY.md`,
//! `THREAT_MODEL.md` "Sensitive fixture leak").
//!
//! Every byte under `fixtures/**` is scanned. A match is a [`Finding::Secret`] and fails `check`.
//! The scanner is deliberately blunt: a false positive is a five-minute conversation, a false
//! negative is a published credential.

use std::path::Path;
use std::sync::LazyLock;

use crate::check::Finding;

/// One named pattern the scanner refuses to see in a fixture.
#[derive(Debug)]
pub struct Rule {
    /// Stable name, reported in the finding.
    pub name: &'static str,
    /// The compiled matcher.
    pub matcher: &'static LazyLock<regex::Regex>,
}

/// Compiles one contract-constant pattern.
///
/// Unicode mode is off: the workspace pins `regex` to `default-features = false, features =
/// ["std"]` (decision D19), so `unicode-case` and `unicode-perl` are absent and `(?i)`, `\s`,
/// `\d` and `\b` must be ASCII. Every pattern here is ASCII by construction — a credential, a
/// UUID and an E.164 number have no non-ASCII spelling — so ASCII semantics are the intended
/// semantics, not a concession.
macro_rules! pattern {
    ($name:ident, $source:expr) => {
        #[allow(
            clippy::expect_used,
            reason = "the pattern is a compile-time constant; a build whose scanner does not \
                      compile is broken before it can gate anything"
        )]
        static $name: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::RegexBuilder::new($source)
                .unicode(false)
                .build()
                .expect("scanner pattern must be a valid regular expression")
        });
    };
}

pattern!(PRIVATE_KEY, r"-----BEGIN [A-Z ]*PRIVATE KEY");
pattern!(OPENAI_KEY, r"sk-[A-Za-z0-9]{16,}");
pattern!(GITHUB_TOKEN, r"gh[po]_");
pattern!(SLACK_TOKEN, r"xox[baprs]-");
pattern!(AWS_KEY, r"AKIA[0-9A-Z]{16}");
pattern!(BEARER, r"Bearer ");
pattern!(JWT, r"eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.");
pattern!(
    CREDENTIAL_KEY,
    r#"(?i)"(password|token|secret|authorization|cookie|api_key|access_token|refresh_token|session)"\s*:"#
);
pattern!(EMAIL, r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}");
pattern!(PHONE, r"\+[1-9]\d{7,14}\b");
pattern!(HANDLE, r#"(?:^|[\s"(])@[A-Za-z0-9_]{2,}"#);
pattern!(URL, r"https?://");
// `.test` is a reserved special-use domain. A concrete public-permalink contract needs a
// renderable synthetic URL, so permit HTTPS URLs on that domain while continuing to reject any
// provider or user URL in fixtures.
pattern!(SYNTHETIC_TEST_URL, r"https://[A-Za-z0-9.-]+\.test");
pattern!(OBJECT_STORE_URL, r"(?:s3|gs)://");
// A JSON string whose first segment is a filesystem root, e.g. `"/var/lib/blob"` or
// `"/Users/…"`. **Not** every leading-slash string: an RFC 6901 JSON Pointer starts with `/`
// too, and `ratatoskr_error_contracts::FieldPath` is exactly such a pointer, published on the
// wire on purpose (`ARCHITECTURE.md` S5.5, "validation errors identify safe field paths"). A
// pointer of identifier tokens carries no storage location, so flagging one would make the S14
// path rule unimplementable rather than enforceable.
pattern!(
    ABSOLUTE_PATH,
    r#""/(bin|boot|dev|etc|home|lib|media|mnt|opt|private|proc|root|run|sbin|srv|sys|tmp|usr|var|Applications|Library|Network|System|Users|Volumes)(/|")"#
);
pattern!(WINDOWS_PATH, r"[A-Za-z]:\\");
pattern!(
    UUID,
    r"(?i)\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b"
);

/// The reserved synthetic identifier block (§7.1). Every UUID in every fixture must match it, so
/// "no real user identifiers" is a mechanical check and a real identity is obvious in a diff.
///
/// Matched case-insensitively: an uppercase spelling of a reserved-block UUID is still that
/// identity, and rejecting the spelling is the contract's job (fixture
/// `core/event-envelope/invalid/event-id-uppercase-uuid.json`), not the scanner's.
pub const SYNTHETIC_UUID_PATTERN: &str = r"(?i)^018f0000-0000-7000-8000-[0-9a-f]{12}$";

pattern!(
    SYNTHETIC_UUID,
    r"(?i)^018f0000-0000-7000-8000-[0-9a-f]{12}$"
);

/// Every rule, in report order.
#[must_use]
pub fn rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "private-key",
            matcher: &PRIVATE_KEY,
        },
        Rule {
            name: "openai-key",
            matcher: &OPENAI_KEY,
        },
        Rule {
            name: "github-token",
            matcher: &GITHUB_TOKEN,
        },
        Rule {
            name: "slack-token",
            matcher: &SLACK_TOKEN,
        },
        Rule {
            name: "aws-access-key",
            matcher: &AWS_KEY,
        },
        Rule {
            name: "bearer-header",
            matcher: &BEARER,
        },
        Rule {
            name: "jwt",
            matcher: &JWT,
        },
        Rule {
            name: "credential-shaped-key",
            matcher: &CREDENTIAL_KEY,
        },
        Rule {
            name: "email-address",
            matcher: &EMAIL,
        },
        Rule {
            name: "e164-phone-number",
            matcher: &PHONE,
        },
        Rule {
            name: "at-handle",
            matcher: &HANDLE,
        },
        Rule {
            name: "url",
            matcher: &URL,
        },
        Rule {
            name: "object-store-url",
            matcher: &OBJECT_STORE_URL,
        },
        Rule {
            name: "absolute-path",
            matcher: &ABSOLUTE_PATH,
        },
        Rule {
            name: "windows-path",
            matcher: &WINDOWS_PATH,
        },
    ]
}

/// Scans one fixture's text and reports every rule it trips.
#[must_use]
pub fn scan_text(display: &str, text: &str) -> Vec<Finding> {
    let without_synthetic_test_urls = SYNTHETIC_TEST_URL.replace_all(text, "");
    let mut findings: Vec<Finding> = rules()
        .iter()
        .filter(|rule| {
            if rule.name == "url" {
                rule.matcher.is_match(&without_synthetic_test_urls)
            } else {
                rule.matcher.is_match(text)
            }
        })
        .map(|rule| Finding::Secret {
            path: display.to_owned(),
            pattern: rule.name,
        })
        .collect();
    // The float rule is a byte-stability rule, not a secrecy rule: a float is banned because its
    // rendering varies between platforms. An `invalid/` fixture is never rendered — it exists to
    // be rejected — and one of them (`progress-percent-fractional.json`) must carry a float to
    // prove the integer bound holds through deserialization.
    if !display.contains("/invalid/")
        && let Ok(document) = serde_json::from_str::<serde_json::Value>(text)
        && contains_float(&document)
    {
        findings.push(Finding::Secret {
            path: display.to_owned(),
            pattern: "floating-point-number",
        });
    }
    for candidate in UUID.find_iter(text) {
        if !SYNTHETIC_UUID.is_match(candidate.as_str()) {
            findings.push(Finding::Secret {
                path: display.to_owned(),
                pattern: "uuid-outside-the-reserved-synthetic-block",
            });
            break;
        }
    }
    findings
}

/// `true` when any number anywhere in the document is not an integer.
///
/// Checked against the parsed document rather than the raw bytes, because a sub-second instant
/// such as `2026-08-17T10:00:00.123456789Z` is not a float and must not be reported as one.
fn contains_float(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Number(number) => number.is_f64(),
        serde_json::Value::Object(members) => members.values().any(contains_float),
        serde_json::Value::Array(items) => items.iter().any(contains_float),
        _ => false,
    }
}

/// Scans every file under `fixtures/`, whatever its extension.
///
/// Not `walk_json`: `fixtures/invalid-expectations.toml` lives under `fixtures/**` and quotes
/// fixture values verbatim, so a JSON-only walk would leave the one non-JSON file in the tree
/// unscanned. A file that is not UTF-8 is skipped — the regex rules have nothing to say about
/// bytes that are not text.
#[must_use]
pub fn scan_tree(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    for absolute in crate::fixtures::walk_all(&root.join("fixtures")) {
        let display = absolute
            .strip_prefix(root)
            .unwrap_or(&absolute)
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(text) = std::fs::read_to_string(&absolute) else {
            continue;
        };
        findings.extend(scan_text(&display, &text));
    }
    findings
}
