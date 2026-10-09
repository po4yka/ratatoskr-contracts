//! The live-message registry: every message on the fleet bus has the producers and consumers
//! the single bus table of XR-021 CONTRACTS.md S01 gives it.

#![allow(clippy::expect_used, clippy::panic, reason = "test diagnostics")]

use std::path::{Path, PathBuf};

use ratatoskr_contractsc::metadata::Contract;
use ratatoskr_contractsc::{Metadata, registry};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("contractsc sits under tools")
        .to_path_buf()
}

fn metadata() -> Metadata {
    let text = std::fs::read_to_string(repo_root().join(Metadata::FILE_NAME))
        .expect("contracts metadata is readable");
    Metadata::parse(&text).expect("contracts metadata parses")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Command,
    Event,
}

impl Kind {
    /// The bus class prefix the registration kind selects (S01).
    fn class(self) -> &'static str {
        match self {
            Self::Command => "cmd",
            Self::Event => "evt",
        }
    }
}

/// The registration of `message_type` in the metadata: its kind and the contract carrying it.
fn registered<'a>(metadata: &'a Metadata, message_type: &str) -> (Kind, &'a Contract) {
    let found: Vec<(Kind, &Contract)> = metadata
        .contracts
        .iter()
        .filter_map(|contract| {
            if contract
                .command
                .as_ref()
                .is_some_and(|command| command.command_type == message_type)
            {
                Some((Kind::Command, contract))
            } else if contract
                .event
                .as_ref()
                .is_some_and(|event| event.event_type == message_type)
            {
                Some((Kind::Event, contract))
            } else {
                None
            }
        })
        .collect();
    match found.as_slice() {
        [one] => *one,
        other => panic!(
            "{message_type} is registered {} times, expected once",
            other.len()
        ),
    }
}

fn sorted(values: &[String]) -> Vec<String> {
    let mut copy = values.to_vec();
    copy.sort();
    copy
}

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

/// One row of the S01 table: (message type, kind, producers, consumers).
type Row = (
    &'static str,
    Kind,
    &'static [&'static str],
    &'static [&'static str],
);

/// The rows of S01 that the metadata states, including every message XR-021 adds.
const ROWS: &[Row] = &[
    // Added by XR-021.
    (
        "content.capture.requested.v1",
        Kind::Command,
        &["ratatoskr-platform", "ratatoskr-x"],
        &["ratatoskr-extractor"],
    ),
    (
        "content.document.extracted.v1",
        Kind::Event,
        &["ratatoskr-extractor"],
        &["ratatoskr-knowledge"],
    ),
    (
        "vault.backup_policy.apply_requested.v1",
        Kind::Command,
        &["ratatoskr-github"],
        &["ratatoskr-vault"],
    ),
    (
        "platform.schedule.registration_requested.v1",
        Kind::Command,
        &["ratatoskr-channel-digests", "ratatoskr-github"],
        &["ratatoskr-platform"],
    ),
    // Registered before XR-021 and stated by S01.
    (
        "social.capture.requested.v1",
        Kind::Command,
        &["ratatoskr-platform"],
        &["ratatoskr-instagram", "ratatoskr-threads", "ratatoskr-x"],
    ),
    (
        "social.source.captured.v1",
        Kind::Event,
        &["ratatoskr-x", "ratatoskr-instagram", "ratatoskr-threads"],
        &["ratatoskr-knowledge"],
    ),
    (
        "social.source.updated.v1",
        Kind::Event,
        &["ratatoskr-x", "ratatoskr-instagram", "ratatoskr-threads"],
        &["ratatoskr-knowledge"],
    ),
    (
        "social.source.removed.v1",
        Kind::Event,
        &["ratatoskr-x", "ratatoskr-instagram", "ratatoskr-threads"],
        &["ratatoskr-knowledge"],
    ),
    (
        "ai_archive.archive.imported.v1",
        Kind::Event,
        &["ratatoskr-chatgpt", "ratatoskr-claude"],
        &["ratatoskr-knowledge"],
    ),
    (
        "ai_archive.conversation.added.v1",
        Kind::Event,
        &["ratatoskr-chatgpt", "ratatoskr-claude"],
        &["ratatoskr-knowledge"],
    ),
    (
        "ai_archive.conversation.updated.v1",
        Kind::Event,
        &["ratatoskr-chatgpt", "ratatoskr-claude"],
        &["ratatoskr-knowledge"],
    ),
    (
        "ai_archive.project.added.v1",
        Kind::Event,
        &["ratatoskr-chatgpt", "ratatoskr-claude"],
        &["ratatoskr-knowledge"],
    ),
    (
        "ai_archive.project.updated.v1",
        Kind::Event,
        &["ratatoskr-chatgpt", "ratatoskr-claude"],
        &["ratatoskr-knowledge"],
    ),
    (
        "ai_archive.artifact.added.v1",
        Kind::Event,
        &["ratatoskr-chatgpt", "ratatoskr-claude"],
        &["ratatoskr-knowledge"],
    ),
    (
        "ai_archive.artifact.updated.v1",
        Kind::Event,
        &["ratatoskr-chatgpt", "ratatoskr-claude"],
        &["ratatoskr-knowledge"],
    ),
    (
        "ai_archive.subject.tombstoned.v1",
        Kind::Event,
        &["ratatoskr-chatgpt", "ratatoskr-claude"],
        &["ratatoskr-knowledge"],
    ),
    (
        "knowledge.repository_analysis.requested.v1",
        Kind::Event,
        &["ratatoskr-github"],
        &["ratatoskr-knowledge"],
    ),
    (
        "knowledge.repository_analysis.completed.v1",
        Kind::Event,
        &["ratatoskr-knowledge"],
        &["ratatoskr-github"],
    ),
    (
        "knowledge.repository_analysis.failed.v1",
        Kind::Event,
        &["ratatoskr-knowledge"],
        &["ratatoskr-github"],
    ),
    (
        "vault.backup_policy.acknowledged.v1",
        Kind::Event,
        &["ratatoskr-vault"],
        &["ratatoskr-github"],
    ),
    (
        "channel_digest.subscription.set_requested.v1",
        Kind::Command,
        &["ratatoskr-platform"],
        &["ratatoskr-channel-digests"],
    ),
    (
        "channel_digest.run.requested.v1",
        Kind::Command,
        &["ratatoskr-platform"],
        &["ratatoskr-channel-digests"],
    ),
    (
        "channel_digest.schedule.occurrence_requested.v1",
        Kind::Command,
        &["ratatoskr-platform"],
        &["ratatoskr-channel-digests"],
    ),
    (
        "knowledge.channel_digest_recap.requested.v1",
        Kind::Command,
        &["ratatoskr-channel-digests"],
        &["ratatoskr-knowledge"],
    ),
    (
        "knowledge.channel_digest_recap.completed.v1",
        Kind::Event,
        &["ratatoskr-knowledge"],
        &["ratatoskr-channel-digests"],
    ),
    (
        "knowledge.channel_digest_recap.failed.v1",
        Kind::Event,
        &["ratatoskr-knowledge"],
        &["ratatoskr-channel-digests"],
    ),
];

#[test]
fn every_live_message_has_the_producers_and_consumers_of_the_bus_table() {
    let metadata = metadata();
    for (message_type, kind, producers, consumers) in ROWS {
        let (registered_kind, contract) = registered(&metadata, message_type);
        assert_eq!(
            registered_kind,
            *kind,
            "{message_type}: the subject class `{}` follows the registration kind",
            kind.class()
        );
        assert_eq!(
            sorted(&contract.producers),
            sorted(&names(producers)),
            "{message_type}: producers"
        );
        assert_eq!(
            sorted(&contract.consumers),
            sorted(&names(consumers)),
            "{message_type}: consumers"
        );
    }
}

#[test]
fn operation_reports_list_exactly_the_services_that_emit_them() {
    let metadata = metadata();
    let (kind, contract) = registered(&metadata, "platform.operation.reported.v1");
    assert_eq!(kind, Kind::Event);
    assert_eq!(
        contract.producers,
        names(&[
            "ratatoskr-extractor",
            "ratatoskr-x",
            "ratatoskr-instagram",
            "ratatoskr-threads",
            "ratatoskr-chatgpt",
            "ratatoskr-claude",
            "ratatoskr-channel-digests",
        ]),
        "github, knowledge and vault do not emit operation reports"
    );
    assert_eq!(
        sorted(&contract.consumers),
        sorted(&names(&["ratatoskr-platform", "ratatoskr-x"])),
        "Platform projects the reports and X reads the extractor's"
    );
}

#[test]
fn a_registered_event_is_described_as_a_fact_not_a_command() {
    let metadata = metadata();
    let (kind, contract) = registered(&metadata, "knowledge.repository_analysis.requested.v1");
    assert_eq!(kind, Kind::Event);
    assert!(
        contract.summary.starts_with("Request fact:"),
        "the registration is an event, so its summary must not call it a command: {:?}",
        contract.summary
    );
}

#[test]
fn registry_maps_agree_with_the_metadata_for_the_new_payloads() {
    let commands = registry::command_payload_types();
    let events = registry::event_payload_types();
    for (path, command_type) in [
        (
            "ratatoskr_document_contracts::ContentCaptureRequested",
            "content.capture.requested.v1",
        ),
        (
            "ratatoskr_backup_contracts::VaultBackupPolicyApplyRequested",
            "vault.backup_policy.apply_requested.v1",
        ),
        (
            "ratatoskr_operation_contracts::PlatformScheduleRegistrationRequested",
            "platform.schedule.registration_requested.v1",
        ),
    ] {
        assert_eq!(commands.get(path).copied(), Some(command_type), "{path}");
    }
    assert_eq!(
        events
            .get("ratatoskr_document_contracts::ContentDocumentExtracted")
            .copied(),
        Some("content.document.extracted.v1")
    );
}
