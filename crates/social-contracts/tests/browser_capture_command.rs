//! Browser-originated social capture command contract.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_event_envelope::{CommandEnvelope, CommandPayload};
use ratatoskr_social_contracts::{
    AcquisitionMethod, SavedAuthority, SocialCaptureProvider, SocialCaptureRequested,
};

const COMMAND: &str = r#"{
  "command_id": "018f0000-0000-7000-8000-000000000001",
  "command_type": "social.capture.requested.v1",
  "issued_at": "2026-08-27T10:00:00Z",
  "producer": "ratatoskr-platform",
  "aggregate_id": "operation:018f0000-0000-7000-8000-000000000002",
  "correlation_id": "operation:018f0000-0000-7000-8000-000000000002",
  "tenant_id": "user:018f0000-0000-7000-8000-000000000003",
  "schema_version": 1,
  "payload": {
    "operation_id": "018f0000-0000-7000-8000-000000000002",
    "idempotency_key": {
      "algorithm": "sha256",
      "hex": "0000000000000000000000000000000000000000000000000000000000000000"
    },
    "original_permalink": "https://x.com/ratatoskr/status/1",
    "captured_at": "2026-08-27T10:00:00Z",
    "provider": "x",
    "acquisition": "browser_extension",
    "saved_authority": "explicit_user_capture"
  }
}
"#;

#[test]
fn round_trips_closed_provenance() {
    for (provider, expected) in [
        ("x", SocialCaptureProvider::X),
        ("instagram", SocialCaptureProvider::Instagram),
        ("threads", SocialCaptureProvider::Threads),
    ] {
        let command = CommandEnvelope::from_json(
            COMMAND
                .replace(
                    "\"provider\": \"x\"",
                    &format!("\"provider\": \"{provider}\""),
                )
                .replace(
                    "https://x.com/ratatoskr/status/1",
                    &format!("https://{provider}.example/ratatoskr/1"),
                )
                .as_bytes(),
        )
        .expect("a canonical command envelope must parse");
        let capture = command
            .payload_as::<SocialCaptureRequested>()
            .expect("the social capture payload must parse");

        assert_eq!(
            SocialCaptureRequested::COMMAND_TYPE,
            "social.capture.requested.v1"
        );
        assert_eq!(capture.provider, expected);
        assert_eq!(capture.acquisition, AcquisitionMethod::BrowserExtension);
        assert_eq!(capture.saved_authority, SavedAuthority::ExplicitUserCapture);

        let serialized = serde_json::to_value(&capture).expect("the payload must serialize");
        for forbidden in [
            "credentials",
            "cookies",
            "browser_session",
            "page_content",
            "saved",
        ] {
            assert!(
                serialized.get(forbidden).is_none(),
                "{forbidden} is not permitted in a browser social-capture payload"
            );
        }
    }
}

#[test]
fn refuses_an_unknown_social_provider() {
    let command = CommandEnvelope::from_json(
        COMMAND
            .replace("\"provider\": \"x\"", "\"provider\": \"mastodon\"")
            .as_bytes(),
    )
    .expect("the outer command envelope does not classify provider names");

    assert!(
        command.payload_as::<SocialCaptureRequested>().is_err(),
        "a provider outside the closed social routing vocabulary must be refused"
    );
}
