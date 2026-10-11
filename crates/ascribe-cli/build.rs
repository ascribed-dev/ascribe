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
    // `HEAD` moves on a checkout, and a detached `HEAD` on a commit.
    // `packed-refs` changes when a packed branch moves without a loose ref.
    let files = [
        git(&["rev-parse", "--git-path", "HEAD"]),
        git(&["rev-parse", "--git-path", "packed-refs"]),
    ];
    // Outside FileSystem: the build script asks whether Git's files exist,
    // since Cargo runs the script on every build when a watched path is
    // missing.
    for path in files.into_iter().flatten() {
        if Path::new(&path).exists() {
            println!("cargo::rerun-if-changed={path}");
        }
    }
    // A branch's ref is a loose file only until `git gc` packs it, and the
    // next commit writes a new one. Watching the nearest folder that holds it
    // sees that file appear, since Cargo takes a folder as changed when any
    // file in it is.
    if let Some(branch) = git(&["symbolic-ref", "-q", "HEAD"])
        .and_then(|branch| git(&["rev-parse", "--git-path", &branch]))
        && let Some(folder) = Path::new(&branch).ancestors().skip(1).find(|p| p.is_dir())
    {
        println!("cargo::rerun-if-changed={}", folder.display());
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
