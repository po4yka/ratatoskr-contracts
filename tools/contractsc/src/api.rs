//! The Rust public-API compatibility checker: what changed between the committed baseline of a
//! crate's exported surface and the surface the current sources produce.
//!
//! The seven contract crates are consumed across the Ratatoskr workspace, so a change to what
//! they export — a removed type, a renamed function, a brand-new public item — is a cross-
//! repository event even when it is semver-compatible. This module owns the comparison and the
//! classification; producing the current snapshot (`cargo-public-api`) and touching disk stay in
//! `main.rs`, which keeps every function here pure and unit-testable without the external tool.
//!
//! # Stated limit, mirroring `compat.rs`
//!
//! Line-set comparison sees presence, absence and signature text. It cannot see a *meaning*
//! change behind an unchanged signature; only review catches that. A green `api-check` is never a
//! semantic guarantee.

use std::collections::BTreeSet;
use std::fmt::Write as _;

/// The directory every committed public-API baseline lives under, relative to the repository
/// root: one `<package-name>.txt` per contract crate.
pub const API_BASELINE_DIR: &str = "compat/api";

/// The package names of every contract crate, derived from the same registry that drives
/// generation. A crate cannot gain or lose a baseline without gaining or losing a registered
/// root type, which keeps this list honest by construction. The Rust-side crate identifier's
/// underscores spell the package name with hyphens (`ratatoskr_event_envelope` publishes as
/// `ratatoskr-event-envelope`).
#[must_use]
pub fn contract_crate_names() -> BTreeSet<String> {
    crate::registry::root_types()
        .into_iter()
        .filter_map(|root| root.rust_path.split("::").next().map(str::to_owned))
        .map(|crate_name| crate_name.replace('_', "-"))
        .collect()
}

/// The difference between two public-API snapshots of one crate.
///
/// `breaking` holds lines present in the baseline and absent now — a removed public item, or an
/// item whose signature text changed, which surfaces as a remove paired with an add.
/// `additive` holds lines absent from the baseline and present now. Additive differences fail the
/// check too: consumers compile against the full exported surface, so an addition is exactly the
/// change this repository must see coming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiDiff {
    /// Baseline-only items: removed or changed public surface.
    pub breaking: Vec<String>,
    /// Current-only items: newly exported public surface.
    pub additive: Vec<String>,
}

impl ApiDiff {
    /// `true` when the two snapshots export exactly the same items.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.breaking.is_empty() && self.additive.is_empty()
    }

    /// The human-readable report for one crate, in the classifier's own voice.
    #[must_use]
    pub fn report(&self, crate_name: &str) -> String {
        if self.is_clean() {
            return format!("unchanged  {crate_name}\n");
        }
        let mut report = String::new();
        if !self.breaking.is_empty() {
            let _ = writeln!(
                report,
                "BREAKING   {crate_name} removes or changes {} public item(s):",
                self.breaking.len()
            );
            for item in &self.breaking {
                let _ = writeln!(report, "  - {item}");
            }
        }
        if !self.additive.is_empty() {
            let _ = writeln!(
                report,
                "ADDITIVE   {crate_name} exports {} new public item(s):",
                self.additive.len()
            );
            for item in &self.additive {
                let _ = writeln!(report, "  + {item}");
            }
        }
        let _ = writeln!(
            report,
            "an intentional change is blessed by: cargo contracts api-write"
        );
        report
    }
}

/// Extracts the public-item lines from one `cargo-public-api --short-text` document.
///
/// Comment lines (`#`) are provenance, blank lines are layout; neither is API surface. Every
/// remaining line is one public item, deduplicated and sorted so the comparison below is a set
/// operation regardless of the order the producer chose.
#[must_use]
pub fn snapshot_items(document: &str) -> Vec<String> {
    let items: BTreeSet<&str> = document
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter(|line| !line.trim().is_empty())
        .collect();
    items.into_iter().map(str::to_owned).collect()
}

/// Renders one committed baseline file: provenance header, then the sorted item lines.
///
/// Deterministic by construction — sorted items, fixed wording, no clock — so two writers with
/// the same inputs produce byte-identical files.
#[must_use]
pub fn render_baseline(crate_name: &str, produced_by: &str, items: &[String]) -> String {
    let mut rendered = String::new();
    let _ = write!(
        rendered,
        "# Public API baseline of {crate_name}, the frozen reference `api-check` diffs against.\n\
         # Produced by {produced_by}.\n\
         # Regenerate with: cargo contracts api-write\n"
    );
    for item in snapshot_items(&items.join("\n")) {
        let _ = writeln!(rendered, "{item}");
    }
    rendered
}

/// Classifies the difference between a committed baseline document and a freshly produced one.
///
/// Baseline-only items are [`ApiDiff::breaking`], current-only items are
/// [`ApiDiff::additive`].
#[must_use]
pub fn classify(baseline_document: &str, current_document: &str) -> ApiDiff {
    let baseline: BTreeSet<String> = snapshot_items(baseline_document).into_iter().collect();
    let current: BTreeSet<String> = snapshot_items(current_document).into_iter().collect();
    ApiDiff {
        breaking: baseline.difference(&current).cloned().collect(),
        additive: current.difference(&baseline).cloned().collect(),
    }
}
