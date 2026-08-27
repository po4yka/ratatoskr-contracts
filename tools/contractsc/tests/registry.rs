//! Registry closure for the operational contract family.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "assertion failures are the reporting boundary of this test binary"
)]

use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{GENERATOR_VERSION, Metadata, generate, registry};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

#[test]
fn operational_contract_is_registered_once() {
    let root = repo_root();
    let paths: Vec<&str> = registry::root_types()
        .iter()
        .map(|root| root.rust_path)
        .filter(|path| path.starts_with("ratatoskr_operational_contracts::"))
        .collect();
    assert_eq!(
        paths,
        [
            "ratatoskr_operational_contracts::AuditEventPage",
            "ratatoskr_operational_contracts::OperationInspectionPage",
            "ratatoskr_operational_contracts::PublicStatusDocument",
            "ratatoskr_operational_contracts::ScheduleInspectionPage",
        ],
        "the operational family must have exactly four sorted generator roots"
    );

    let metadata_text = std::fs::read_to_string(root.join(Metadata::FILE_NAME))
        .expect("contracts metadata must be readable");
    let metadata = Metadata::parse(&metadata_text).expect("contracts metadata must parse");
    let generated =
        generate(&metadata, GENERATOR_VERSION).expect("operational roots must generate");
    let operational: Vec<_> = metadata
        .contracts
        .iter()
        .filter(|contract| contract.family == "operational")
        .collect();
    assert_eq!(operational.len(), 4, "one metadata entry per root");
    assert!(operational.iter().all(|contract| {
        contract.root_types.iter().all(|declared| {
            let typescript = declared
                .output
                .replace("schemas/", "generated/typescript/")
                .replace(".schema.json", ".ts");
            declared
                .output
                .starts_with("schemas/json-schema/operational/")
                && generated.contains_key(Path::new(&declared.output))
                && generated.contains_key(Path::new(&typescript))
        }) && root.join(&contract.fixtures_dir).join("valid").is_dir()
    }));

    let expectations = std::fs::read_to_string(root.join("fixtures/invalid-expectations.toml"))
        .expect("invalid expectation registry must be readable");
    assert!(
        expectations.contains("operational/"),
        "operational invalid fixtures must have declared expectations"
    );
    assert!(
        root.join("compat/api/ratatoskr-operational-contracts.txt")
            .is_file(),
        "the new crate must have a frozen Rust API baseline"
    );
}
