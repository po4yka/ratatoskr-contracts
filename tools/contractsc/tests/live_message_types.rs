//! The live-message registry: every message on the fleet bus has the producers and consumers
//! the single bus table of XR-021 CONTRACTS.md S01 gives it, and every audience the registry
//! declares that nothing wires is listed in [`UNWIRED`] with the reason (XR-021 round 2, R2-19).

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

/// Which list of a registry row an [`UNWIRED`] entry belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Producer,
    Consumer,
}

impl Side {
    fn of(self, contract: &Contract) -> &[String] {
        match self {
            Self::Producer => &contract.producers,
            Self::Consumer => &contract.consumers,
        }
    }
}

/// Registry audiences that are a declaration only: (contract id, side, service, reason).
///
/// Rule R7 forbids an empty producers or consumers list, so a row keeps the audience it was
/// designed for even when no service publishes or reads the message today. [`ROWS`] says which
/// audience is wired; this table says which declared audience is not, so the two together account
/// for every entry of every pinned row.
const UNWIRED: &[(&str, Side, &str, &str)] = &[
    (
        "knowledge.social_source_analysis_completed",
        Side::Consumer,
        "ratatoskr-instagram",
        "S01 and R2-19: published with no consumer; a feedback link no flow depends on needs an Edge durable and an ACL stanza",
    ),
    (
        "knowledge.ai_archive_analysis_completed",
        Side::Consumer,
        "ratatoskr-chatgpt",
        "S01 and R2-19: published with no consumer; a feedback link no flow depends on needs an Edge durable and an ACL stanza",
    ),
    (
        "knowledge.ai_archive_analysis_completed",
        Side::Consumer,
        "ratatoskr-claude",
        "S01 and R2-19: published with no consumer; a feedback link no flow depends on needs an Edge durable and an ACL stanza",
    ),
    (
        "platform.notification_raised",
        Side::Producer,
        "ratatoskr-knowledge",
        "R2-19: nothing in the fleet publishes evt.platform.notification.raised.v1; what raises a notification is a feature decision outside XR-021",
    ),
    (
        "platform.notification_raised",
        Side::Producer,
        "ratatoskr-github",
        "R2-19: nothing in the fleet publishes evt.platform.notification.raised.v1; what raises a notification is a feature decision outside XR-021",
    ),
    (
        "platform.notification_raised",
        Side::Producer,
        "ratatoskr-vault",
        "R2-19: nothing in the fleet publishes evt.platform.notification.raised.v1; what raises a notification is a feature decision outside XR-021",
    ),
    (
        "platform.notification_raised",
        Side::Producer,
        "ratatoskr-x",
        "R2-19: nothing in the fleet publishes evt.platform.notification.raised.v1; what raises a notification is a feature decision outside XR-021",
    ),
    (
        "platform.operation_progressed",
        Side::Producer,
        "ratatoskr-platform",
        "Platform does not publish evt.platform.operation.progressed.v1 on the bus today",
    ),
    (
        "platform.operation_progressed",
        Side::Consumer,
        "ratatoskr-knowledge",
        "Platform does not publish the event, so Knowledge has nothing to read",
    ),
];

/// One row of the S01 table: (message type, kind, wired producers, wired consumers).
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
    // Published or declared without a wired audience (S01 and R2-19); `UNWIRED` holds the rest.
    (
        "knowledge.analysis.completed.v1",
        Kind::Event,
        &["ratatoskr-knowledge"],
        &[],
    ),
    (
        "knowledge.ai_archive_analysis.completed.v1",
        Kind::Event,
        &["ratatoskr-knowledge"],
        &[],
    ),
    (
        "platform.notification.raised.v1",
        Kind::Event,
        &[],
        &["ratatoskr-telegram"],
    ),
    ("platform.operation.progressed.v1", Kind::Event, &[], &[]),
];

/// The declared-but-unwired services of `contract_id` on `side`.
fn unwired(contract_id: &str, side: Side) -> Vec<&'static str> {
    UNWIRED
        .iter()
        .filter(|(id, entry_side, _, _)| *id == contract_id && *entry_side == side)
        .map(|(_, _, service, _)| *service)
        .collect()
}

fn with_unwired(wired: &[&str], contract_id: &str, side: Side) -> Vec<String> {
    let mut all = names(wired);
    all.extend(names(&unwired(contract_id, side)));
    all.sort();
    all
}

/// Every pinned row's registry audience equals its wired audience plus its `UNWIRED` entries.
fn assert_audiences_are_wired_or_listed(metadata: &Metadata) {
    for (message_type, kind, producers, consumers) in ROWS {
        let (registered_kind, contract) = registered(metadata, message_type);
        assert_eq!(
            registered_kind,
            *kind,
            "{message_type}: the subject class `{}` follows the registration kind",
            kind.class()
        );
        assert_eq!(
            sorted(&contract.producers),
            with_unwired(producers, &contract.id, Side::Producer),
            "{message_type}: producers are the wired ones plus the UNWIRED entries"
        );
        assert_eq!(
            sorted(&contract.consumers),
            with_unwired(consumers, &contract.id, Side::Consumer),
            "{message_type}: consumers are the wired ones plus the UNWIRED entries"
        );
    }
}

#[test]
fn every_live_message_has_the_producers_and_consumers_of_the_bus_table() {
    assert_audiences_are_wired_or_listed(&metadata());
}

#[test]
fn declared_audiences_that_nothing_wires_are_listed_and_nothing_else() {
    let metadata = metadata();
    let pinned: Vec<&str> = ROWS
        .iter()
        .map(|(message_type, ..)| registered(&metadata, message_type).1.id.as_str())
        .collect();
    let mut seen: Vec<(&str, &str, &str)> = Vec::new();
    for (contract_id, side, service, reason) in UNWIRED {
        let contract = metadata
            .contracts
            .iter()
            .find(|contract| contract.id == *contract_id)
            .unwrap_or_else(|| panic!("{contract_id}: no such contract in contracts.toml"));
        assert!(
            side.of(contract).iter().any(|name| name == service),
            "{contract_id}: {service} is not on the {side:?} side of the registry row"
        );
        assert!(
            pinned.contains(contract_id),
            "{contract_id}: no ROWS entry pins this contract, so its UNWIRED entry would go unchecked"
        );
        assert!(
            !reason.trim().is_empty(),
            "{contract_id}: {service} needs a reason"
        );
        let key = (
            *contract_id,
            if matches!(side, Side::Producer) {
                "producer"
            } else {
                "consumer"
            },
            *service,
        );
        assert!(
            !seen.contains(&key),
            "{contract_id}: {service} is listed twice"
        );
        seen.push(key);
    }
    // (b) a declared audience that is neither wired in ROWS nor listed in UNWIRED fails here.
    assert_audiences_are_wired_or_listed(&metadata);
}

#[test]
fn transfer_rows_are_http_counterparts_and_not_bus_messages() {
    let metadata = metadata();
    let transfer: Vec<&Contract> = metadata
        .contracts
        .iter()
        .filter(|contract| contract.id.starts_with("transfer."))
        .collect();
    assert!(
        !transfer.is_empty(),
        "the registry carries the transfer rows"
    );
    for contract in transfer {
        assert!(
            contract.command.is_none() && contract.event.is_none(),
            "{}: a transfer row is an HTTP body; its audience is an HTTP counterpart, not a bus subject",
            contract.id
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
