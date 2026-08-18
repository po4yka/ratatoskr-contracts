//! `OperationStatus` — tests O-1 and O-5.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_operation_contracts::OperationStatus;

/// The seven tokens `ARCHITECTURE.md` S5.4 spells, in lifecycle order. `README.md`'s prose
/// "completed, partially completed" is wrong; S5.4 is the authority.
const DOCUMENTED: [(&str, bool); 7] = [
    ("accepted", false),
    ("queued", false),
    ("running", false),
    ("succeeded", true),
    ("partially_succeeded", true),
    ("failed", true),
    ("cancelled", true),
];

/// O-1. Every documented status parses, renders back to the same token, and reports the
/// documented terminality.
#[test]
fn all_seven_documented_statuses_parse() {
    assert_eq!(OperationStatus::ALL.len(), DOCUMENTED.len());

    for (status, (token, terminal)) in OperationStatus::ALL.iter().zip(DOCUMENTED) {
        let wire = format!("\"{token}\"");
        let decoded: OperationStatus =
            serde_json::from_str(&wire).unwrap_or_else(|error| panic!("{token} parses: {error}"));
        assert_eq!(decoded, *status);
        assert_eq!(status.as_str(), token);
        assert_eq!(status.to_string(), token);
        assert_eq!(serde_json::to_string(status).unwrap(), wire);
        assert_eq!(status.is_terminal(), terminal);
    }
}

/// O-5. `DOMAIN.md` invariant 6, **rejected explicitly** branch (spec D6).
///
/// The enum is closed because a client that guesses at an unknown lifecycle state reports
/// unfinished work as finished. The rejection names every legal variant so an operator reading
/// the log knows which build is behind.
#[test]
fn unknown_operation_status_is_rejected() {
    let error = serde_json::from_str::<OperationStatus>("\"paused\"")
        .expect_err("an unknown lifecycle state must not be guessed at");
    let text = error.to_string();
    assert!(text.contains("paused"), "{text}");
    for (token, _) in DOCUMENTED {
        assert!(text.contains(token), "{text} must name {token}");
    }

    // Nor is a differently-cased or differently-spelled variant of a known state accepted.
    assert!(serde_json::from_str::<OperationStatus>("\"Succeeded\"").is_err());
    assert!(serde_json::from_str::<OperationStatus>("\"partiallySucceeded\"").is_err());
    assert!(serde_json::from_str::<OperationStatus>("\"completed\"").is_err());
}
