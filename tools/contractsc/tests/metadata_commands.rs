//! `commands` metadata registration must agree with `CommandPayload`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{Finding, GENERATOR_VERSION, Metadata, generate, metadata};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

#[test]
fn rejects_mismatched_command_type() {
    let committed = std::fs::read_to_string(repo_root().join(Metadata::FILE_NAME))
        .expect("contracts.toml is committed");
    let synthetic = format!(
        "{committed}\n\
         [[contract]]\n\
         id = \"social.capture_requested\"\n\
         family = \"commands\"\n\
         major_version = 1\n\
         lifecycle = \"proposed\"\n\
         classification = \"internal\"\n\
         owner = \"ratatoskr-platform\"\n\
         producers = [\"ratatoskr-platform\"]\n\
         consumers = [\"ratatoskr-x\"]\n\
         crate_name = \"ratatoskr-social-contracts\"\n\
         canonical_path = \"crates/social-contracts/src/capture.rs\"\n\
         fixtures_dir = \"fixtures/social/social-source-snapshot\"\n\
         summary = \"Synthetic command metadata mismatch fixture.\"\n\
         [[contract.root_type]]\n\
         rust_path = \"ratatoskr_social_contracts::SocialCaptureRequested\"\n\
         output = \"schemas/json-schema/commands/capture-requested.v1.schema.json\"\n\
         schema_id = \"urn:ratatoskr:contracts:commands:v1:SocialCaptureRequested\"\n\
         unknown_policy = \"preserve\"\n\
         privacy = \"boundary_metadata\"\n\
         [contract.command]\n\
         command_type = \"social.capture.other.v1\"\n\
         payload_type = \"ratatoskr_social_contracts::SocialCaptureRequested\"\n"
    );
    let metadata = Metadata::parse(&synthetic).expect("commands metadata must parse");
    let generated = generate(&metadata, GENERATOR_VERSION).expect("schemas generate in memory");
    let findings = metadata::validate(&metadata, &repo_root(), &generated);

    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::Metadata { rule, detail }
                if *rule == "R10" && detail.contains("disagrees with CommandPayload::COMMAND_TYPE")
        )),
        "the mismatched command type must be named by the command-metadata rule: {findings:#?}"
    );
}
