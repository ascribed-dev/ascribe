//! The synthetic project has no diagnostics at any size, so timing its check
//! measures work and not reporting.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::print_stderr
)]

use tessera_check::{Project, check_all_builds};
use tessera_synthetic::Synthetic;

fn check(pages: usize) -> usize {
    let dir = tempfile::tempdir().expect("temp dir");
    Synthetic::new(pages).write_to(dir.path()).expect("writes");
    let project = Project::load(&dir.path().join("ascribe.toml")).expect("loads");
    let diagnostics = check_all_builds(&project);
    for d in &diagnostics {
        eprintln!("{}: {}", d.slug.as_str(), d.message);
    }
    diagnostics.len()
}

#[test]
fn small_projects_check_clean() {
    for pages in [1, 20, 300] {
        assert_eq!(check(pages), 0, "{pages} pages");
    }
}

/// The standard project. Slow in a debug build, so it runs with `--ignored`
/// (the CI job and the release checks do).
#[test]
#[ignore = "slow in debug builds; run with --release -- --ignored"]
fn the_standard_project_checks_clean() {
    assert_eq!(check(3000), 0);
}
