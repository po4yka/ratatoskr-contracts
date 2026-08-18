//! The read-only gate's report type and every finding it can carry.
//!
//! `cargo contracts check` never writes, so it can run against a read-only checkout. Contract
//! problems are [`Finding`]s rather than errors: the gate reports **all** of them in one run and
//! exits once, which is what makes it usable as a single CI step.

use std::path::PathBuf;

/// One problem the gate found.
///
/// [`Finding::Tampered`] is deliberately distinct from [`Finding::Stale`]: staleness is the
/// routine "the Rust source changed and nobody regenerated" case, while tampering means a
/// generated file was **edited by hand**, which `AGENTS.md` forbids outright.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Finding {
    /// An expected generated artifact does not exist on disk.
    Missing {
        /// Repository-relative path of the missing artifact.
        path: PathBuf,
    },
    /// A generated artifact exists but its bytes differ from what the generator produces.
    Stale {
        /// Repository-relative path of the stale artifact.
        path: PathBuf,
        /// One-based line number of the first difference.
        line: usize,
        /// The generator's line, truncated to 200 characters.
        expected: String,
        /// The line on disk, truncated to 200 characters.
        actual: String,
    },
    /// A `*.schema.json` under `schemas/**` that no registered root type produces.
    Orphan {
        /// Repository-relative path of the unclaimed artifact.
        path: PathBuf,
    },
    /// A generated artifact's embedded `source_digest` does not recompute from its own body.
    Tampered {
        /// Repository-relative path of the hand-edited artifact.
        path: PathBuf,
    },
    /// A `contracts.toml` rule R1–R14 failed.
    Metadata {
        /// The rule identifier, e.g. `R2`.
        rule: &'static str,
        /// What is wrong and how to fix it.
        detail: String,
    },
    /// A field lint rule L1–L7 failed.
    Lint {
        /// The rule identifier, e.g. `L1`.
        rule: &'static str,
        /// The offending property, as `TypeName#/properties/name`.
        pointer: String,
        /// What is wrong and how to fix it.
        detail: String,
    },
    /// A fixture does not meet its declared expectation.
    Fixture {
        /// Repository-relative path of the fixture.
        path: String,
        /// What is wrong and how to fix it.
        detail: String,
    },
    /// A fixture matches a secret or PII pattern.
    Secret {
        /// Repository-relative path of the fixture.
        path: String,
        /// The name of the tripped scanner rule.
        pattern: &'static str,
    },
}

impl Finding {
    /// A total, stable ordering key: `(kind, primary subject, secondary subject)`.
    ///
    /// Used to sort a report deterministically and to collapse findings that are duplicates in
    /// every respect. Every arm carries its discriminating text, so two genuinely different
    /// problems never collapse into one report line.
    #[must_use]
    pub fn sort_key(&self) -> (&'static str, String, String) {
        match self {
            Self::Missing { path } => ("missing", path.display().to_string(), String::new()),
            Self::Stale { path, line, .. } => {
                ("stale", path.display().to_string(), line.to_string())
            }
            Self::Orphan { path } => ("orphan", path.display().to_string(), String::new()),
            Self::Tampered { path } => ("tampered", path.display().to_string(), String::new()),
            Self::Metadata { rule, detail } => ("metadata", (*rule).to_owned(), detail.clone()),
            // `detail` is part of the key, exactly as it is for `Metadata` and `Fixture`: one
            // rule can fail on one property for two independent reasons (L4 fires separately for
            // an unknown timestamp name and for a missing clock authority), and collapsing those
            // would hide the second until the operator had fixed the first and re-run. Nothing
            // needs the coarser key: `lint::declared_types` already keys types by name, so a type
            // inlined into several documents is linted once.
            Self::Lint {
                rule,
                pointer,
                detail,
            } => ("lint", format!("{rule} {pointer}"), detail.clone()),
            Self::Fixture { path, detail } => ("fixture", path.clone(), detail.clone()),
            Self::Secret { path, pattern } => ("secret", path.clone(), (*pattern).to_owned()),
        }
    }
}

impl std::fmt::Display for Finding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { path } => {
                write!(
                    formatter,
                    "missing  {}: run `cargo contracts generate`",
                    path.display()
                )
            }
            Self::Stale {
                path,
                line,
                expected,
                actual,
            } => write!(
                formatter,
                "stale    {}:{line}\n           expected {expected}\n           actual   {actual}",
                path.display()
            ),
            Self::Orphan { path } => write!(
                formatter,
                "orphan   {}: no registered root type produces this file",
                path.display()
            ),
            Self::Tampered { path } => write!(
                formatter,
                "tampered {}: source_digest does not recompute; the file was edited by hand",
                path.display()
            ),
            Self::Metadata { rule, detail } => write!(formatter, "metadata {rule}: {detail}"),
            Self::Lint {
                rule,
                pointer,
                detail,
            } => {
                write!(formatter, "lint     {rule} {pointer}: {detail}")
            }
            Self::Fixture { path, detail } => write!(formatter, "fixture  {path} {detail}"),
            Self::Secret { path, pattern } => {
                write!(formatter, "secret   {path}: matches {pattern}")
            }
        }
    }
}

/// Everything one `check` run found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckReport {
    /// Every finding, sorted by [`Finding::sort_key`] and deduplicated.
    pub findings: Vec<Finding>,
}

impl CheckReport {
    /// Builds a report from unsorted findings.
    #[must_use]
    pub fn new(mut findings: Vec<Finding>) -> Self {
        findings.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        findings.dedup_by(|left, right| left.sort_key() == right.sort_key());
        Self { findings }
    }

    /// `true` when the tree is exactly what the generator produces and nothing else is wrong.
    ///
    /// This is the `Current` state of the specification's `check` table, and the
    /// `ARCHITECTURE.md` S11 clean-checkout guarantee.
    #[must_use]
    pub fn is_current(&self) -> bool {
        self.findings.is_empty()
    }

    /// The process exit code this report implies: `0` when current, `1` otherwise.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        i32::from(!self.is_current())
    }
}

impl std::fmt::Display for CheckReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_current() {
            return writeln!(
                formatter,
                "current: every contract artifact matches its canonical Rust source"
            );
        }
        for finding in &self.findings {
            writeln!(formatter, "{finding}")?;
        }
        write!(formatter, "{} finding(s)", self.findings.len())
    }
}
