//! Deterministic generation and the clean-tree guarantee — tests G-1 to G-9.
//!
//! `docs/TESTING.md`: "Deterministic generation and clean-tree checks" (T3) and "Release
//! artifacts are rebuilt and compared with committed/generated expectations" (T10).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{Finding, GENERATOR_VERSION, Metadata, check, generate};

/// The repository root, derived from the compiled-in manifest directory so the tests do not
/// depend on the working directory either.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tools/contractsc sits two levels below the repository root")
        .to_path_buf()
}

/// The committed metadata.
fn metadata() -> Metadata {
    let path = repo_root().join(Metadata::FILE_NAME);
    let text = std::fs::read_to_string(&path).expect("contracts.toml is committed");
    Metadata::parse(&text).expect("contracts.toml parses")
}

/// Every generated artifact, in memory.
fn generated() -> BTreeMap<PathBuf, String> {
    generate(&metadata(), GENERATOR_VERSION).expect("the committed contracts generate")
}

/// G-1. Two independent runs of the pure function agree byte for byte: no `HashMap` iteration, no
/// clock, no address-dependent ordering.
#[test]
fn generate_twice_produces_identical_bytes() {
    let metadata = metadata();
    let first = generate(&metadata, GENERATOR_VERSION).expect("first run");
    let second = generate(&metadata, GENERATOR_VERSION).expect("second run");
    assert_eq!(first, second);
    assert_eq!(
        first.len(),
        4,
        "four registered roots produce four artifacts"
    );
}

/// G-2. The committed tree is exactly what the generator produces, and nothing else is wrong.
///
/// This is the `ARCHITECTURE.md` S11 clean-checkout guarantee and `docs/TESTING.md` T10: the
/// release artifacts are rebuilt here and compared with the committed expectations.
#[test]
fn committed_artifacts_match_generator() {
    let report = check(&repo_root()).expect("check runs against the committed tree");
    assert!(report.is_current(), "{report}");
}

/// G-3. The only tripwire for a dependency enabling `serde_json/preserve_order`.
///
/// Cargo features are additive across a workspace, so any crate turning that feature on — directly
/// or through `schemars/preserve_order` — would flip every generated file from `BTreeMap` order to
/// insertion order and silently rewrite every committed artifact. `#[cfg]` cannot see it from
/// here. **Never delete or `#[ignore]` this test.**
#[test]
fn serde_json_map_is_btreemap() {
    let mut members = serde_json::Map::new();
    members.insert("b".to_owned(), serde_json::json!(1));
    members.insert("a".to_owned(), serde_json::json!(2));
    let rendered = serde_json::to_string(&serde_json::Value::Object(members))
        .expect("a map of integers serializes");
    assert_eq!(rendered, r#"{"a":2,"b":1}"#);
}

/// G-4. Every `required` array is lexicographic, so a Rust field reorder is a schema no-op.
#[test]
fn required_arrays_are_sorted() {
    for (path, body) in generated() {
        let document: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
        let mut checked = 0usize;
        visit(&document, &mut |node| {
            let Some(names) = node.get("required").and_then(serde_json::Value::as_array) else {
                return;
            };
            let names: Vec<&str> = names.iter().filter_map(serde_json::Value::as_str).collect();
            let mut sorted = names.clone();
            sorted.sort_unstable();
            assert_eq!(
                names,
                sorted,
                "{} has an unsorted `required`",
                path.display()
            );
            checked += 1;
        });
        assert!(
            checked > 0,
            "{} declares no `required` at all",
            path.display()
        );
    }
}

/// G-5. No generated schema contains a floating-point number, whose rendering can differ between
/// platforms.
#[test]
fn no_floats_in_generated_schemas() {
    for (path, body) in generated() {
        let document: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
        assert!(
            !contains_float(&document),
            "{} contains a float",
            path.display()
        );
    }
}

/// G-6. The bytes do not depend on the working directory or on the environment.
///
/// Driven through the real binary rather than the library, because `std::env::set_var` is `unsafe`
/// under edition 2024 and this workspace forbids `unsafe_code`. A subprocess gives the stronger
/// guarantee anyway: a whole process with a scrambled environment and a different current
/// directory still reproduces the committed bytes.
#[test]
fn generation_is_independent_of_cwd_and_env() {
    let root = repo_root();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_contractsc"))
        .arg("check")
        .arg("--root")
        .arg(&root)
        .current_dir(std::env::temp_dir())
        .env("HOME", "/nonexistent")
        .env("TZ", "Pacific/Kiritimati")
        .env("LANG", "tr_TR.UTF-8")
        .env("LC_ALL", "tr_TR.UTF-8")
        .output()
        .expect("the generator binary runs");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// G-7. A hand edit to a generated file is reported as `Stale`, naming the first differing line.
#[test]
fn check_detects_a_hand_edit() {
    let staged = StagedTree::new("hand-edit");
    let target = staged
        .path
        .join("schemas/json-schema/core/event-envelope.v1.schema.json");
    let body = std::fs::read_to_string(&target).expect("the artifact is committed");
    let edited = body.replacen("The common envelope", "The uncommon envelope", 1);
    assert_ne!(edited, body, "the description anchor still exists");
    std::fs::write(&target, &edited).expect("the staged copy is writable");

    let report = check(&staged.path).expect("check runs against the staged copy");
    let stale = report
        .findings
        .iter()
        .find_map(|finding| match finding {
            Finding::Stale {
                path,
                line,
                expected,
                actual,
            } => Some((path.clone(), *line, expected.clone(), actual.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected a Stale finding, got {report}"));
    assert_eq!(
        stale.0,
        PathBuf::from("schemas/json-schema/core/event-envelope.v1.schema.json")
    );
    assert!(stale.1 > 0, "the reported line is one-based");
    assert!(
        stale.2.contains("The common envelope"),
        "expected line: {}",
        stale.2
    );
    assert!(
        stale.3.contains("The uncommon envelope"),
        "actual line: {}",
        stale.3
    );
}

/// G-7b. A difference confined to the line terminators is still `Stale`.
///
/// Stripping the final newline, or rewriting the file as CRLF, leaves every *visible* character
/// in place, and `recompute_digest` re-renders the parsed JSON so the embedded digest still
/// matches — neither `Stale` nor `Tampered` would fire if the gate compared anything less than
/// the raw bytes. `git diff` reports both, so `cargo contracts check` must too, or the two
/// disagree about a clean checkout (§6.6 rules 7–8, `ARCHITECTURE.md` S11).
#[test]
fn check_detects_a_line_terminator_only_edit() {
    for (name, rewrite) in [
        (
            "no-final-newline",
            (|body: &str| {
                body.strip_suffix('\n')
                    .expect("a generated artifact ends with exactly one newline")
                    .to_owned()
            }) as fn(&str) -> String,
        ),
        ("crlf", |body: &str| body.replace('\n', "\r\n")),
    ] {
        let staged = StagedTree::new(name);
        let relative = Path::new("schemas/json-schema/core/event-envelope.v1.schema.json");
        let target = staged.path.join(relative);
        let body = std::fs::read_to_string(&target).expect("the artifact is committed");
        let edited = rewrite(&body);
        assert_ne!(edited, body, "{name} must actually change the bytes");
        std::fs::write(&target, &edited).expect("the staged copy is writable");

        let report = check(&staged.path).expect("check runs against the staged copy");
        assert!(
            report.findings.iter().any(|finding| matches!(
                finding,
                Finding::Stale { path, .. } if path == relative
            )),
            "{name}: expected a Stale finding, got {report}"
        );
        assert_eq!(report.exit_code(), 1, "{name}: the gate must fail");
    }
}

/// G-8. A generated file whose body no longer implies its own digest is reported as `Tampered`,
/// distinctly from routine staleness. This is the `AGENTS.md` "never edited manually" violation.
#[test]
fn check_detects_a_tampered_digest() {
    let staged = StagedTree::new("tampered");
    let target = staged
        .path
        .join("schemas/json-schema/core/error-envelope.v1.schema.json");
    let body = std::fs::read_to_string(&target).expect("the artifact is committed");
    let edited = body.replacen(
        "\"type\": \"object\"",
        "\"type\": \"object\", \"minProperties\": 1",
        1,
    );
    assert_ne!(edited, body, "the type anchor still exists");
    std::fs::write(&target, edited).expect("the staged copy is writable");

    let report = check(&staged.path).expect("check runs against the staged copy");
    assert!(
        report.findings.contains(&Finding::Tampered {
            path: PathBuf::from("schemas/json-schema/core/error-envelope.v1.schema.json"),
        }),
        "expected a Tampered finding, got {report}"
    );
}

/// G-9. A `*.schema.json` under `schemas/**` that no root type produces is an `Orphan` — a deleted
/// contract's leftover artifact, which `git diff` alone would never catch.
#[test]
fn check_detects_an_orphan_schema() {
    let staged = StagedTree::new("orphan");
    let orphan = staged
        .path
        .join("schemas/json-schema/core/retired.v1.schema.json");
    std::fs::write(&orphan, "{}\n").expect("the staged copy is writable");

    let report = check(&staged.path).expect("check runs against the staged copy");
    assert!(
        report.findings.contains(&Finding::Orphan {
            path: PathBuf::from("schemas/json-schema/core/retired.v1.schema.json"),
        }),
        "expected an Orphan finding, got {report}"
    );
}

/// A throwaway copy of the repository, removed when the test ends.
struct StagedTree {
    /// Root of the copy.
    path: PathBuf,
}

impl StagedTree {
    /// Copies the repository, minus `.git` and `target`, into a fresh temporary directory.
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "ratatoskr-contractsc-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        copy_tree(&repo_root(), &path);
        Self { path }
    }
}

impl Drop for StagedTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Recursively copies `from` to `to`, skipping the version-control and build directories.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a temporary directory is creatable");
    for entry in std::fs::read_dir(from)
        .expect("the repository is readable")
        .flatten()
    {
        let name = entry.file_name();
        if name == ".git" || name == "target" {
            continue;
        }
        let source = entry.path();
        let destination = to.join(&name);
        if source.is_dir() {
            copy_tree(&source, &destination);
        } else {
            std::fs::copy(&source, &destination).expect("a repository file is copyable");
        }
    }
}

/// Calls `visit` on every JSON object in the document, root first.
fn visit(value: &serde_json::Value, visitor: &mut impl FnMut(&serde_json::Value)) {
    match value {
        serde_json::Value::Object(members) => {
            visitor(value);
            for member in members.values() {
                visit(member, visitor);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                visit(item, visitor);
            }
        }
        _ => {}
    }
}

/// `true` when any number anywhere in the document is not an integer.
fn contains_float(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Number(number) => number.is_f64(),
        serde_json::Value::Object(members) => members.values().any(contains_float),
        serde_json::Value::Array(items) => items.iter().any(contains_float),
        _ => false,
    }
}
