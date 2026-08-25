//! `contractsc` — the deterministic contract generator and gate for `ratatoskr-contracts`.
//!
//! # Shape
//!
//! [`generate`] is a **pure function**: no clock, environment variable, hostname, current
//! directory, username, absolute path or git hash ever reaches its output. Identical inputs give
//! identical bytes on any machine, in any directory, at any time. All disk access lives in
//! `main.rs` and inside [`check`], which is what makes the determinism tests meaningful rather
//! than tautological.
//!
//! # The clean-checkout guarantee
//!
//! `ARCHITECTURE.md` S11 requires a clean tree after regeneration. [`generate`] writes exactly the
//! bytes [`check`] compares against, and both come from the same pure function, so
//! `cargo contracts generate && git diff --exit-code` passes on an unmodified checkout.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod api;
pub mod check;
pub mod compat;
pub mod fixtures;
pub mod lint;
pub mod metadata;
pub mod normalize;
pub mod provenance;
pub mod registry;
pub mod render;
pub mod secrets;
pub mod typescript;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use crate::check::{CheckReport, Finding};
pub use crate::metadata::Metadata;
pub use crate::typescript::emit_typescript;

/// This generator's own version, recorded in every provenance block.
///
/// A compile-time constant, never a runtime lookup: the recorded identity must be reproducible
/// from the source tree alone.
pub const GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The directory every generated JSON Schema artifact lives under.
pub const SCHEMAS_DIR: &str = "schemas";

/// The suffix that marks a generated JSON Schema artifact.
pub const SCHEMA_SUFFIX: &str = ".schema.json";

/// Everything that can stop the generator, as opposed to a contract problem it reports.
///
/// Contract problems are [`Finding`]s. A `GenError` means the generator itself could not run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GenError {
    /// A generated schema contained a number that is not an integer (§6.6 rule 6).
    NonIntegerNumber {
        /// JSON pointer of the offending number.
        pointer: String,
        /// The number, as rendered.
        value: String,
    },
    /// A file could not be read or written.
    Io {
        /// The path that failed.
        path: PathBuf,
        /// The operating system's explanation.
        detail: String,
    },
    /// `contracts.toml` could not be parsed.
    Metadata {
        /// The TOML parse error, which `deny_unknown_fields` makes precise.
        detail: String,
    },
    /// A normalized schema used a construct outside the supported TypeScript subset, so no sound
    /// TypeScript projection exists. The generator never emits an unsound approximation.
    UnrepresentableConstruct {
        /// The `$id` of the offending schema.
        schema_id: String,
        /// The construct, spelled as the document spells it.
        construct: String,
    },
}

impl std::fmt::Display for GenError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonIntegerNumber { pointer, value } => write!(
                formatter,
                "{pointer} is {value}, which is not an integer; floating point is banned in \
                 generated schemas because its rendering varies between platforms"
            ),
            Self::Io { path, detail } => write!(formatter, "{}: {detail}", path.display()),
            Self::Metadata { detail } => write!(formatter, "contracts.toml: {detail}"),
            Self::UnrepresentableConstruct {
                schema_id,
                construct,
            } => write!(
                formatter,
                "{construct} has no TypeScript projection in {schema_id}; the generator never \
                 emits an unsound approximation"
            ),
        }
    }
}

impl std::error::Error for GenError {}

/// The whole generator: the compiled Rust types plus the parsed metadata, in, the exact bytes of
/// every generated artifact, out.
///
/// Keyed by repository-relative output path, so the caller decides where the repository is and
/// the function itself never learns.
///
/// # Errors
///
/// [`GenError::NonIntegerNumber`] if any generated schema contains a float.
pub fn generate(
    metadata: &Metadata,
    generator_version: &str,
) -> Result<BTreeMap<PathBuf, String>, GenError> {
    let mut generated = BTreeMap::new();
    for root in registry::root_types() {
        // A root the metadata does not declare is reported by rule R2; it is not this function's
        // job to guess an `$id` for it.
        let Some((contract, declared)) = metadata.contract_of(root.rust_path) else {
            continue;
        };

        let mut generator = render::generator();
        let mut schema = (root.schema)(&mut generator);
        normalize::sort_required(&mut schema);
        normalize::assert_no_floats(&schema)?;
        schema.insert("$id".to_owned(), serde_json::json!(declared.schema_id));
        schema.insert("title".to_owned(), serde_json::json!(root.short_name()));

        // Steps 5–7 of §6.4 *are* the digest definition: the hash is taken over the fully
        // rendered schema with the provenance member absent, so it is a pure function of contract
        // shape and a `generator_version` bump moves no digest.
        let provenance_free_body = render::render(&schema);
        let digest = provenance::source_digest(&provenance_free_body);
        schema.insert(
            provenance::PROVENANCE_KEY.to_owned(),
            provenance::block(
                root.rust_path,
                &contract.id,
                contract.major_version,
                generator_version,
                &digest,
            ),
        );

        generated.insert(PathBuf::from(&declared.output), render::render(&schema));

        // The TypeScript family mirrors the JSON Schema family one-to-one (design D1): the path
        // is derived mechanically from the same `output` field, and the emitter consumes the
        // very value the JSON renderer serialized — provenance block included, so the `.ts`
        // header can name the same contract identity (D2, D5).
        let Some(typescript_output) = typescript::typescript_output_path(&declared.output) else {
            continue;
        };
        let typescript_body = typescript::emit_typescript(&declared.schema_id, schema.as_value())?;
        generated.insert(PathBuf::from(typescript_output), typescript_body);
    }
    Ok(generated)
}

/// Reads the tree and reports every finding. Writes nothing, ever.
///
/// Runs, in order: metadata rules R1–R14, drift detection against the generated bytes, the orphan
/// sweep, the field lint L1–L7, fixture expectation checking, and the secret/PII scan.
///
/// # Errors
///
/// [`GenError::Io`] when `contracts.toml` cannot be read, [`GenError::Metadata`] when it cannot be
/// parsed, and anything [`generate`] returns. Contract problems are `Finding`s, not `Err`.
pub fn check(root: &Path) -> Result<CheckReport, GenError> {
    let metadata_path = root.join(Metadata::FILE_NAME);
    let text = std::fs::read_to_string(&metadata_path).map_err(|error| GenError::Io {
        path: metadata_path,
        detail: error.to_string(),
    })?;
    let metadata = Metadata::parse(&text).map_err(|detail| GenError::Metadata { detail })?;
    let generated = generate(&metadata, GENERATOR_VERSION)?;

    let mut findings = metadata::validate(&metadata, root, &generated);
    findings.extend(drift(root, &generated));
    findings.extend(orphans(root, &generated));
    findings.extend(lint::run(&metadata, &generated));
    findings.extend(fixtures::validate(&metadata, root));
    findings.extend(secrets::scan_tree(root));
    Ok(CheckReport::new(findings))
}

/// Compares every expected artifact against disk.
fn drift(root: &Path, generated: &BTreeMap<PathBuf, String>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (relative, expected) in generated {
        let Ok(actual) = std::fs::read_to_string(root.join(relative)) else {
            findings.push(Finding::Missing {
                path: relative.clone(),
            });
            continue;
        };
        // `first_difference` compares the whole byte sequence, terminators included, and returns
        // `None` only when the two documents are byte-identical. It is therefore the byte
        // comparison itself, not a prettier report built on top of one: `check` and
        // `git diff --exit-code` cannot disagree.
        if let Some((line, expected_line, actual_line)) = first_difference(expected, &actual) {
            findings.push(Finding::Stale {
                path: relative.clone(),
                line,
                expected: expected_line,
                actual: actual_line,
            });
        }
        // A file whose own body no longer implies its own digest was edited by hand. That is a
        // different failure from routine staleness, and `AGENTS.md` forbids it outright. Each
        // family recomputes through its own provenance spelling: a JSON extension keyword for
        // `schemas/`, the leading block comment for `generated/typescript/`.
        let is_typescript = relative
            .extension()
            .is_some_and(|extension| extension == "ts");
        let self_consistent = if is_typescript {
            typescript::recompute_digest(&actual)
                .ok()
                .zip(typescript::embedded_digest(&actual))
                .is_some_and(|(recomputed, embedded)| recomputed == embedded)
        } else {
            provenance::recompute_digest(&actual)
                .ok()
                .zip(provenance::embedded_digest(&actual))
                .is_some_and(|(recomputed, embedded)| recomputed == embedded)
        };
        if !self_consistent {
            findings.push(Finding::Tampered {
                path: relative.clone(),
            });
        }
    }
    findings
}

/// Sweeps the managed output directories for artifacts no registered root type produces: a
/// leftover `*.schema.json` under `schemas/**` and, identically, a stray `.ts` under
/// [`typescript::TYPESCRIPT_DIR`] — files `git diff` alone would never catch.
fn orphans(root: &Path, generated: &BTreeMap<PathBuf, String>) -> Vec<Finding> {
    let mut findings = Vec::new();
    let sweeps = [
        (Path::new(SCHEMAS_DIR), SCHEMA_SUFFIX),
        (Path::new(typescript::TYPESCRIPT_DIR), ".ts"),
    ];
    for (directory, suffix) in sweeps {
        // `walk_all`, not `walk_json`: the TypeScript family is `.ts` files, which the JSON
        // filter would silently drop and the sweep would never see.
        for absolute in fixtures::walk_all(&root.join(directory)) {
            let Ok(relative) = absolute.strip_prefix(root) else {
                continue;
            };
            if !absolute
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(suffix))
            {
                continue;
            }
            if !generated.contains_key(relative) {
                findings.push(Finding::Orphan {
                    path: relative.to_path_buf(),
                });
            }
        }
    }
    findings
}

/// The first differing line of two documents: `(one-based line, expected, actual)`, each
/// truncated to 200 characters. `None` **only** when the two documents are byte-identical.
///
/// The split is `split_inclusive('\n')`, not `lines()`, so each compared piece carries its own
/// terminator. That is what makes the comparison total: `lines()` discards the trailing newline
/// and any trailing `\r`, so a file converted to CRLF or missing its final newline would compare
/// equal here while `git diff` reported it dirty — the §6.6 rule-7/rule-8 hole this function is
/// the only guard for. Concatenating the pieces reproduces the input exactly, so two documents
/// that differ in any byte differ in some piece.
#[must_use]
pub fn first_difference(expected: &str, actual: &str) -> Option<(usize, String, String)> {
    let mut expected_lines = expected.split_inclusive('\n');
    let mut actual_lines = actual.split_inclusive('\n');
    let mut line = 0usize;
    loop {
        line += 1;
        match (expected_lines.next(), actual_lines.next()) {
            (None, None) => return None,
            (left, right) => {
                if left == right {
                    continue;
                }
                return Some((line, describe(left), describe(right)));
            }
        }
    }
}

/// Renders one line for a report, truncated to 200 characters, with the terminator spelled out.
///
/// A difference confined to the terminator — a CRLF, or a missing final newline — is invisible
/// unless the escape is printed, and an unescaped `\n` would break the report across lines.
fn describe(line: Option<&str>) -> String {
    let Some(text) = line else {
        return "<end of file>".to_owned();
    };
    truncate(&text.replace('\r', "\\r").replace('\n', "\\n"))
}

/// Clamps a reported line to 200 characters so a report stays readable, with the last character
/// spent on an ellipsis when anything was dropped.
fn truncate(line: &str) -> String {
    if line.chars().count() <= 200 {
        return line.to_owned();
    }
    line.chars().take(199).collect::<String>() + "…"
}
