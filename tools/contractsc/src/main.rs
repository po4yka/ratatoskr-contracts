//! `contractsc` — the command-line front end. The only place in the generator that touches disk
//! for writing.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use clap::Parser as _;
use ratatoskr_contractsc::compat::ReportFormat;
use ratatoskr_contractsc::{GENERATOR_VERSION, Metadata, api, compat, generate};

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

/// The verbs `DEVELOPMENT.md` names.
#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Write every generated artifact. Writes a file only when its bytes differ.
    Generate,
    /// Read-only gate: metadata validation, drift detection, the field lint, fixture validation
    /// and the secret scan. Writes nothing, ever. Exit 1 on any finding.
    Check,
    /// Compile the current generated TypeScript under a throwaway strict-mode project.
    /// Not part of the repository gate. Exit 1 on diagnostics or a missing compiler.
    CheckTypescript,
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
    /// Rewrite every contract crate's public-API baseline under `compat/api/`. Requires the
    /// `cargo-public-api` binary. This is the bless path for an approved public-API change:
    /// rerun, review the diff, commit it with the change that caused it.
    ApiWrite,
    /// Regenerate every crate's public API in memory and diff against the committed baselines
    /// under `compat/api/`. Exit 1 when anything differs — additive differences included.
    /// Not part of the repository gate; CI runs it in its own compatibility job.
    ApiCheck,
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
        Command::CheckTypescript => run_check_typescript(&root),
        Command::Compat { old, new, format } => run_compat(&old, &new, format),
        Command::ApiWrite => run_api_write(&root),
        Command::ApiCheck => run_api_check(&root),
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

/// `contractsc check-typescript`.
///
/// The environment override is read here, at the process boundary; the library stays free of
/// environment lookups, which is what keeps the determinism guarantees honest.
fn run_check_typescript(root: &Path) -> anyhow::Result<std::process::ExitCode> {
    use ratatoskr_contractsc::typescript::{CompileVerdict, SpawnOutcome, check_typescript_with};

    let metadata = load_metadata(root)?;
    let env_override = std::env::var("CONTRACTSC_TSC").ok();
    let verdict = check_typescript_with(
        &metadata,
        GENERATOR_VERSION,
        env_override.as_deref(),
        |directory, program, arguments| {
            std::process::Command::new(program)
                .args(arguments)
                .current_dir(directory)
                .output()
                .map(|output| SpawnOutcome {
                    success: output.status.success(),
                    output: format!(
                        "{}{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    ),
                })
        },
    )?;
    let exit_code = match &verdict {
        CompileVerdict::Compiled => std::process::ExitCode::SUCCESS,
        _ => std::process::ExitCode::FAILURE,
    };
    match verdict {
        CompileVerdict::Compiled => println!("typescript: strict-mode compilation succeeded"),
        CompileVerdict::Diagnostics(output) => eprint!("{output}"),
        CompileVerdict::Unavailable(guidance) => eprintln!("{guidance}"),
    }
    Ok(exit_code)
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

/// The package names of every contract crate, derived from the same registry that drives
/// generation, so a crate cannot gain or lose a baseline without gaining or losing a root type.
fn contract_crate_names() -> Vec<String> {
    api::contract_crate_names().into_iter().collect()
}

/// The manifest path of one contract crate.
///
/// The package `ratatoskr-<short>` lives in `crates/<short>`; the leading hyphenated prefix is
/// what differs between a package name and a directory name here. A future mismatch between that
/// spelling rule and the tree fails loudly right here, never silently.
fn crate_manifest(root: &Path, package_name: &str) -> anyhow::Result<PathBuf> {
    let short = package_name
        .strip_prefix("ratatoskr-")
        .unwrap_or(package_name);
    let manifest = root.join("crates").join(short).join("Cargo.toml");
    anyhow::ensure!(
        manifest.is_file(),
        "{} is missing; a registered root type names package {package_name}",
        manifest.display()
    );
    Ok(manifest)
}

/// Runs `cargo public-api` against one crate manifest and returns its stdout.
///
/// Plain output is already one public item per line; the compact spellings are the omit flags
/// (`-s`, `--omit`), and there has never been a `--short-text`. Process spawning lives here, at
/// the boundary, for the same reason the environment override of `check-typescript` does: the
/// library stays free of everything that makes output depend on the machine.
fn public_api_text(root: &Path, crate_name: &str) -> anyhow::Result<String> {
    let manifest = crate_manifest(root, crate_name)?;
    let output = std::process::Command::new("cargo")
        .arg("public-api")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .context(
            "cannot run cargo public-api; install it with: cargo install cargo-public-api --locked",
        )?;
    anyhow::ensure!(
        output.status.success(),
        "cargo public-api failed for {crate_name}:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// The provenance line every baseline records: which producer version, which compiler.
///
/// The repository pins stable as its only supported toolchain, so `resolve_toolchain` inside
/// cargo-public-api hands the doc build to the `nightly` channel; the version recorded here is
/// that compiler's, read through the same resolution, never the active stable's.
fn producer_versions() -> anyhow::Result<String> {
    let tool = std::process::Command::new("cargo")
        .args(["public-api", "--version"])
        .output()
        .context(
            "cannot run cargo public-api; install it with: cargo install cargo-public-api --locked",
        )?;
    let compiler = std::process::Command::new("rustup")
        .args(["run", "nightly", "rustc", "--version"])
        .output()
        .context("cannot ask the nightly toolchain for its rustc version")?;
    anyhow::ensure!(
        compiler.status.success(),
        "the nightly toolchain is not usable: {}{}",
        String::from_utf8_lossy(&compiler.stdout),
        String::from_utf8_lossy(&compiler.stderr)
    );
    Ok(format!(
        "{}, over {}",
        String::from_utf8_lossy(&tool.stdout).trim(),
        String::from_utf8_lossy(&compiler.stdout).trim()
    ))
}

/// `contractsc api-write`.
fn run_api_write(root: &Path) -> anyhow::Result<std::process::ExitCode> {
    let producer = producer_versions()?;
    for crate_name in contract_crate_names() {
        let document = api::render_baseline(
            &crate_name,
            &producer,
            &api::snapshot_items(&public_api_text(root, &crate_name)?),
        );
        let relative = PathBuf::from(api::API_BASELINE_DIR).join(format!("{crate_name}.txt"));
        let absolute = root.join(&relative);
        if std::fs::read_to_string(&absolute).is_ok_and(|existing| existing == document) {
            println!("unchanged {}", relative.display());
            continue;
        }
        if let Some(parent) = absolute.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        std::fs::write(&absolute, document)
            .with_context(|| format!("cannot write {}", absolute.display()))?;
        println!("wrote     {}", relative.display());
    }
    Ok(std::process::ExitCode::SUCCESS)
}

/// `contractsc api-check`.
fn run_api_check(root: &Path) -> anyhow::Result<std::process::ExitCode> {
    let mut dirty = 0usize;
    for crate_name in contract_crate_names() {
        let baseline_path = root
            .join(api::API_BASELINE_DIR)
            .join(format!("{crate_name}.txt"));
        let committed = std::fs::read_to_string(&baseline_path).with_context(|| {
            format!(
                "{} is missing; run `cargo contracts api-write` and commit it",
                baseline_path.display()
            )
        })?;
        let current = public_api_text(root, &crate_name)?;
        let diff = api::classify(&committed, &current);
        print!("{}", diff.report(&crate_name));
        dirty += usize::from(!diff.is_clean());
    }
    if dirty == 0 {
        println!(
            "api: every contract crate matches its committed baseline under {}/",
            api::API_BASELINE_DIR
        );
        return Ok(std::process::ExitCode::SUCCESS);
    }
    eprintln!(
        "api: {dirty} crate(s) differ from the committed baselines. An intentional change is \
         blessed by rerunning `cargo contracts api-write`, reviewing the diff, and committing it."
    );
    Ok(std::process::ExitCode::FAILURE)
}
