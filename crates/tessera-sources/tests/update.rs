//! `update`: moving a source's pin, and saying what it changes.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::{Code, Docs};
use tessera_sources::{FileChange, SourcesError, fetch, update};

const V1: &str = "// :snippet-start: login\nfn login() {}\n// :snippet-end:\n";
const V2: &str = "// :snippet-start: login\nfn login(user: &str) {}\n// :snippet-end:\n";

fn source(name: &str, code: &Code) -> String {
    format!(
        "[sources.{name}]\ngit = \"{}\"\nbranch = \"main\"\n",
        code.url()
    )
}

fn pinned(docs: &Docs, name: &str) -> String {
    let lock = docs.read("ascribe.lock").unwrap();
    let at = lock.find(&format!("name = \"{name}\"")).unwrap();
    let commit = &lock[at..];
    let start = commit.find("commit = \"").unwrap() + 10;
    commit[start..start + 40].to_owned()
}

#[test]
fn a_moved_pin_copies_the_files_again_and_lists_the_commits() {
    let code = Code::new();
    code.write("src/auth.rs", V1).write("src/other.rs", "x\n");
    let first = code.commit("first");
    let docs = Docs::new(&source("api", &code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs#login\n",
    );
    fetch(&docs.workspace(), &[], &docs.options()).unwrap();

    code.write("src/auth.rs", V2);
    code.commit("Take a user");
    code.write("src/other.rs", "y\n");
    let head = code.commit("Change another file");

    let report = update(&docs.workspace(), &[], None, &docs.options()).unwrap();
    assert!(report.changed);
    let api = &report.sources[0];
    assert!(api.moved);
    assert_eq!(api.from.as_deref(), Some(first.as_str()));
    assert_eq!(api.to, head);
    assert_eq!(api.followed, "main");
    let commits = api.commits.as_ref().unwrap();
    assert_eq!(commits.count, 2);
    let subjects: Vec<&str> = commits.newest.iter().map(|c| c.subject.as_str()).collect();
    assert_eq!(subjects, ["Change another file", "Take a user"]);
    // Only the copy a snippet uses changed.
    let files: Vec<_> = api
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.change))
        .collect();
    assert_eq!(files, [("src/auth.rs", FileChange::Changed)]);
    assert_eq!(docs.read("sources/api/src/auth.rs").as_deref(), Some(V2));
    assert_eq!(pinned(&docs, "api"), head);
    assert_eq!(docs.check(), Vec::<String>::new());
}

#[test]
fn with_nothing_to_move_no_file_changes() {
    let code = Code::new();
    code.write("src/auth.rs", V1);
    let first = code.commit("first");
    let docs = Docs::new(&source("api", &code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n",
    );
    fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    let lock = docs.read("ascribe.lock").unwrap();
    let modified = std::fs::metadata(common::path(docs.root(), "ascribe.lock"))
        .unwrap()
        .modified()
        .unwrap();

    let report = update(&docs.workspace(), &[], None, &docs.options()).unwrap();
    assert!(!report.changed);
    let api = &report.sources[0];
    assert!(!api.moved && api.files.is_empty() && api.commits.is_none());
    assert_eq!(api.to, first);
    assert_eq!(docs.read("ascribe.lock").unwrap(), lock);
    assert_eq!(
        std::fs::metadata(common::path(docs.root(), "ascribe.lock"))
            .unwrap()
            .modified()
            .unwrap(),
        modified
    );
}

#[test]
fn to_moves_the_pin_to_a_revision_back_or_forward() {
    let code = Code::new();
    code.write("src/auth.rs", V1);
    let first = code.commit("first");
    code.write("src/auth.rs", V2);
    let second = code.commit("second");
    code.git_tag("v1", &first);
    let docs = Docs::new(&source("api", &code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n",
    );

    // The first pin, by update, is the head of the branch.
    let report = update(&docs.workspace(), &[], None, &docs.options()).unwrap();
    assert_eq!(report.sources[0].to, second);
    assert!(report.sources[0].first_copy && report.sources[0].from.is_none());

    // Back to a tag.
    let report = update(
        &docs.workspace(),
        &["api".into()],
        Some("v1"),
        &docs.options(),
    )
    .unwrap();
    assert_eq!(report.sources[0].to, first);
    assert_eq!(docs.read("sources/api/src/auth.rs").as_deref(), Some(V1));
    // Forward to a commit, by its hash.
    let report = update(
        &docs.workspace(),
        &["api".into()],
        Some(&second),
        &docs.options(),
    )
    .unwrap();
    assert_eq!(report.sources[0].to, second);
    assert_eq!(report.sources[0].commits.as_ref().unwrap().count, 1);

    // An unknown revision fails with git's own message, and changes nothing.
    let lock = docs.read("ascribe.lock");
    let error = update(&docs.workspace(), &[], Some("nope"), &docs.options()).unwrap_err();
    assert!(
        matches!(&error, SourcesError::Git { name, message } if name == "api" && message.contains("nope")),
        "{error}"
    );
    assert_eq!(docs.read("ascribe.lock"), lock);
    assert!(matches!(
        update(&docs.workspace(), &[], Some("-x"), &docs.options()),
        Err(SourcesError::BadRevision(_))
    ));
}

#[test]
fn a_region_that_is_gone_is_left_for_check_to_report() {
    let code = Code::new();
    code.write("src/auth.rs", V1);
    code.commit("first");
    let docs = Docs::new(&source("api", &code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs#login\n",
    );
    fetch(&docs.workspace(), &[], &docs.options()).unwrap();

    code.write("src/auth.rs", "fn sign_in() {}\n");
    let head = code.commit("Rename login");
    let report = update(&docs.workspace(), &[], None, &docs.options()).unwrap();
    assert_eq!(report.sources[0].to, head);
    assert_eq!(pinned(&docs, "api"), head);
    let problems = docs.check();
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].starts_with("snippet-region-missing"),
        "{problems:?}"
    );
}

#[test]
fn two_sources_at_once_and_one_by_name() {
    let api = Code::new();
    api.write("src/auth.rs", V1);
    api.commit("first");
    let cli = Code::new();
    cli.write("cmd/main.go", "package main\n");
    let cli_first = cli.commit("first");
    let docs = Docs::new(&format!("{}{}", source("api", &api), source("cli", &cli)));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n\n@snippet: cli:cmd/main.go\n",
    );
    fetch(&docs.workspace(), &[], &docs.options()).unwrap();

    api.write("src/auth.rs", V2);
    let api_head = api.commit("second");
    cli.write("cmd/main.go", "package main\n\nfunc main() {}\n");
    let cli_head = cli.commit("second");

    // By name, only that one moves.
    let report = update(&docs.workspace(), &["api".into()], None, &docs.options()).unwrap();
    assert_eq!(report.sources.len(), 1);
    assert_eq!(pinned(&docs, "api"), api_head);
    assert_eq!(pinned(&docs, "cli"), cli_first);

    // Without names, every one; the one already at its head doesn't move.
    let report = update(&docs.workspace(), &[], None, &docs.options()).unwrap();
    let moved: Vec<(&str, bool)> = report
        .sources
        .iter()
        .map(|s| (s.name.as_str(), s.moved))
        .collect();
    assert_eq!(moved, [("api", false), ("cli", true)]);
    assert_eq!(pinned(&docs, "cli"), cli_head);
    assert!(matches!(
        update(&docs.workspace(), &[], Some("main"), &docs.options()),
        Err(SourcesError::ToNeedsOneSource)
    ));
}
