//! Sets `ASCRIBE_VERSION`, what `ascribe --version` prints after the name.
//!
//! - A release (`ASCRIBE_RELEASE=1`, set by the release workflow) prints the
//!   crate's version alone: `0.2.0`.
//! - The nightly canary names its commit in `ASCRIBE_COMMIT`, and its version
//!   is already stamped as a pre-release: `0.2.1-next.13 (42f00096ceb0)`.
//! - Any other build, such as one from `main`, is neither, and says so:
//!   `0.2.0+dev (42f00096ceb0)`, with the commit Git says the checkout is at,
//!   or `0.2.0+dev` alone outside a Git checkout. `+dev` is semantic
//!   versioning's build metadata, so a tool that compares versions still reads
//!   `0.2.0`, while a person sees it isn't the release.

// A build script talks to Cargo on standard output.
#![allow(clippy::print_stdout)]

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-env-changed=ASCRIBE_RELEASE");
    println!("cargo::rerun-if-env-changed=ASCRIBE_COMMIT");
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let version = if env::var("ASCRIBE_RELEASE").is_ok_and(|v| v == "1") {
        version
    } else {
        match env::var("ASCRIBE_COMMIT") {
            Ok(commit) if !commit.is_empty() => format!("{version} ({commit})"),
            _ => match checkout_commit() {
                Some(commit) => format!("{version}+dev ({commit})"),
                None => format!("{version}+dev"),
            },
        }
    };
    println!("cargo::rustc-env=ASCRIBE_VERSION={version}");
}

/// The commit the checkout is at, abbreviated to twelve characters as the
/// canary's is, when this is a Git checkout and Git can run. Asks Cargo to run
/// the script again when `HEAD`, or the branch it's on, moves.
fn checkout_commit() -> Option<String> {
    for path in [
        git(&["rev-parse", "--git-path", "HEAD"]),
        git(&["rev-parse", "--git-path", "packed-refs"]),
    ]
    .into_iter()
    .flatten()
    .chain(
        git(&["symbolic-ref", "-q", "HEAD"])
            .and_then(|branch| git(&["rev-parse", "--git-path", &branch])),
    ) {
        // Outside FileSystem: the build script asks whether a Git file exists,
        // since Cargo runs the script on every build when a watched file is
        // missing.
        if Path::new(&path).exists() {
            println!("cargo::rerun-if-changed={path}");
        }
    }
    git(&["rev-parse", "--short=12", "HEAD"])
}

/// Runs Git in the crate's folder, with no file system monitor, as every `git`
/// Ascribe starts, and returns its trimmed output when it succeeds.
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(["-c", "core.fsmonitor=false"])
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}
