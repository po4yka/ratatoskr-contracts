//! The four closed vocabularies (spec: authority, completeness and availability requirements).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_social_contracts::{
    AcquisitionMethod, CaptureCompleteness, SavedAuthority, UpstreamAvailability,
};

#[test]
fn every_spelled_variant_parses_and_round_trips() {
    for method in [
        "official_api",
        "share_extension",
        "browser_extension",
        "public_resolution",
        "data_export",
        "legacy_import",
    ] {
        let wire = format!("\"{method}\"");
        let parsed: AcquisitionMethod = serde_json::from_str(&wire).expect(method);
        assert_eq!(serde_json::to_string(&parsed).unwrap(), wire);
    }

    for authority in [
        "authoritative_platform_state",
        "explicit_user_capture",
        "export_observation",
        "legacy_observation",
    ] {
        let wire = format!("\"{authority}\"");
        let parsed: SavedAuthority = serde_json::from_str(&wire).expect(authority);
        assert_eq!(serde_json::to_string(&parsed).unwrap(), wire);
    }

    for completeness in ["complete", "partial"] {
        let wire = format!("\"{completeness}\"");
        let parsed: CaptureCompleteness = serde_json::from_str(&wire).expect(completeness);
        assert_eq!(serde_json::to_string(&parsed).unwrap(), wire);
    }

    for availability in ["available", "unavailable", "deleted_upstream"] {
        let wire = format!("\"{availability}\"");
        let parsed: UpstreamAvailability = serde_json::from_str(&wire).expect(availability);
        assert_eq!(serde_json::to_string(&parsed).unwrap(), wire);
    }
}

#[test]
fn unknown_values_are_rejected_not_guessed() {
    // Misreading how a source arrived, or what its saved-state claim is worth, is exactly the
    // Instagram-phantom-bookmark failure mode; an unknown value must stop processing.
    for rejected in [
        ("carrier_pigeon", "AcquisitionMethod"),
        ("platform_says_so", "SavedAuthority"),
        ("half", "CaptureCompleteness"),
        ("shadowbanned", "UpstreamAvailability"),
    ] {
        let wire = format!("\"{}\"", rejected.0);
        let error = match rejected.1 {
            "AcquisitionMethod" => serde_json::from_str::<AcquisitionMethod>(&wire)
                .err()
                .map(|e| e.to_string()),
            "SavedAuthority" => serde_json::from_str::<SavedAuthority>(&wire)
                .err()
                .map(|e| e.to_string()),
            "CaptureCompleteness" => serde_json::from_str::<CaptureCompleteness>(&wire)
                .err()
                .map(|e| e.to_string()),
            _ => serde_json::from_str::<UpstreamAvailability>(&wire)
                .err()
                .map(|e| e.to_string()),
        };
        let error = error.unwrap_or_else(|| panic!("{} must be rejected", rejected.0));
        assert!(
            error.contains("unknown variant"),
            "{}: expected an unknown-variant error, got {error}",
            rejected.0
        );
    }
}
