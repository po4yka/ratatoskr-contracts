//! `WireTimestamp` — test I-8, the canonicality rule (spec D11).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::{IdentifierError, WireTimestamp};

/// I-8. Exactly one spelling per instant: parsing accepts a string only when re-rendering the
/// parsed value reproduces the input byte for byte.
#[test]
fn accepts_only_its_own_canonical_form() {
    // `ARCHITECTURE.md` S5.2's own example, byte for byte.
    for accepted in ["2026-08-17T10:00:00Z", "2026-08-17T10:00:00.123456789Z"] {
        let parsed = WireTimestamp::parse(accepted).expect("the canonical form is accepted");
        assert_eq!(parsed.to_wire(), accepted);
        assert_eq!(parsed.to_string(), accepted);
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            format!("\"{accepted}\""),
            "the wire form is the canonical text"
        );
    }

    // Parses as RFC 3339, but is a second spelling of an instant that already has one.
    for non_canonical in [
        "2026-08-17T12:00:00+02:00",
        "2026-08-17T10:00:00+00:00",
        "2026-08-17T10:00:00.120Z",
        "2026-08-17T10:00:00.000Z",
        "2026-08-17T10:00:00z",
    ] {
        let error = WireTimestamp::parse(non_canonical).expect_err("a second spelling is refused");
        assert!(
            matches!(error, IdentifierError::NonCanonicalInstant { .. }),
            "{non_canonical} must be reported as non-canonical, got {error}"
        );
        assert!(error.to_string().contains("not in canonical form"));
    }

    // Not an instant at all: no offset, so no instant is designated.
    for malformed in ["2026-08-17T10:00:00", "2026-08-17", "not-a-time"] {
        assert!(
            WireTimestamp::parse(malformed).is_err(),
            "{malformed} must not parse as a WireTimestamp"
        );
    }

    // A space separator is rejected however jiff classifies it.
    assert!(WireTimestamp::parse("2026-08-17 10:00:00Z").is_err());
}

/// The published `PATTERN` is a lower bound the canonical rule tightens: every canonical
/// rendering matches it, and it alone is not enough to prove canonicality.
#[test]
fn published_pattern_matches_every_canonical_rendering() {
    let pattern = regex::Regex::new(WireTimestamp::PATTERN).expect("PATTERN compiles");
    for accepted in ["2026-08-17T10:00:00Z", "2026-08-17T10:00:00.123456789Z"] {
        assert!(pattern.is_match(accepted));
    }
    // The pattern accepts a spelling the Rust layer rejects — see ADR-0001's "the schema is a
    // lower bound" consequence.
    assert!(pattern.is_match("2026-08-17T10:00:00.000Z"));
    assert!(WireTimestamp::parse("2026-08-17T10:00:00.000Z").is_err());
}

/// `from_jiff`/`as_jiff` are the producer-side entry points and lose nothing.
#[test]
fn wraps_and_unwraps_a_jiff_instant() {
    let parsed = WireTimestamp::parse("2026-08-17T10:00:00Z").expect("canonical");
    assert_eq!(WireTimestamp::from_jiff(parsed.as_jiff()), parsed);
    assert!(WireTimestamp::now().as_jiff() >= parsed.as_jiff());
}
