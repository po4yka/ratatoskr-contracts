//! Advisory delivery hints: quiet-hours window grammar.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_notification_contracts::NotificationRaised;

/// A full valid payload carrying one quiet-hours window, with both offsets supplied.
fn payload_with_quiet_hours(start: u32, end: u32) -> String {
    format!(
        r#"{{
  "notification_id": "018f0000-0000-7000-8000-000000000712",
  "class_registry_version": 1,
  "class": "backup_outcome",
  "recipient": "user:018f0000-0000-7000-8000-000000000005",
  "title": "Nightly backup finished",
  "quiet_hours": {{
    "start_offset_seconds": {start},
    "end_offset_seconds": {end}
  }}
}}"#
    )
}

/// A window whose start equals its end cannot say whether it means an empty day or a full one,
/// so no consumer may be left to guess between opposite behaviours. Refused at parse.
#[test]
fn degenerate_quiet_hours_window_is_refused() {
    let error = serde_json::from_str::<NotificationRaised>(&payload_with_quiet_hours(3600, 3600))
        .expect_err("equal bounds are ambiguous");
    assert!(
        error
            .to_string()
            .contains("must not start and end on the same offset"),
        "failed for the wrong reason: {error}"
    );
}

/// Wrap-around windows - start greater than end, crossing UTC midnight - are the normal shape of
/// an overnight hold and parse fine.
///
/// Green on arrival while the placeholder stands: it pins the property the enforcement step must
/// preserve, exactly like a golden-output pin.
#[test]
fn wrap_around_window_parses() {
    let parsed =
        serde_json::from_str::<NotificationRaised>(&payload_with_quiet_hours(82_800, 25_200))
            .expect("an overnight window is representable");
    let hint = parsed.quiet_hours.expect("the window travels");
    assert_eq!(hint.start_offset_seconds(), 82_800);
    assert_eq!(hint.end_offset_seconds(), 25_200);
}

/// Each offset names a second inside one UTC day; anything beyond is unrepresentable as a daily
/// window and refused at parse. Pinned on the schema side by `quiet-hours-offset-out-of-range.json`.
#[test]
fn out_of_range_offsets_are_refused() {
    for (start, end) in [(86_400, 25_200), (25_200, u32::MAX)] {
        let error =
            serde_json::from_str::<NotificationRaised>(&payload_with_quiet_hours(start, end))
                .expect_err("beyond one UTC day");
        assert!(
            error.to_string().contains("at most 86399 seconds"),
            "case ({start}, {end}) failed for the wrong reason: {error}"
        );
    }
}
