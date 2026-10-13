//! Registry facts of the notes events (OBS-023).
#![allow(clippy::expect_used, clippy::panic, reason = "test diagnostics")]

use std::path::{Path, PathBuf};

use ratatoskr_contractsc::Metadata;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("contractsc sits under tools")
        .to_path_buf()
}

/// File paths and the vault display name are written by the owner, so every notes event is
/// `user_content`, like the notification title and the archived conversation events.
#[test]
fn notes_events_are_user_content() {
    let text =
        std::fs::read_to_string(repo_root().join(Metadata::FILE_NAME)).expect("contracts metadata");
    let metadata = Metadata::parse(&text).expect("metadata parses");
    let notes: Vec<_> = metadata
        .contracts
        .iter()
        .filter(|contract| contract.crate_name == "ratatoskr-notes-contracts")
        .collect();
    assert_eq!(notes.len(), 3, "one governed entry per notes event");
    for contract in notes {
        for root in &contract.root_types {
            assert_eq!(
                format!("{:?}", root.privacy),
                "UserContent",
                "{} carries owner-written paths or names",
                contract.id
            );
        }
    }
}
