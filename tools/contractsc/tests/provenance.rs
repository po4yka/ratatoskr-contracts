//! The provenance header and the provenance-free digest — tests P-1 to P-5.
//!
//! `ARCHITECTURE.md` S11: "Generated outputs include a provenance header containing generator
//! version, source digest, and contract version."

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{
    GENERATOR_VERSION, Metadata, generate, normalize, provenance, registry, render, typescript,
};

/// Every member the provenance block must carry, and nothing else.
const REQUIRED_MEMBERS: &[&str] = &[
    "canonical_source",
    "contract_id",
    "contract_major_version",
    "generated",
    "generator",
    "generator_version",
    "schemars_version",
    "source_digest",
    "validation_note",
];

/// The repository root, derived from the compiled-in manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

/// Every generated artifact, in memory. The TypeScript family is excluded: its provenance
/// rides in a leading block comment rather than a JSON extension keyword, and the
/// `typescript_header_*` tests cover it with comment-aware rules.
fn generated() -> BTreeMap<PathBuf, String> {
    let path = repo_root().join(Metadata::FILE_NAME);
    let text = std::fs::read_to_string(&path).expect("contracts.toml is committed");
    let metadata = Metadata::parse(&text).expect("contracts.toml parses");
    generate(&metadata, GENERATOR_VERSION)
        .expect("the committed contracts generate")
        .into_iter()
        .filter(|(path, _)| path.to_string_lossy().ends_with(".schema.json"))
        .collect()
}

/// P-1. All nine members are present, the marker string is exact, and the digest is a lowercase
/// `sha256:` hex string.
#[test]
fn provenance_header_is_complete() {
    for (path, body) in generated() {
        let document: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
        let block = document
            .get(provenance::PROVENANCE_KEY)
            .and_then(serde_json::Value::as_object)
            .unwrap_or_else(|| panic!("{} carries no provenance block", path.display()));

        let members: Vec<&str> = block.keys().map(String::as_str).collect();
        assert_eq!(members, REQUIRED_MEMBERS, "{}", path.display());

        assert_eq!(
            block.get("generated").and_then(serde_json::Value::as_str),
            Some(provenance::GENERATED_MARKER),
            "{} carries the wrong generated-file marker",
            path.display()
        );
        assert_eq!(
            block.get("generator").and_then(serde_json::Value::as_str),
            Some(provenance::GENERATOR_NAME)
        );
        assert_eq!(
            block
                .get("generator_version")
                .and_then(serde_json::Value::as_str),
            Some(GENERATOR_VERSION)
        );
        assert_eq!(
            block
                .get("validation_note")
                .and_then(serde_json::Value::as_str),
            Some(provenance::VALIDATION_NOTE)
        );
        assert!(
            block
                .get("contract_major_version")
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|major| major >= 1),
            "{} declares no contract major version",
            path.display()
        );

        let digest = block
            .get("source_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{} carries no source_digest", path.display()));
        let hex = digest
            .strip_prefix("sha256:")
            .unwrap_or_else(|| panic!("{digest} is not a sha256 digest"));
        assert_eq!(hex.len(), 64, "{digest}");
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "{digest} is not lowercase hex"
        );
    }
}

/// P-2. Delete the provenance member, re-render with the canonical serializer, hash: the result is
/// the embedded digest. The digest definition is well-formed and third-party auditable.
#[test]
fn source_digest_matches_recomputed_body() {
    for (path, body) in generated() {
        let embedded = provenance::embedded_digest(&body)
            .unwrap_or_else(|| panic!("{} carries no source_digest", path.display()));
        let recomputed = provenance::recompute_digest(&body)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(embedded, recomputed, "{}", path.display());
    }
}

/// P-3. Nothing environment-dependent leaked into an artifact: no absolute path, no home
/// directory, no user name, and no `generated_at`-style clock reading.
#[test]
fn no_generated_file_contains_a_timestamp_or_absolute_path() {
    let root = repo_root();
    let root_text = root.display().to_string();
    let mut needles = vec![root_text];
    for variable in ["HOME", "USER", "LOGNAME", "CARGO_HOME", "PWD"] {
        if let Ok(value) = std::env::var(variable)
            && value.len() > 3
        {
            needles.push(value);
        }
    }

    for (path, body) in generated() {
        for needle in &needles {
            assert!(
                !body.contains(needle.as_str()),
                "{} leaks the environment value {needle:?}",
                path.display()
            );
        }
        let document: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
        let block = document
            .get(provenance::PROVENANCE_KEY)
            .and_then(serde_json::Value::as_object)
            .expect("a provenance block");
        for clock_reading in ["generated_at", "built_at", "timestamp", "date"] {
            assert!(
                !block.contains_key(clock_reading),
                "{} records {clock_reading}, which no generator can reproduce",
                path.display()
            );
        }
    }
}

/// P-4. The recorded generator identity is true: `SCHEMARS_VERSION` is the version the lockfile
/// pins, because schemars shapes the bytes.
#[test]
fn schemars_version_const_matches_lockfile() {
    let text =
        std::fs::read_to_string(repo_root().join("Cargo.lock")).expect("Cargo.lock is committed");
    let lockfile: toml::Value = toml::from_str(&text).expect("Cargo.lock is TOML");
    let packages = lockfile
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("Cargo.lock lists packages");
    let locked = packages
        .iter()
        .find(|package| package.get("name").and_then(toml::Value::as_str) == Some("schemars"))
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .expect("schemars is locked");
    assert_eq!(locked, provenance::SCHEMARS_VERSION);
}

/// P-5. The serialize and deserialize contracts currently produce identical bytes, so publishing
/// one schema is lossless. If this ever fails, both must be published.
#[test]
fn serialize_and_deserialize_contracts_are_identical() {
    for root in registry::root_types() {
        let mut deserialize = schemars::generate::SchemaSettings::draft2020_12()
            .for_deserialize()
            .into_generator();
        let mut serialize = schemars::generate::SchemaSettings::draft2020_12()
            .for_serialize()
            .into_generator();
        let mut reading = (root.schema)(&mut deserialize);
        let mut writing = (root.schema)(&mut serialize);
        normalize::sort_required(&mut reading);
        normalize::sort_required(&mut writing);
        assert_eq!(
            render::render(&reading),
            render::render(&writing),
            "{} reads and writes different shapes",
            root.rust_path
        );
    }
}

/// Every `.ts` artifact, in memory.
fn typescript_generated() -> BTreeMap<PathBuf, String> {
    let path = repo_root().join(Metadata::FILE_NAME);
    let text = std::fs::read_to_string(&path).expect("contracts.toml is committed");
    let metadata = Metadata::parse(&text).expect("contracts.toml parses");
    generate(&metadata, GENERATOR_VERSION)
        .expect("the committed contracts generate")
        .into_iter()
        .filter(|(path, _)| path.to_string_lossy().ends_with(".ts"))
        .collect()
}

/// The member keys spelled in a header's content lines, in order. A content line is a header
/// line stripped of its comment decoration; `key: value` lines contribute their key, blank
/// separators contribute nothing.
fn header_members(header: &str) -> Vec<&str> {
    header
        .lines()
        .map(str::trim)
        .filter(|line| *line != "/*" && *line != "*/" && !line.is_empty() && *line != "*")
        .map(|line| {
            assert!(
                line.starts_with("* "),
                "every header line is a comment continuation: {line:?}"
            );
            let content = line.get(2..).expect("the `* ` prefix is two bytes");
            let (key, separator) = content
                .split_once(": ")
                .unwrap_or_else(|| panic!("header line {content:?} is not `key: value`"));
            assert!(
                !separator.contains('\n'),
                "a provenance member value spans one line only"
            );
            key
        })
        .collect()
}

/// P-TS-1. Each `.ts` artifact opens with the provenance block comment: the generated-file
/// marker as the first member line, then the remaining eight members, and no timestamp member
/// anywhere — the same nine-member rigor the JSON family carries (design D5).
#[test]
fn typescript_header_carries_required_members() {
    for (path, file) in typescript_generated() {
        let (header, _) = typescript::header_and_body(&file)
            .unwrap_or_else(|| panic!("{} has no leading block comment", path.display()));
        let members = header_members(header);

        assert_eq!(
            members.first(),
            Some(&"generated"),
            "{}: the marker member must come first",
            path.display()
        );
        let marker_line = header
            .lines()
            .map(str::trim)
            .find(|line| line.starts_with("* generated: "))
            .unwrap_or_else(|| panic!("{}: no generated marker", path.display()));
        assert_eq!(
            marker_line,
            format!("* generated: {}", provenance::GENERATED_MARKER),
            "{}: wrong generated-file marker",
            path.display()
        );

        let keys: Vec<&str> = members.clone();
        let mut expected = REQUIRED_MEMBERS.to_vec();
        expected.sort_unstable();
        let mut seen = keys.clone();
        seen.sort_unstable();
        assert_eq!(seen, expected, "{}", path.display());
        assert_eq!(
            keys.len(),
            REQUIRED_MEMBERS.len(),
            "{} repeats a provenance member",
            path.display()
        );

        for clock_reading in ["generated_at", "built_at", "timestamp", "date"] {
            assert!(
                !keys.contains(&clock_reading),
                "{} records {clock_reading}, which no generator can reproduce",
                path.display()
            );
        }
    }
}

/// P-TS-2. The recorded digest is SHA-256 over the header-less body, recomputable by stripping
/// everything through the closing comment delimiter; flipping any body byte breaks the match,
/// which is what makes hand edits distinguishable from staleness.
#[test]
fn typescript_body_digest_detects_tampering() {
    for (path, file) in typescript_generated() {
        let (_, body) = typescript::header_and_body(&file)
            .unwrap_or_else(|| panic!("{} has no leading block comment", path.display()));
        let embedded = typescript::embedded_digest(&file)
            .unwrap_or_else(|| panic!("{} carries no source_digest", path.display()));
        let recomputed =
            typescript::recompute_digest(&file).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(embedded, recomputed, "{}", path.display());

        let hex = embedded
            .strip_prefix("sha256:")
            .unwrap_or_else(|| panic!("{embedded} is not a sha256 digest"));
        assert_eq!(hex.len(), 64, "{embedded}");
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "{embedded} is not lowercase hex"
        );

        let mut tampered = body.to_owned();
        if let Some(position) = tampered.find("export") {
            tampered.replace_range(position..=position, "E");
        }
        assert_ne!(
            tampered,
            body,
            "{}: export anchor still exists",
            path.display()
        );
        let tampered_file = format!("{}\n{}", header_of(&file), tampered);
        let tampered_recomputed =
            typescript::recompute_digest(&tampered_file).unwrap_or_else(|error| panic!("{error}"));
        assert_ne!(
            tampered_recomputed,
            embedded,
            "{}: digest did not move after an edit",
            path.display()
        );
    }
}

/// The header half of an already-split artifact, for reconstruction in tampering scenarios.
fn header_of(file: &str) -> &str {
    typescript::header_and_body(file)
        .map(|(header, _)| header)
        .expect("the artifact carries a header")
}
