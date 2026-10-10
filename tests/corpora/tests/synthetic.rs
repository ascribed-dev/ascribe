//! The synthetic project has no errors or warnings at any size, and the
//! standard one no advice either, so timing its check measures work and not
//! reporting. A smaller one has fewer pages than fragments and images, so
//! the content checks advise on what nothing uses.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::print_stderr
)]

use ascribe_check::{Project, Severity, check_all_builds};
use ascribe_synthetic::Synthetic;

/// How many diagnostics the project has; advice too, with `advice`.
fn check(project: Synthetic, advice: bool) -> usize {
    let dir = tempfile::tempdir().expect("temp dir");
    project.write_to(dir.path()).expect("writes");
    let project = Project::load(&dir.path().join("ascribe.toml")).expect("loads");
    let diagnostics: Vec<_> = check_all_builds(&project)
        .into_iter()
        .filter(|d| advice || d.severity != Severity::Advice)
        .collect();
    for d in &diagnostics {
        eprintln!("{}: {}", d.slug.as_str(), d.message);
    }
    diagnostics.len()
}

#[test]
fn small_projects_check_clean() {
    for pages in [1, 20, 300] {
        assert_eq!(check(Synthetic::new(pages), false), 0, "{pages} pages");
        assert_eq!(
            check(Synthetic::new(pages).with_snippets(), false),
            0,
            "{pages} pages with snippets"
        );
    }
}

/// The standard project. Slow in a debug build, so it runs with `--ignored`
/// (the CI job and the release checks do).
#[test]
#[ignore = "slow in debug builds; run with --release -- --ignored"]
fn the_standard_project_checks_clean() {
    assert_eq!(check(Synthetic::standard(), true), 0);
    assert_eq!(check(Synthetic::standard().with_snippets(), true), 0);
}
