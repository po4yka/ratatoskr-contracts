//! SITE-028: approval identity is a first-version boundary, with absent legacy bindings preserved.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test assertions report contract failures"
)]

use ratatoskr_sites_contracts::{DeployRequest, SiteApprovalChanged, SiteManifest};
use serde_json::{Value, json};

fn manifest() -> Value {
    json!({
        "site_id": "docs", "hostname": "docs.example.test", "publications": ["handbook"],
        "capabilities": ["pages.read"],
        "github_repository": {"repository": "owner/docs-site", "repository_id": "1420131804"}
    })
}

fn accepts(schema: &Value, value: &Value) -> bool {
    jsonschema::validator_for(schema)
        .expect("the type emits a valid schema")
        .is_valid(value)
}

#[test]
fn repository_identity_is_bound() {
    let value = manifest();
    let parsed = serde_json::from_value::<SiteManifest>(value.clone());
    assert!(
        parsed.is_ok(),
        "the manifest must accept the new repository identity: {parsed:?}"
    );
    assert_eq!(
        serde_json::to_value(parsed.expect("accepted manifest")).expect("serializes"),
        value
    );
    let schema =
        serde_json::to_value(schemars::schema_for!(SiteManifest)).expect("schema serializes");
    assert!(accepts(&schema, &value));

    for binding in [
        json!({"repository": "Owner/docs-site", "repository_id": "1420131804"}),
        json!({"repository": "https://github.com/owner/docs-site", "repository_id": "1420131804"}),
        json!({"repository": "owner/docs-site", "repository_id": "0"}),
        json!({"repository": "owner/docs-site", "repository_id": "01"}),
        json!({"repository": "owner/docs-site", "repository_id": 1_420_131_804_u64}),
        json!({"repository": "owner/docs-site"}),
        json!({"repository": "owner/docs-site", "repository_id": "1", "workflow": "caller-controlled"}),
    ] {
        let mut invalid = value.clone();
        invalid
            .as_object_mut()
            .expect("manifest object")
            .insert("github_repository".to_owned(), binding);
        assert!(
            serde_json::from_value::<SiteManifest>(invalid.clone()).is_err(),
            "serde accepted {invalid}"
        );
        assert!(!accepts(&schema, &invalid), "schema accepted {invalid}");
    }

    let event = json!({
        "site_id": "docs", "manifest_sha256": "a".repeat(64), "capabilities": ["pages.read"],
        "hostname": "docs.example.test", "state": "approved", "github_repository": value.get("github_repository").expect("binding exists")
    });
    let parsed = serde_json::from_value::<SiteApprovalChanged>(event.clone());
    assert!(
        parsed.is_ok(),
        "the approval event must bind the same repository: {parsed:?}"
    );
    assert_eq!(
        serde_json::to_value(parsed.expect("accepted event")).expect("serializes"),
        event
    );
}

#[test]
fn a_legacy_manifest_keeps_its_exact_wire_shape() {
    let mut value = manifest();
    value
        .as_object_mut()
        .expect("manifest is an object")
        .remove("github_repository");
    let parsed: SiteManifest =
        serde_json::from_value(value.clone()).expect("legacy manifest decodes");
    assert_eq!(serde_json::to_value(parsed).expect("serializes"), value);
}

#[test]
fn deploy_request_is_digest_only() {
    let value = json!({
        "commit": "a".repeat(40), "artifact_sha256": "b".repeat(64),
        "attestation_bundle": {"mediaType": "application/vnd.dev.sigstore.bundle.v0.3+json"}
    });
    let parsed = serde_json::from_value::<DeployRequest>(value.clone());
    assert!(
        parsed.is_ok(),
        "the request must accept an immutable digest and proof: {parsed:?}"
    );
    assert_eq!(
        serde_json::to_value(parsed.expect("accepted request")).expect("serializes"),
        value
    );
    let schema =
        serde_json::to_value(schemars::schema_for!(DeployRequest)).expect("schema serializes");
    assert!(accepts(&schema, &value));
    let mut invalid = Vec::new();
    for digest in [
        "sha256:short",
        "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "sha-latest",
        "https://another.example/a",
    ] {
        let mut changed = value.clone();
        changed
            .as_object_mut()
            .expect("request object")
            .insert("artifact_sha256".to_owned(), json!(digest));
        invalid.push(changed);
    }
    for name in ["host", "url", "repository", "tag"] {
        let mut changed = value.clone();
        changed
            .as_object_mut()
            .expect("request object")
            .insert(name.to_owned(), json!("caller-controlled"));
        invalid.push(changed);
    }
    for name in ["commit", "artifact_sha256", "attestation_bundle"] {
        let mut changed = value.clone();
        changed
            .as_object_mut()
            .expect("request object")
            .remove(name);
        invalid.push(changed);
    }
    let mut null_proof = value.clone();
    null_proof
        .as_object_mut()
        .expect("request object")
        .insert("attestation_bundle".to_owned(), Value::Null);
    invalid.push(null_proof);
    for value in invalid {
        assert!(
            serde_json::from_value::<DeployRequest>(value.clone()).is_err(),
            "serde accepted {value}"
        );
        assert!(!accepts(&schema, &value), "schema accepted {value}");
    }
}
