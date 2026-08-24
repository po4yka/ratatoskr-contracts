//! The `check-typescript` verb — tests V-1 to V-4.
//!
//! `docs/TESTING.md` T-adjacent: the editing-loop compile verifier. Every process spawn sits
//! behind the injectable runner, so these cover success, diagnostics and a missing compiler
//! without requiring `tsc` on the machine (design D7).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion in a test binary is the reporting mechanism, not a hazard"
)]

use std::io;
use std::path::{Path, PathBuf};

use ratatoskr_contractsc::{
    GENERATOR_VERSION, Metadata,
    typescript::{CompileVerdict, SpawnOutcome, check_typescript_with},
};

/// The repository root, derived from the compiled-in manifest directory.
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

/// A runner that always succeeds silently: the clean-compilation world.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the runner signature is infallibility-agnostic; these stand-ins simply never fail"
)]
fn compiling(_dir: &Path, _program: &str, _args: &[String]) -> io::Result<SpawnOutcome> {
    Ok(SpawnOutcome {
        success: true,
        output: String::new(),
    })
}

/// A runner that reports one fixed tsc diagnostic and non-zero status.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the runner signature is infallibility-agnostic; these stand-ins simply never fail"
)]
fn failing(_dir: &Path, _program: &str, _args: &[String]) -> io::Result<SpawnOutcome> {
    Ok(SpawnOutcome {
        success: false,
        output: "message.ts:1:7 - error TS2322: Type 'number' is not assignable to type 'string'."
            .to_owned(),
    })
}

/// A runner that can never spawn anything: no override resolves, no npx exists.
fn nothing_installed(_dir: &Path, _program: &str, _args: &[String]) -> io::Result<SpawnOutcome> {
    Err(io::Error::new(io::ErrorKind::NotFound, "no such file"))
}

/// A runner standing in for npx on a machine without TypeScript: the process runs, then
/// cancels because the `tsc` package is absent.
fn npx_without_typescript(
    _dir: &Path,
    program: &str,
    _args: &[String],
) -> io::Result<SpawnOutcome> {
    if program == "npx" {
        Ok(SpawnOutcome {
            success: false,
            output: "npm error npx canceled due to missing packages and no YES option: \
                     [\"tsc\"]"
                .to_owned(),
        })
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "no such file"))
    }
}

/// V-1. A clean strict-mode compilation exits zero.
#[test]
fn clean_compilation_exits_zero() {
    let verdict = check_typescript_with(&metadata(), GENERATOR_VERSION, None, compiling)
        .expect("the verb runs");
    assert_eq!(verdict, CompileVerdict::Compiled);
}

/// V-2. A type error surfaces the compiler's diagnostic and a non-zero outcome.
#[test]
fn a_compiler_diagnostic_is_surfaced() {
    let verdict = check_typescript_with(&metadata(), GENERATOR_VERSION, None, failing)
        .expect("the verb runs");
    match verdict {
        CompileVerdict::Diagnostics(output) => {
            assert!(
                output.contains("TS2322"),
                "the diagnostic must reach the caller verbatim: {output}"
            );
        }
        other => panic!("expected diagnostics, got {other:?}"),
    }
}

/// V-3. When neither the environment override nor local resolution spawns a compiler, the
/// verdict is unavailable with guidance that names both escape hatches.
#[test]
fn an_unavailable_compiler_yields_actionable_guidance() {
    let verdict = check_typescript_with(&metadata(), GENERATOR_VERSION, None, nothing_installed)
        .expect("the verb runs");
    match verdict {
        CompileVerdict::Unavailable(guidance) => {
            assert!(
                guidance.contains("CONTRACTSC_TSC"),
                "guidance must name the override variable: {guidance}"
            );
            assert!(
                guidance.contains("npm install") || guidance.contains("typescript"),
                "guidance must say how to get a compiler: {guidance}"
            );
        }
        other => panic!("expected actionable unavailability, got {other:?}"),
    }
}

/// V-4. The environment override wins over local resolution: when it works, its program is
/// what gets invoked.
#[test]
fn the_environment_override_is_preferred() {
    let invoked_programs = std::cell::RefCell::new(Vec::new());
    let verdict = check_typescript_with(
        &metadata(),
        GENERATOR_VERSION,
        Some("/opt/homebrew/bin/tsc"),
        |_dir, program, _args| {
            invoked_programs.borrow_mut().push(program.to_owned());
            Ok(SpawnOutcome {
                success: true,
                output: String::new(),
            })
        },
    )
    .expect("the verb runs");
    assert_eq!(verdict, CompileVerdict::Compiled);
    assert_eq!(
        invoked_programs.borrow().as_slice(),
        ["/opt/homebrew/bin/tsc"],
        "the override must be the first and only program attempted"
    );
}

/// V-5. A fallback that *runs* but cannot resolve a compiler — npx cancelling on a missing
/// `tsc` package — is unavailability with guidance, not a compiler diagnostic.
#[test]
fn an_npx_resolution_cancellation_is_guidance_not_a_diagnostic() {
    let verdict =
        check_typescript_with(&metadata(), GENERATOR_VERSION, None, npx_without_typescript)
            .expect("the verb runs");
    match verdict {
        CompileVerdict::Unavailable(guidance) => {
            assert!(
                guidance.contains("CONTRACTSC_TSC"),
                "guidance must name the override variable: {guidance}"
            );
        }
        other => panic!("expected actionable unavailability, got {other:?}"),
    }
}
