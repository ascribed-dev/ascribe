//! `ascribe diff` and `ascribe drift` on a project loaded from disk, in a
//! temporary repository, each with a typed result.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::process::Command;

use tessera_diff::{DiffError, DiffOptions, DriftOptions, PageStatus, diff_project, drift_project};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[builds.site]\n";

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// A repository whose one commit has two pages, then a working tree with one
/// of them changed and a broken link added.
fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q", "-b", "main"]);
    write(root, "ascribe.toml", MODEL);
    write(root, "docs/index.md", "# Home\n\nHello.\n");
    write(root, "docs/guide.md", "# Guide\n\nSteps.\n");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "first"]);
    write(
        root,
        "docs/guide.md",
        "# Guide\n\nOther steps. See [nothing](missing.md).\n",
    );
    dir
}

fn load(root: &Path) -> tessera_check::Project {
    tessera_check::Project::load(&root.join("ascribe.toml")).unwrap()
}

#[test]
fn diff_project_reports_the_changed_pages_and_the_working_trees_errors() {
    let dir = repository();
    let project = load(dir.path());
    let options = DiffOptions {
        base: Some("main"),
        ..DiffOptions::default()
    };
    let diff = diff_project(&project, &options).unwrap();
    let report = &diff.report;
    assert_eq!(report.base.requested, "main");
    assert_eq!(report.builds.len(), 1);
    let pages = &report.builds[0].pages;
    assert_eq!(pages.len(), 1, "{pages:?}");
    assert_eq!(pages[0].path, "guide.md");
    assert_eq!(pages[0].status, PageStatus::Changed);
    let errors = tessera_check::diagnose(&project, &[])
        .unwrap()
        .diagnostics
        .iter()
        .filter(|d| d.severity == tessera_check::Severity::Error)
        .count();
    assert!(errors > 0);
    assert_eq!(report.working_tree_errors, errors);
    assert!(diff.html().contains("<html"));
}

#[test]
fn an_unknown_build_is_a_typed_error() {
    let dir = repository();
    let project = load(dir.path());
    let builds = ["web".to_owned()];
    let options = DiffOptions {
        builds: &builds,
        ..DiffOptions::default()
    };
    match diff_project(&project, &options) {
        Err(DiffError::UnknownBuild(e)) => assert_eq!(e.name, "web"),
        other => panic!("expected an unknown build, got {other:?}"),
    }
    let options = DriftOptions {
        builds: &builds,
        ..DriftOptions::default()
    };
    assert!(matches!(
        drift_project(&project, &options),
        Err(DiffError::UnknownBuild(_))
    ));
}

#[test]
fn drift_project_compares_with_the_revision_asked_for() {
    let dir = repository();
    let project = load(dir.path());
    let options = DriftOptions {
        base: Some("HEAD"),
        base_exact: true,
        builds: &[],
    };
    let report = drift_project(&project, &options).unwrap();
    assert_eq!(report.base.requested, "HEAD");
    assert!(report.pages.is_empty(), "no page has an example");
    assert!(!report.needs_reading());
}
