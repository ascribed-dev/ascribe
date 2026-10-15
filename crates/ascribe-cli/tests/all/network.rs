//! Nothing in the binary but `ascribe sources fetch` and `ascribe sources
//! update` can open a connection (decision 14 of the docs plan): no crate
//! links a network library or uses sockets, the only `git` subcommands that
//! reach another repository are run from one module of `ascribe-sources`, and
//! only those two commands call into it. `tests/sources.rs` holds the same
//! from the outside, by running every other command with `git`'s network
//! turned off.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every Rust file under `crates/*/src`, by its path from the repository's
/// root, with `/` separators.
fn sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let rel = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((rel, fs::read_to_string(&path).unwrap()));
            }
        }
    }
    let root = repo();
    let mut out = Vec::new();
    for entry in fs::read_dir(root.join("crates")).unwrap() {
        let src = entry.unwrap().path().join("src");
        if src.is_dir() {
            walk(&src, &root, &mut out);
        }
    }
    assert!(out.len() > 50, "found only {} files", out.len());
    out
}

#[test]
fn no_network_library_is_linked() {
    let lock = fs::read_to_string(repo().join("Cargo.lock")).unwrap();
    for name in [
        "reqwest",
        "hyper",
        "ureq",
        "curl",
        "isahc",
        "attohttpc",
        "git2",
        "gix",
        "tokio",
        "async-std",
        "smol",
        "native-tls",
        "rustls",
        "openssl",
    ] {
        assert!(
            !lock.contains(&format!("\nname = \"{name}\"\n")),
            "Cargo.lock has {name}: the binary links no network, git, or async library"
        );
    }
    for (path, text) in sources() {
        for socket in ["std::net", "TcpStream", "TcpListener", "UdpSocket"] {
            assert!(!text.contains(socket), "{path} uses {socket}");
        }
    }
}

#[test]
fn only_the_sources_module_reaches_another_repository() {
    // `git` subcommands that can talk to another repository.
    let reaching = [
        "\"fetch\"",
        "\"clone\"",
        "\"pull\"",
        "\"push\"",
        "\"ls-remote\"",
        "\"remote\"",
        "\"submodule\"",
        "\"archive\"",
    ];
    let mut found = Vec::new();
    for (path, text) in sources() {
        for word in reaching {
            if text.contains(word) {
                found.push(format!("{path}: {word}"));
            }
        }
    }
    assert_eq!(
        found,
        ["crates/ascribe-sources/src/remote.rs: \"fetch\""],
        "only ascribe-sources' remote module may run a git command that reaches another repository"
    );
    // The module is private to its crate, and the crate's two functions that
    // use it are called by the two commands alone.
    let lib = fs::read_to_string(repo().join("crates/ascribe-sources/src/lib.rs")).unwrap();
    assert!(
        lib.contains("\nmod remote;\n"),
        "remote is a private module"
    );
    let mut callers = Vec::new();
    for (path, text) in sources() {
        if path.starts_with("crates/ascribe-sources/") {
            continue;
        }
        for call in ["ascribe_sources::fetch(", "ascribe_sources::update("] {
            if text.contains(call) {
                callers.push(format!("{path}: {call}"));
            }
        }
    }
    assert_eq!(
        callers,
        [
            "crates/ascribe-cli/src/commands/sources.rs: ascribe_sources::fetch(",
            "crates/ascribe-cli/src/commands/sources.rs: ascribe_sources::update(",
        ]
    );
}
