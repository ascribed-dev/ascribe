//! Sets `ASCRIBE_VERSION`, what `ascribe --version` prints after the name: the
//! crate's version, followed by the commit when the build names one in
//! `ASCRIBE_COMMIT`. The nightly canary does, so a canary says what it was
//! built from; other builds print the version alone.

use std::env;

fn main() {
    println!("cargo::rerun-if-env-changed=ASCRIBE_COMMIT");
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let version = match env::var("ASCRIBE_COMMIT") {
        Ok(commit) if !commit.is_empty() => format!("{version} ({commit})"),
        _ => version,
    };
    println!("cargo::rustc-env=ASCRIBE_VERSION={version}");
}
