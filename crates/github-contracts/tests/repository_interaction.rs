//! Repository preview wire-contract tests.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "assertions in a test binary"
)]

use ratatoskr_github_contracts::{
    RepositoryActionRequest, RepositoryActionResult, RepositoryPreviewRequest,
    RepositoryPreviewResponse,
};
use serde_json::json;

/// Preview requests and responses preserve every declared fact and refuse contract drift.
#[test]
fn preview_contract_round_trips_and_refuses_unknown_fields() {
    let request_wire = json!({
        "repository_url": "https://github.com/owner/repository"
    });
    let request: RepositoryPreviewRequest =
        serde_json::from_value(request_wire.clone()).expect("a valid preview request decodes");
    assert_eq!(
        serde_json::to_value(request).expect("the preview request serializes"),
        request_wire,
        "the canonical repository URL must round-trip exactly"
    );

    let response_wire = json!({
        "target": {
            "github_repository_numeric_id": 42,
            "repository_full_name": "owner/repository",
            "canonical_url": "https://github.com/owner/repository"
        },
        "description": "A small repository description.",
        "stargazer_count": 123,
        "primary_language": "Rust",
        "account_ref": "github-account:018f0000-0000-7000-8000-000000000604",
        "available_actions": ["metadata", "track", "star"]
    });
    let response: RepositoryPreviewResponse =
        serde_json::from_value(response_wire.clone()).expect("a valid preview response decodes");
    assert_eq!(
        serde_json::to_value(response).expect("the preview response serializes"),
        response_wire,
        "stable identity and optional display facts must round-trip exactly"
    );

    let mut request_with_unknown = request_wire;
    request_with_unknown
        .as_object_mut()
        .expect("the request fixture is an object")
        .insert("future_field".to_owned(), json!(true));
    let request_rejected =
        serde_json::from_value::<RepositoryPreviewRequest>(request_with_unknown).is_err();

    let mut response_with_unknown = response_wire;
    response_with_unknown
        .as_object_mut()
        .expect("the response fixture is an object")
        .insert("future_field".to_owned(), json!(true));
    let response_rejected =
        serde_json::from_value::<RepositoryPreviewResponse>(response_with_unknown).is_err();

    assert!(
        request_rejected && response_rejected,
        "preview shapes must reject unknown members: request_rejected={request_rejected}, response_rejected={response_rejected}"
    );
}

/// Repository actions are closed, confirmed, idempotent requests without provider credentials.
#[test]
fn repository_action_requires_mode_target_confirmation_and_idempotency() {
    let valid_wire = json!({
        "mode": "star",
        "target": {
            "github_repository_numeric_id": 42,
            "repository_full_name": "owner/repository",
            "canonical_url": "https://github.com/owner/repository"
        },
        "account_ref": "github-account:018f0000-0000-7000-8000-000000000604",
        "confirmation_evidence_ref": "telegram-confirmation:018f0000-0000-7000-8000-000000000605",
        "idempotency_key": "telegram-github-action.018f0000-0000-7000-8000-000000000606"
    });
    let request: RepositoryActionRequest =
        serde_json::from_value(valid_wire.clone()).expect("a valid confirmed star request decodes");
    assert_eq!(
        serde_json::to_value(request).expect("the action request serializes"),
        valid_wire,
        "mode, stable target, account, confirmation, and idempotency must round-trip"
    );

    let mut unknown_mode = valid_wire.clone();
    unknown_mode
        .as_object_mut()
        .expect("the action fixture is an object")
        .insert("mode".to_owned(), json!("mirror"));

    let mut star_without_account = valid_wire.clone();
    star_without_account
        .as_object_mut()
        .expect("the action fixture is an object")
        .remove("account_ref");

    let mut empty_confirmation = valid_wire.clone();
    empty_confirmation
        .as_object_mut()
        .expect("the action fixture is an object")
        .insert("confirmation_evidence_ref".to_owned(), json!(""));

    let mut empty_idempotency = valid_wire.clone();
    empty_idempotency
        .as_object_mut()
        .expect("the action fixture is an object")
        .insert("idempotency_key".to_owned(), json!(""));

    let mut credential_like_member = valid_wire;
    credential_like_member
        .as_object_mut()
        .expect("the action fixture is an object")
        .insert(
            "github_token".to_owned(),
            json!("ghp_must_never_cross_this_boundary"),
        );

    let accepted_invalid_cases = [
        ("unknown_mode", unknown_mode),
        ("star_without_account_ref", star_without_account),
        ("empty_confirmation_evidence_ref", empty_confirmation),
        ("empty_idempotency_key", empty_idempotency),
        ("credential_like_member", credential_like_member),
    ]
    .into_iter()
    .filter_map(|(name, wire)| {
        serde_json::from_value::<RepositoryActionRequest>(wire)
            .is_ok()
            .then_some(name)
    })
    .collect::<Vec<_>>();

    assert!(
        accepted_invalid_cases.is_empty(),
        "action contract accepted invalid cases: {accepted_invalid_cases:?}"
    );
}

/// Component outcomes preserve partial truth and refuse impossible aggregate claims.
#[test]
fn component_outcomes_reject_inconsistent_aggregate_and_backup_claims() {
    let partial_wire = json!({
        "aggregate": "partial",
        "metadata": { "status": "succeeded" },
        "provider_star": { "status": "succeeded" },
        "desired_backup": {
            "status": "failed",
            "reason": "dependency_unavailable"
        }
    });
    let partial: RepositoryActionResult = serde_json::from_value(partial_wire.clone())
        .expect("a star success with backup failure decodes as partial");
    assert_eq!(
        serde_json::to_value(partial).expect("the partial action result serializes"),
        partial_wire,
        "all three component facts must round-trip beside aggregate partial"
    );

    let accepted_backup_wire = json!({
        "aggregate": "succeeded",
        "metadata": { "status": "succeeded" },
        "provider_star": {
            "status": "skipped",
            "reason": "not_applicable"
        },
        "desired_backup": { "status": "accepted" }
    });
    let accepted_backup: RepositoryActionResult =
        serde_json::from_value(accepted_backup_wire.clone())
            .expect("accepted desired backup policy decodes");
    assert_eq!(
        serde_json::to_value(accepted_backup).expect("the accepted backup result serializes"),
        accepted_backup_wire,
        "desired backup acceptance must remain accepted rather than become backup success"
    );

    let mut metadata_accepted = partial_wire.clone();
    *metadata_accepted
        .pointer_mut("/metadata/status")
        .expect("the metadata status exists") = json!("accepted");

    let mut provider_star_accepted = partial_wire.clone();
    *provider_star_accepted
        .pointer_mut("/provider_star/status")
        .expect("the provider-star status exists") = json!("accepted");

    let mut inconsistent_aggregate = partial_wire;
    *inconsistent_aggregate
        .pointer_mut("/aggregate")
        .expect("the aggregate exists") = json!("succeeded");

    let accepted_invalid_cases = [
        ("metadata_accepted", metadata_accepted),
        ("provider_star_accepted", provider_star_accepted),
        ("inconsistent_aggregate", inconsistent_aggregate),
    ]
    .into_iter()
    .filter_map(|(name, wire)| {
        serde_json::from_value::<RepositoryActionResult>(wire)
            .is_ok()
            .then_some(name)
    })
    .collect::<Vec<_>>();

    assert!(
        accepted_invalid_cases.is_empty(),
        "component result accepted invalid cases: {accepted_invalid_cases:?}"
    );
}
