//! Deterministic generation and the clean-tree guarantee — tests G-1 to G-9, plus the TypeScript
//! projection suite TS-1 to TS-4.
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

use ratatoskr_contractsc::{
    Finding, GENERATOR_VERSION, Metadata, SCHEMA_SUFFIX, check, emit_typescript, generate,
};

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
        108,
        "fifty-four registered roots produce fifty-four JSON Schema artifacts plus \
         fifty-four TypeScript counterparts; update this pin when the registry changes"
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
/// JSON-family only: the TypeScript family has no `required` keyword to sort.
#[test]
fn required_arrays_are_sorted() {
    for (path, body) in generated() {
        if !path.to_string_lossy().ends_with(SCHEMA_SUFFIX) {
            continue;
        }
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
/// platforms. JSON-family only: TypeScript spells numbers as `number` either way.
#[test]
fn no_floats_in_generated_schemas() {
    for (path, body) in generated() {
        if !path.to_string_lossy().ends_with(SCHEMA_SUFFIX) {
            continue;
        }
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

/// C-TS-1. A deleted `.ts` artifact is reported `Missing` and fails the gate, exactly as a
/// deleted JSON Schema would be.
#[test]
fn check_reports_missing_typescript_declaration() {
    let staged = StagedTree::new("ts-missing");
    let relative = Path::new("generated/typescript/json-schema/core/event-envelope.v1.ts");
    std::fs::remove_file(staged.path.join(relative)).expect("the staged copy is writable");

    let report = check(&staged.path).expect("check runs against the staged copy");
    assert!(
        report.findings.iter().any(|finding| matches!(
            finding,
            Finding::Missing { path } if path == relative
        )),
        "expected a Missing finding, got {report}"
    );
    assert_eq!(report.exit_code(), 1, "the gate must fail");
}

/// C-TS-2. A `.ts` whose bytes differ from what generation produces is reported `Stale`. The
/// edit here also breaks the digest, which co-fires `Tampered`; this test pins the Stale half.
#[test]
fn check_reports_stale_typescript_regeneration() {
    let staged = StagedTree::new("ts-stale");
    let relative = Path::new("generated/typescript/events/social.source.updated.v1.ts");
    let target = staged.path.join(relative);
    let body = std::fs::read_to_string(&target).expect("the artifact is committed");
    let edited = body.replacen("export interface", "export interfaCe", 1);
    assert_ne!(edited, body, "the export anchor still exists");
    std::fs::write(&target, &edited).expect("the staged copy is writable");

    let report = check(&staged.path).expect("check runs against the staged copy");
    assert!(
        report.findings.iter().any(|finding| matches!(
            finding,
            Finding::Stale { path, .. } if path == relative
        )),
        "expected a Stale finding, got {report}"
    );
    assert_eq!(report.exit_code(), 1, "the gate must fail");
}

/// C-TS-3. An artifact whose recorded digest no longer matches its own body is reported
/// `Tampered` — the `AGENTS.md` hand-edit violation — even when the edit is a single hex digit
/// of the header. Mirrors G-8 for the TypeScript provenance spelling.
#[test]
fn check_reports_tampered_typescript_declaration() {
    let staged = StagedTree::new("ts-tampered");
    let relative = Path::new("generated/typescript/json-schema/core/blob-ref.v1.ts");
    let target = staged.path.join(relative);
    let body = std::fs::read_to_string(&target).expect("the artifact is committed");
    let needle = " * source_digest: sha256:";
    let hex_start = body.find(needle).expect("the digest member exists") + needle.len();
    let first_hex = body
        .get(hex_start..hex_start + 1)
        .expect("the digest starts with a hex digit");
    let flipped = if first_hex == "0" { "1" } else { "0" };
    let edited = format!(
        "{}{flipped}{}",
        body.get(..hex_start + 1).expect("boundary checked above"),
        body.get(hex_start + 2..).unwrap_or_default()
    );
    assert_ne!(edited, body, "the recorded digest was actually changed");
    std::fs::write(&target, &edited).expect("the staged copy is writable");

    let report = check(&staged.path).expect("check runs against the staged copy");
    assert!(
        report.findings.contains(&Finding::Tampered {
            path: relative.to_path_buf(),
        }),
        "expected a Tampered finding, got {report}"
    );
    assert_eq!(report.exit_code(), 1, "the gate must fail");
}

/// C-TS-4. A stray `.ts` under `generated/typescript/` that no registered root produces is an
/// `Orphan`, mirroring the schemas sweep.
#[test]
fn check_reports_orphaned_typescript_file() {
    let staged = StagedTree::new("ts-orphan");
    let relative = Path::new("generated/typescript/json-schema/core/retired.v1.ts");
    let orphan = staged.path.join(relative);
    std::fs::create_dir_all(orphan.parent().expect("the parent exists"))
        .expect("the staged copy is writable");
    std::fs::write(&orphan, "/*\n */\nexport type Retired = never;\n")
        .expect("the staged copy is writable");

    let report = check(&staged.path).expect("check runs against the staged copy");
    assert!(
        report.findings.contains(&Finding::Orphan {
            path: relative.to_path_buf(),
        }),
        "expected an Orphan finding, got {report}"
    );
    assert_eq!(report.exit_code(), 1, "the gate must fail");
}

/// TS-1. After `generate()` every root type has exactly one `.ts` under `generated/typescript/`
/// mirroring its schema path one-to-one (D1), exporting the final schema-id segment as the root
/// type followed by every `$defs` member in sorted order (D4), with no imports and no `any`.
#[test]
fn generated_typescript_artifacts_mirror_the_schema_tree() {
    let generated = generated();
    let typescript = typescript_subset(&generated);
    assert_eq!(
        typescript.len(),
        54,
        "fifty-four roots must yield fifty-four TypeScript files"
    );

    for (path, body) in &generated {
        let Some(stem) = path
            .to_string_lossy()
            .strip_prefix("schemas/")
            .and_then(|tail| tail.strip_suffix(SCHEMA_SUFFIX))
            .map(str::to_owned)
        else {
            continue;
        };
        let typescript_path = PathBuf::from(format!("generated/typescript/{stem}.ts"));
        let source = generated
            .get(&typescript_path)
            .unwrap_or_else(|| panic!("{}: no TypeScript counterpart", typescript_path.display()));
        let schema: serde_json::Value =
            serde_json::from_str(body).expect("the schema artifact is valid JSON");
        let schema_id = schema
            .get("$id")
            .and_then(serde_json::Value::as_str)
            .expect("every artifact names its $id");

        let mut previous_position =
            exported_declaration_position(source, root_type_name(schema_id)).unwrap_or_else(|| {
                panic!(
                    "{}: root type {} is not exported",
                    typescript_path.display(),
                    root_type_name(schema_id)
                )
            });
        let definitions = schema
            .get("$defs")
            .and_then(serde_json::Value::as_object)
            .expect("every artifact embeds $defs");
        for name in definitions.keys() {
            let position = exported_declaration_position(source, name).unwrap_or_else(|| {
                panic!(
                    "{}: $defs member {name} is not exported",
                    typescript_path.display()
                )
            });
            assert!(
                position > previous_position,
                "{}: $defs member {name} is exported before the preceding declaration; \
                 definitions must follow the root in sorted order",
                typescript_path.display()
            );
            previous_position = position;
        }
        let code_only = strip_block_comments(source);
        assert!(
            !has_import_statement(&code_only),
            "{} imports from another artifact; declarations are self-contained",
            typescript_path.display()
        );
        assert!(
            !contains_bare_word(&code_only, "any"),
            "{} uses the banned any type",
            typescript_path.display()
        );
    }
}

/// TS-2. Two independent runs agree byte for byte across the TypeScript family, exactly as G-1
/// requires of the JSON Schema family.
#[test]
fn generated_typescript_is_byte_deterministic() {
    let metadata = metadata();
    let first = generate(&metadata, GENERATOR_VERSION).expect("first run");
    let second = generate(&metadata, GENERATOR_VERSION).expect("second run");
    let first_typescript = typescript_subset(&first);
    assert_eq!(
        first_typescript.len(),
        54,
        "fifty-four roots must yield fifty-four TypeScript files"
    );
    assert_eq!(first_typescript, typescript_subset(&second));
}

/// TS-3. No timestamp-shaped content in generated *code*: a clock reading is the classic way a
/// generator stops reproducing its own bytes. Block comments are stripped first because `JSDoc`
/// bodies are verbatim copies of the canonical Rust doc comments, whose prose may legitimately
/// quote dated examples; anything the emitter itself produces lives outside them.
#[test]
fn generated_typescript_contains_no_timestamps() {
    let generated = generated();
    let typescript = typescript_subset(&generated);
    assert_eq!(
        typescript.len(),
        54,
        "fifty-four roots must yield fifty-four TypeScript files"
    );
    for (path, body) in &typescript {
        if let Some(position) = iso_date_positions(&strip_block_comments(body)).first() {
            panic!("{path:?}: timestamp-shaped content at byte offset {position}");
        }
    }
}

/// TS-4. Fail-closed projection: a construct outside the D3 subset aborts naming the schema
/// identifier rather than emitting an unsound approximation. Driven through the public seam
/// directly, because the compiled-in registry cannot carry such a construct: the schemas come
/// from schemars over real contract types, and `patternProperties` appears in none of them.
///
/// The call goes through [`emit_typescript`] alone — `generate` inserts an artifact only when
/// every step returned `Ok`, so an emitter error aborts the whole run and leaves no `.ts` behind.
#[test]
fn unrepresentable_construct_aborts_generation() {
    let schema_id = "urn:ratatoskr:contracts:test:v1:Patterned";
    let schema = serde_json::json!({
        "$id": schema_id,
        "type": "object",
        "patternProperties": {"^x-": {"type": "string"}}
    });
    let error = emit_typescript(schema_id, &schema)
        .expect_err("patternProperties has no TypeScript projection");
    let rendered = error.to_string();
    assert!(rendered.contains("patternProperties"), "{rendered}");
    assert!(rendered.contains(schema_id), "{rendered}");
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

/// Every `(path, body)` of the TypeScript family, in map order.
fn typescript_subset(generated: &BTreeMap<PathBuf, String>) -> Vec<(&Path, &str)> {
    let mut subset = Vec::new();
    for (path, body) in generated {
        if let Ok(relative) = path.strip_prefix("generated/typescript") {
            let source: &str = String::as_str(body);
            subset.push((relative, source));
        }
    }
    subset
}

/// The final `:`-segment of a schema identifier:
/// `urn:ratatoskr:contracts:core:v1:EventEnvelope` → `EventEnvelope`.
fn root_type_name(schema_id: &str) -> &str {
    schema_id
        .rsplit(':')
        .next()
        .expect("rsplit always yields at least one item")
}

/// Byte offset of the export declaring `name`, whichever export form the emitter chose for it,
/// or `None`. Every occurrence of each form's needle is considered, because the *first* one
/// may sit inside a longer identifier — `export interface AiConversationAdded` contains the
/// `AiConversation` needle — while a later occurrence is the real declaration.
fn exported_declaration_position(body: &str, name: &str) -> Option<usize> {
    const FORMS: [&str; 4] = [
        "export interface ",
        "export type ",
        "export const ",
        "export enum ",
    ];
    FORMS
        .iter()
        .filter_map(|form| {
            let needle = format!("{form}{name}");
            let mut search_from = 0;
            while let Some(offset) = body.get(search_from..).and_then(|tail| tail.find(&needle)) {
                let position = search_from + offset;
                let end = position + needle.len();
                match body.get(end..).and_then(|tail| tail.chars().next()) {
                    // A declaration is followed by `{`, `=`, a newline — anything that is not
                    // itself part of a longer identifier.
                    Some(next) if !next.is_alphanumeric() && next != '_' && next != '$' => {
                        return Some(position);
                    }
                    Some(_) => search_from = end,
                    None => return None,
                }
            }
            None
        })
        .min()
}

/// `true` when `word` occurs as a standalone token — delimited by non-identifier characters —
/// so prose mentions (`import head`) and longer identifiers (`AiArchiveImport`) do not count.
fn contains_bare_word(text: &str, word: &str) -> bool {
    let mut token = String::new();
    for character in text.chars() {
        if character.is_ascii_alphanumeric() || character == '_' || character == '$' {
            token.push(character);
        } else {
            if token == word {
                return true;
            }
            token.clear();
        }
    }
    token == word
}

/// `true` when some line is an import *statement*: the keyword followed by a braced or starred
/// clause, a module string, or a `from` tail. The committed contracts do contain a property
/// literally named `import` (`AiArchiveImport.import`) and one named `imported_at`; neither
/// spelling imports anything, so both must pass.
fn has_import_statement(code: &str) -> bool {
    code.lines().any(|line| {
        let Some(rest) = line.trim_start().strip_prefix("import") else {
            return false;
        };
        let head = rest.trim_start();
        head.starts_with('{')
            || head.starts_with('*')
            || head.starts_with('"')
            || head.starts_with('\'')
            || rest.contains(" from ")
    })
}

/// Byte offsets of every `\d{4}-\d{2}-\d{2}` shape: the timestamp spelling TS-3 bans. A manual
/// scan over bytes rather than a regex dependency; every lookup goes through `get`, so an
/// offset at the tail of a short file simply fails to match instead of panicking.
fn iso_date_positions(text: &str) -> Vec<usize> {
    let bytes = text.as_bytes();
    let byte_at = |index: usize| bytes.get(index).copied();
    let digit = |index: usize| byte_at(index).is_some_and(|byte| byte.is_ascii_digit());
    let hyphen = |index: usize| byte_at(index) == Some(b'-');
    let mut positions = Vec::new();
    for start in 0..bytes.len().saturating_sub(9) {
        if digit(start)
            && digit(start + 1)
            && digit(start + 2)
            && digit(start + 3)
            && hyphen(start + 4)
            && digit(start + 5)
            && digit(start + 6)
            && hyphen(start + 7)
            && digit(start + 8)
            && digit(start + 9)
        {
            positions.push(start);
        }
    }
    positions
}

/// The input with every `/* ... */` block comment removed, newlines preserved so offsets in the
/// remaining text stay meaningful. A manual scan rather than a regex dependency; string literals
/// containing `/*` cannot occur in the emitted code, whose only strings are quoted schema
/// identifiers and doc prose already inside comments.
fn strip_block_comments(text: &str) -> String {
    let mut stripped = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    let mut in_comment = false;
    while let Some(character) = characters.next() {
        match (in_comment, character) {
            (false, '/') if characters.peek() == Some(&'*') => {
                characters.next();
                in_comment = true;
            }
            (true, '*') if characters.peek() == Some(&'/') => {
                characters.next();
                in_comment = false;
            }
            (comment_held, character) => {
                if !comment_held {
                    stripped.push(character);
                } else if character == '\n' {
                    stripped.push('\n');
                }
            }
        }
    }
    stripped
}
