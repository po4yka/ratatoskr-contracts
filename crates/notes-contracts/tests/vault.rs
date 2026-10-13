//! The vault connection events and their envelope composition.

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes run on values the test itself built, so a \
              missing member panics the test, which is the reporting mechanism"
)]

mod common;

use ratatoskr_event_envelope::{EventEnvelope, EventPayload, EventType};
use ratatoskr_identifiers::dropped_field_pointers;
use ratatoskr_notes_contracts::{
    VaultChannel, VaultConnected, VaultDisconnected, VaultDisplayName,
};

use common::{VAULT_ENVELOPE, connected};

/// Both payloads are registered facts with a past-tense action at major 1.
#[test]
fn event_type_constants_are_the_registered_names() {
    for (name, action) in [
        (VaultConnected::EVENT_TYPE, "connected"),
        (VaultDisconnected::EVENT_TYPE, "disconnected"),
    ] {
        let parsed = EventType::parse(name).expect("the registered name parses");
        assert_eq!(parsed.bounded_context(), "notes");
        assert_eq!(parsed.aggregate(), "vault");
        assert_eq!(parsed.action(), action);
        assert_eq!(parsed.major(), 1);
        assert!(parsed.action_looks_past_tense());
    }
}

/// A connection travels inside a real envelope, owned by the tenant and aggregated on the
/// connection, and comes back typed and whole.
#[test]
fn connection_travels_inside_a_real_envelope() {
    let payload = connected(VaultChannel::Plugin);

    let mut envelope =
        EventEnvelope::from_json(VAULT_ENVELOPE.as_bytes()).expect("the envelope parses");
    envelope.set_payload(&payload).expect("a JSON object body");
    assert_eq!(envelope.event_type, VaultConnected::event_type());
    assert_eq!(
        envelope.aggregate_id.to_wire(),
        "notes_connection:018f0000-0000-7000-8000-000000000b01"
    );
    assert_eq!(
        envelope
            .tenant_id
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("user:018f0000-0000-7000-8000-000000000005"),
        "the owner rides in the envelope, not in the payload"
    );

    let wire = envelope.to_canonical_json().expect("re-serializes");
    let reparsed = EventEnvelope::from_json(wire.as_bytes()).expect("round trips");
    assert_eq!(reparsed, envelope);
    let sent: serde_json::Value = serde_json::from_str(&wire).expect("JSON");
    let received = serde_json::to_value(&reparsed).expect("re-serialize");
    assert_eq!(
        dropped_field_pointers(&sent, &received),
        Vec::<String>::new()
    );
    assert_eq!(
        reparsed.payload_as::<VaultConnected>().expect("typed"),
        payload
    );
}

/// A disconnection round trips through an envelope with every channel word.
#[test]
fn disconnection_round_trips_with_every_channel() {
    for (channel, word) in [
        (VaultChannel::Sync, "sync"),
        (VaultChannel::Plugin, "plugin"),
        (VaultChannel::Cli, "cli"),
    ] {
        let source = connected(channel);
        let payload = VaultDisconnected {
            connection_id: source.connection_id,
            channel: source.channel,
            vault_display_name: source.vault_display_name,
        };
        let value = serde_json::to_value(&payload).expect("serializes");
        assert_eq!(value["channel"], word);

        let mut envelope =
            EventEnvelope::from_json(VAULT_ENVELOPE.as_bytes()).expect("the envelope parses");
        envelope.set_payload(&payload).expect("a JSON object body");
        assert_eq!(envelope.event_type, VaultDisconnected::event_type());
        let decoded = envelope
            .payload_as::<VaultDisconnected>()
            .expect("the payload comes back typed");
        assert_eq!(decoded, payload);
    }
}

/// A member outside the schema is refused, and so is an unknown channel word.
#[test]
fn unknown_members_and_channels_are_refused() {
    let mut value = serde_json::to_value(connected(VaultChannel::Sync)).expect("serializes");
    value["vault_path"] = serde_json::json!("/home/owner/Personal");
    let error = serde_json::from_value::<VaultConnected>(value.clone()).expect_err("extra member");
    assert!(
        error.to_string().contains("unknown field `vault_path`"),
        "{error}"
    );
    serde_json::from_value::<VaultDisconnected>(value).expect_err("extra member");

    let mut value = serde_json::to_value(connected(VaultChannel::Sync)).expect("serializes");
    value["channel"] = serde_json::json!("webdav");
    serde_json::from_value::<VaultConnected>(value).expect_err("unknown channel");
}

/// The display name is 1 to 255 characters with no control character.
#[test]
fn display_names_are_bounded_and_printable() {
    assert!(VaultDisplayName::parse(&"é".repeat(255)).is_ok());
    assert!(VaultDisplayName::parse(&"a".repeat(256)).is_err());
    assert!(VaultDisplayName::parse("").is_err());
    assert!(VaultDisplayName::parse("line\nbreak").is_err());
    assert!(VaultDisplayName::parse("tab\there").is_err());
}
