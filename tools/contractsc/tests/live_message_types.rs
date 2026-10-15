//! The live-message registry: every message on the fleet bus has the producers and consumers
//! the single bus table of XR-021 CONTRACTS.md S01 gives it, and every audience the registry
//! declares that nothing wires is listed in [`UNWIRED`] with the reason (XR-021 round 2, R2-19).

#![allow(clippy::expect_used, clippy::panic, reason = "test diagnostics")]

use std::path::{Path, PathBuf};

use ratatoskr_contractsc::metadata::Contract;
use ratatoskr_contractsc::{GENERATOR_VERSION, Metadata, generate, registry};

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
    (
        "knowledge.deletion.acknowledged.v1",
        Kind::Event,
        &["ratatoskr-knowledge"],
        &[
            "ratatoskr-chatgpt",
            "ratatoskr-claude",
            "ratatoskr-x",
            "ratatoskr-instagram",
            "ratatoskr-threads",
        ],
    ),
    (
        "platform.stream_erasure.requested.v1",
        Kind::Command,
        &["ratatoskr-chatgpt", "ratatoskr-claude"],
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
            "ratatoskr-notes",
            "ratatoskr-sites",
        ]),
        "github, knowledge and vault do not emit operation reports; notes reports its connect and \
         write operations (OBS-023, OBS-024) and sites its release builds (SITE-025)"
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

/// OBS-024, workspace spec `notes-write-command`: Edge is the only producer of the write command
/// and notes its only consumer; notes publishes the two conflict events, and Edge alone reads
/// them. An extra producer or consumer fails here, and the message names the registered list.
#[test]
fn notes_write_request_has_one_producer() {
    let metadata = metadata();
    for (message_type, kind, producer, consumer) in [
        (
            "notes.file.write_requested.v1",
            Kind::Command,
            "ratatoskr-platform",
            "ratatoskr-notes",
        ),
        (
            "notes.conflict.raised.v1",
            Kind::Event,
            "ratatoskr-notes",
            "ratatoskr-platform",
        ),
        (
            "notes.conflict.resolved.v1",
            Kind::Event,
            "ratatoskr-notes",
            "ratatoskr-platform",
        ),
    ] {
        let (registered_kind, contract) = registered(&metadata, message_type);
        assert_eq!(
            registered_kind,
            kind,
            "{message_type}: the subject class `{}` follows the registration kind",
            kind.class()
        );
        assert_eq!(
            contract.producers,
            names(&[producer]),
            "{message_type}: the only producer is {producer}, but the row names {:?}",
            contract.producers
        );
        assert_eq!(
            contract.consumers,
            names(&[consumer]),
            "{message_type}: the only consumer is {consumer}, but the row names {:?}",
            contract.consumers
        );
    }
}

/// Every property name declared anywhere in a schema document, `$defs` included.
fn property_names(schema: &serde_json::Value, names: &mut Vec<String>) {
    match schema {
        serde_json::Value::Object(members) => {
            if let Some(serde_json::Value::Object(properties)) = members.get("properties") {
                names.extend(properties.keys().cloned());
            }
            for member in members.values() {
                property_names(member, names);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                property_names(item, names);
            }
        }
        _ => {}
    }
}

/// SITE-025, workspace specs `site-publications` and `site-manifest` and the bus table of
/// `docs/NOTES_AND_SITES.md`: Edge alone asks for a release build and Platform alone announces an
/// approval, both for the sites service; the sites service alone announces a release, and that
/// event names no page; the sites service reads the two notes events it builds from.
#[test]
fn sites_message_types_are_registered() {
    let metadata = metadata();
    for (message_type, kind, producer, consumer) in [
        (
            "sites.release.build_requested.v1",
            Kind::Command,
            "ratatoskr-platform",
            Some("ratatoskr-sites"),
        ),
        (
            "sites.release.published.v1",
            Kind::Event,
            "ratatoskr-sites",
            None,
        ),
        (
            "platform.site_approval.changed.v1",
            Kind::Event,
            "ratatoskr-platform",
            Some("ratatoskr-sites"),
        ),
    ] {
        let (registered_kind, contract) = registered(&metadata, message_type);
        assert_eq!(
            registered_kind,
            kind,
            "{message_type}: the subject class `{}` follows the registration kind",
            kind.class()
        );
        assert_eq!(
            contract.producers,
            names(&[producer]),
            "{message_type}: the only producer is {producer}, but the row names {:?}",
            contract.producers
        );
        if let Some(consumer) = consumer {
            assert_eq!(
                contract.consumers,
                names(&[consumer]),
                "{message_type}: the only consumer is {consumer}, but the row names {:?}",
                contract.consumers
            );
        }
    }

    let (_, published) = registered(&metadata, "sites.release.published.v1");
    let output = &published
        .root_types
        .first()
        .expect("the release event declares its payload")
        .output;
    let generated =
        generate(&metadata, GENERATOR_VERSION).expect("the committed contracts generate");
    let schema: serde_json::Value = serde_json::from_str(
        generated
            .get(std::path::Path::new(output))
            .unwrap_or_else(|| panic!("{output} is not generated")),
    )
    .expect("a generated schema is JSON");
    let mut properties = Vec::new();
    property_names(&schema, &mut properties);
    let page_data: Vec<&String> = properties
        .iter()
        .filter(|name| {
            ["title", "slug", "path"]
                .iter()
                .any(|banned| name.ends_with(banned))
        })
        .collect();
    assert!(
        page_data.is_empty(),
        "sites.release.published.v1 must carry no page data, but its schema declares {page_data:?}"
    );

    for message_type in ["notes.file.changed.v1", "notes.vault.disconnected.v1"] {
        let (_, contract) = registered(&metadata, message_type);
        assert!(
            contract
                .consumers
                .iter()
                .any(|name| name == "ratatoskr-sites"),
            "{message_type}: ratatoskr-sites is not a consumer, the row names {:?}",
            contract.consumers
        );
    }
}

/// SITE-027, workspace design D5: the sites service alone announces a draft change, Edge alone
/// reads it, and the message carries no note content and no vault path.
#[test]
fn sites_draft_changed_is_registered() {
    let metadata = metadata();
    let (kind, contract) = registered(&metadata, "sites.draft.changed.v1");
    assert_eq!(kind, Kind::Event, "sites.draft.changed.v1 is an event");
    assert_eq!(
        contract.producers,
        names(&["ratatoskr-sites"]),
        "sites.draft.changed.v1: the only producer is ratatoskr-sites"
    );
    assert_eq!(
        contract.consumers,
        names(&["ratatoskr-platform"]),
        "sites.draft.changed.v1: the only consumer is ratatoskr-platform"
    );
    let output = &contract
        .root_types
        .first()
        .expect("the draft change declares its payload")
        .output;
    let generated =
        generate(&metadata, GENERATOR_VERSION).expect("the committed contracts generate");
    let schema: serde_json::Value = serde_json::from_str(
        generated
            .get(std::path::Path::new(output))
            .unwrap_or_else(|| panic!("{output} is not generated")),
    )
    .expect("a generated schema is JSON");
    let mut properties = Vec::new();
    property_names(&schema, &mut properties);
    let private: Vec<&String> = properties
        .iter()
        .filter(|name| name.ends_with("content") || name.ends_with("path"))
        .collect();
    assert!(
        private.is_empty(),
        "sites.draft.changed.v1 must carry no content and no path, but its schema declares {private:?}"
    );
}
