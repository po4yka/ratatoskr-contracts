//! Compatibility fixtures: both directions exist and re-emit losslessly for every fixture
//! family this crate publishes.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_ai_archive_contracts::{
    AiArchiveImport, AiArchiveSnapshot, AiArchiveTombstone, AiConversation, AiConversationAdded,
    AiConversationUpdated, AiMessage, AiProject, AiProjectAdded, AiProjectUpdated,
};
use ratatoskr_identifiers::dropped_field_pointers;

/// One fixture family: its directory under `fixtures/` and how its root type round-trips.
struct Family {
    /// Repository-relative fixture root, e.g. `ai_archive/conversation`.
    dir: &'static str,

    /// Deserializes into the family's root type and re-serializes.
    roundtrip: fn(&str) -> (serde_json::Value, Result<serde_json::Value, String>),
}

macro_rules! root_case {
    ($ty:ty) => {
        |body: &str| match serde_json::from_str::<$ty>(body) {
            Ok(typed) => (
                serde_json::from_str(body).expect("wire is JSON"),
                serde_json::to_value(&typed).map_err(|error| error.to_string()),
            ),
            Err(error) => (
                serde_json::from_str(body).expect("wire is JSON"),
                Err(error.to_string()),
            ),
        }
    };
}

fn families() -> Vec<Family> {
    vec![
        Family {
            dir: "ai_archive/archive-snapshot",
            roundtrip: root_case!(AiArchiveSnapshot),
        },
        Family {
            dir: "ai_archive/project",
            roundtrip: root_case!(AiProject),
        },
        Family {
            dir: "ai_archive/conversation",
            roundtrip: root_case!(AiConversation),
        },
        Family {
            dir: "ai_archive/message",
            roundtrip: root_case!(AiMessage),
        },
        Family {
            dir: "events/ai_archive.archive.imported.v1",
            roundtrip: root_case!(AiArchiveImport),
        },
        Family {
            dir: "events/ai_archive.conversation.added.v1",
            roundtrip: root_case!(AiConversationAdded),
        },
        Family {
            dir: "events/ai_archive.conversation.updated.v1",
            roundtrip: root_case!(AiConversationUpdated),
        },
        Family {
            dir: "events/ai_archive.project.added.v1",
            roundtrip: root_case!(AiProjectAdded),
        },
        Family {
            dir: "events/ai_archive.project.updated.v1",
            roundtrip: root_case!(AiProjectUpdated),
        },
        Family {
            dir: "events/ai_archive.subject.tombstoned.v1",
            roundtrip: root_case!(AiArchiveTombstone),
        },
    ]
}

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the crate sits two levels below the repository root")
        .to_path_buf()
}

const DIRECTIONS: &[&str] = &[
    "compat/new-consumer-old-producer",
    "compat/old-consumer-new-producer",
];

/// Every family carries both compatibility directions, and every compat fixture parses and
/// re-emits without dropping a member.
#[test]
fn compat_fixtures_exist_for_every_family_and_reemit_losslessly() {
    for family in families() {
        for direction in DIRECTIONS {
            let bucket = repo_root()
                .join("fixtures")
                .join(family.dir)
                .join(direction);
            let found: Vec<std::path::PathBuf> = std::fs::read_dir(&bucket)
                .unwrap_or_else(|error| panic!("{} missing: {error}", bucket.display()))
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
                .collect();
            assert!(
                !found.is_empty(),
                "{}/{direction} has no fixture",
                family.dir
            );
            for path in found {
                let body = std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("{} unreadable: {error}", path.display()));
                let (sent, outcome) = (family.roundtrip)(&body);
                let reemitted =
                    outcome.unwrap_or_else(|error| panic!("{} rejected: {error}", path.display()));
                assert!(
                    dropped_field_pointers(&sent, &reemitted).is_empty(),
                    "{} loses members on re-emission",
                    path.display()
                );
            }
        }
    }
}
