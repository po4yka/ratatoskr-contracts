//! Registry closure for channel-digest commands and recap facts.
#![allow(clippy::expect_used, clippy::panic, reason = "test diagnostics")]

use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{GENERATOR_VERSION, Metadata, generate, registry};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("contractsc sits under tools")
        .to_path_buf()
}

#[test]
fn channel_digest_contracts_are_registered_with_exact_authority() {
    let root = repo_root();
    let metadata_text =
        std::fs::read_to_string(root.join(Metadata::FILE_NAME)).expect("contracts metadata");
    let metadata = Metadata::parse(&metadata_text).expect("metadata parses");
    assert!(
        metadata
            .services
            .known
            .contains(&"ratatoskr-channel-digests".to_owned()),
        "digest service must be in the closed service vocabulary"
    );

    let paths: Vec<&str> = registry::root_types()
        .iter()
        .map(|root| root.rust_path)
        .filter(|path| path.starts_with("ratatoskr_channel_digest_contracts::"))
        .collect();
    assert_eq!(
        paths,
        [
            "ratatoskr_channel_digest_contracts::ChannelDigestRunRequested",
            "ratatoskr_channel_digest_contracts::ChannelDigestSubscriptionSetRequested",
            "ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapCompleted",
            "ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapFailed",
            "ratatoskr_channel_digest_contracts::KnowledgeChannelDigestRecapRequested",
        ]
    );

    let digest_contracts: Vec<_> = metadata
        .contracts
        .iter()
        .filter(|contract| contract.crate_name == "ratatoskr-channel-digest-contracts")
        .collect();
    assert_eq!(
        digest_contracts.len(),
        5,
        "one governed entry per payload root"
    );

    let generated = generate(&metadata, GENERATOR_VERSION).expect("contracts generate");
    for contract in digest_contracts {
        assert_eq!(contract.root_types.len(), 1);
        let declared = contract.root_types.first().expect("one root");
        assert!(generated.contains_key(Path::new(&declared.output)));
        assert!(root.join(&contract.fixtures_dir).join("valid").is_dir());
        assert_eq!(format!("{:?}", declared.privacy), "BoundaryMetadata");
        match contract.id.as_str() {
            "channel_digest.subscription_set_requested" | "channel_digest.run_requested" => {
                assert_eq!(contract.producers, ["ratatoskr-platform"]);
                assert_eq!(contract.consumers, ["ratatoskr-channel-digests"]);
                assert!(contract.command.is_some());
            }
            "knowledge.channel_digest_recap_requested" => {
                assert_eq!(contract.producers, ["ratatoskr-channel-digests"]);
                assert_eq!(contract.consumers, ["ratatoskr-knowledge"]);
                assert!(contract.command.is_some());
            }
            "knowledge.channel_digest_recap_completed"
            | "knowledge.channel_digest_recap_failed" => {
                assert_eq!(contract.producers, ["ratatoskr-knowledge"]);
                assert_eq!(contract.consumers, ["ratatoskr-channel-digests"]);
                assert!(contract.event.is_some());
            }
            other => panic!("unexpected channel-digest contract {other}"),
        }
    }
}
