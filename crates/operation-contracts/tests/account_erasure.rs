//! Account-erasure command and acknowledgement wire-contract tests.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use std::path::{Path, PathBuf};

use ratatoskr_identifiers::dropped_field_pointers;
use ratatoskr_operation_contracts::{AccountErasureAcknowledged, AccountErasureOutcome};

fn contract_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn schema(path: &str) -> serde_json::Value {
    let path = contract_root().join(path);
    assert!(
        path.is_file(),
        "{} must exist as a generated account-erasure contract schema",
        path.display()
    );
    serde_json::from_str(
        &std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{} must be JSON: {error}", path.display()))
}

fn properties(schema: &serde_json::Value) -> &serde_json::Map<String, serde_json::Value> {
    schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .expect("the contract schema is an object with properties")
}

#[test]
fn account_erasure_command_and_acknowledgement_are_redacted_and_owner_scoped() {
    let command = schema("schemas/json-schema/commands/account-erasure-requested.v1.schema.json");
    let acknowledgement =
        schema("schemas/events/identity.account.erasure_acknowledged.v1.schema.json");
    let command_envelope = schema("schemas/json-schema/core/command-envelope.v1.schema.json");
    let event_envelope = schema("schemas/json-schema/core/event-envelope.v1.schema.json");

    for (name, contract) in [
        ("account-erasure command", &command),
        ("account-erasure acknowledgement", &acknowledgement),
    ] {
        let members = properties(contract);
        assert!(
            members.contains_key("operation_id"),
            "{name} must identify the durable erasure operation"
        );
        for forbidden in [
            "confirmation",
            "idempotency_key",
            "provider_account_id",
            "provider_response",
            "credential",
            "secret",
            "content",
        ] {
            assert!(
                !members.contains_key(forbidden),
                "{name} must not expose {forbidden}"
            );
        }
    }

    assert!(
        properties(&acknowledgement).contains_key("outcome"),
        "an acknowledgement must distinguish verified and incomplete terminal outcomes"
    );

    for (name, envelope) in [
        ("command envelope", &command_envelope),
        ("event envelope", &event_envelope),
    ] {
        assert!(
            properties(envelope).contains_key("tenant_id"),
            "{name} must carry the owner scope for account erasure"
        );
    }
}

#[test]
fn acknowledgement_preserves_unknown_additive_members() {
    let path = contract_root().join(
        "fixtures/events/identity.account.erasure_acknowledged.v1/compat/old-consumer-new-producer/with-future-member.json",
    );
    assert!(
        path.is_file(),
        "{} must preserve a newer acknowledgement member",
        path.display()
    );

    let bytes = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    let input: serde_json::Value =
        serde_json::from_str(&bytes).expect("the compatibility fixture is JSON");
    let acknowledgement: AccountErasureAcknowledged =
        serde_json::from_value(input.clone()).expect("the newer acknowledgement must parse");

    assert_eq!(
        acknowledgement.outcome,
        AccountErasureOutcome::IncompleteExternalGrantRevocation
    );
    let reemitted = serde_json::to_value(acknowledgement).expect("the acknowledgement re-emits");
    assert_eq!(
        dropped_field_pointers(&input, &reemitted),
        Vec::<String>::new(),
        "a newer acknowledgement member must survive an older consumer"
    );
}
