//! `contractsc` — the command-line front end. The only place in the generator that touches disk
//! for writing.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use clap::Parser as _;
use ratatoskr_contractsc::compat::ReportFormat;
use ratatoskr_contractsc::{GENERATOR_VERSION, Metadata, compat, generate};

/// Deterministic contract generator and gate for `ratatoskr-contracts`.
#[derive(Debug, clap::Parser)]
#[command(
    name = "contractsc",
    version,
    about = "Deterministic contract generator and gate for ratatoskr-contracts"
)]
struct Cli {
    /// Repository root. Defaults to the directory containing `contracts.toml`, found by walking up
    /// from `CARGO_MANIFEST_DIR`. Never derived from the current working directory, so the output
    /// cannot depend on where the command was invoked.
    #[arg(long, value_name = "PATH", global = true)]
    root: Option<PathBuf>,
    /// The verb.
    #[command(subcommand)]
    command: Command,
}

/// The three verbs `DEVELOPMENT.md` names.
#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Write every generated artifact. Writes a file only when its bytes differ.
    Generate,
    /// Read-only gate: metadata validation, drift detection, the field lint, fixture validation
    /// and the secret scan. Writes nothing, ever. Exit 1 on any finding.
    Check,
    /// Classify a change between two JSON Schema files.
    Compat {
        /// The baseline schema.
        old: PathBuf,
        /// The candidate schema.
        new: PathBuf,
        /// Report shape.
        #[arg(long, value_enum, default_value = "text")]
        format: ReportFormat,
    },
}

fn main() -> anyhow::Result<std::process::ExitCode> {
    let cli = Cli::parse();
    let root = match cli.root {
        Some(explicit) => explicit,
        None => discover_root().context("cannot find contracts.toml above CARGO_MANIFEST_DIR")?,
    };
    match cli.command {
        Command::Generate => run_generate(&root),
        Command::Check => run_check(&root),
        Command::Compat { old, new, format } => run_compat(&old, &new, format),
    }
}

/// Walks up from the compiled-in manifest directory to the directory holding `contracts.toml`.
fn discover_root() -> Option<PathBuf> {
    let mut candidate: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    while let Some(directory) = candidate {
        if directory.join(Metadata::FILE_NAME).is_file() {
            return Some(directory.to_path_buf());
        }
        candidate = directory.parent();
    }
    None
}

/// Reads and parses `contracts.toml`.
fn load_metadata(root: &Path) -> anyhow::Result<Metadata> {
    let path = root.join(Metadata::FILE_NAME);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("cannot read {}", path.display()))?;
    Metadata::parse(&text).map_err(|detail| anyhow::anyhow!("{}: {detail}", path.display()))
}

/// `contractsc generate`.
fn run_generate(root: &Path) -> anyhow::Result<std::process::ExitCode> {
    let metadata = load_metadata(root)?;
    let generated = generate(&metadata, GENERATOR_VERSION)?;
    for (relative, body) in &generated {
        let absolute = root.join(relative);
        if std::fs::read_to_string(&absolute).is_ok_and(|existing| existing == *body) {
            println!("unchanged {}", relative.display());
            continue;
        }
        if let Some(parent) = absolute.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        std::fs::write(&absolute, body)
            .with_context(|| format!("cannot write {}", absolute.display()))?;
        println!("wrote     {}", relative.display());
    }
    Ok(std::process::ExitCode::SUCCESS)
}

/// `contractsc check`.
fn run_check(root: &Path) -> anyhow::Result<std::process::ExitCode> {
    let report = ratatoskr_contractsc::check(root)?;
    println!("{report}");
    Ok(if report.is_current() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    })
}

/// `contractsc compat`.
fn run_compat(
    old: &Path,
    new: &Path,
    format: ReportFormat,
) -> anyhow::Result<std::process::ExitCode> {
    let baseline = read_schema(old)?;
    let current = read_schema(new)?;
    let findings = compat::classify(&baseline, &current);
    print!("{}", compat::report(&findings, format)?);
    Ok(if compat::is_blocking(&findings) {
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    })
}

/// Reads one JSON Schema document.
fn read_schema(path: &Path) -> anyhow::Result<serde_json::Value> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("{} is not JSON", path.display()))
}
