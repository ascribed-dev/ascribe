//! `update`: moving a source's pin, and saying what it changes.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use ascribe_sources::{FileChange, SourcesError, fetch, update};
use common::{Code, Docs};

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
    fetch(&docs.view(), &[], &docs.options()).unwrap();

    code.write("src/auth.rs", V2);
    code.commit("Take a user");
    code.write("src/other.rs", "y\n");
    let head = code.commit("Change another file");

    let report = update(&docs.view(), &[], None, &docs.options()).unwrap();
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
    fetch(&docs.view(), &[], &docs.options()).unwrap();
    let lock = docs.read("ascribe.lock").unwrap();
    let modified = std::fs::metadata(common::path(docs.root(), "ascribe.lock"))
        .unwrap()
        .modified()
        .unwrap();

    let report = update(&docs.view(), &[], None, &docs.options()).unwrap();
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

/// Pins are fetched without their history, so the cache can hold both ends
/// of a move with nothing between them: the commits are still counted from
/// the whole history, and a move back isn't counted at all.
#[test]
fn commits_are_counted_from_the_history_not_the_shallow_cache() {
    let code = Code::new();
    code.write("src/auth.rs", V1);
    let first = code.commit("first");
    for n in 2..=5 {
        code.write("src/other.rs", format!("{n}\n"));
        code.commit(&format!("commit {n}"));
    }
    code.git_tag("v1", &first);
    let docs = Docs::new(&source("api", &code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n",
    );
    // The first pin, the head, fetched alone.
    update(&docs.view(), &[], None, &docs.options()).unwrap();

    let back = update(&docs.view(), &[], Some("v1"), &docs.options()).unwrap();
    let api = &back.sources[0];
    assert!(api.moved && api.back, "{api:?}");
    assert_eq!(api.commits, None);

    let forward = update(&docs.view(), &[], None, &docs.options()).unwrap();
    let api = &forward.sources[0];
    assert!(api.moved && !api.back, "{api:?}");
    let commits = api.commits.as_ref().unwrap();
    assert_eq!(commits.count, 4);
    assert_eq!(commits.newest[0].subject, "commit 5");
    assert_eq!(commits.newest.len(), 4);
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
    let report = update(&docs.view(), &[], None, &docs.options()).unwrap();
    assert_eq!(report.sources[0].to, second);
    assert!(report.sources[0].first_copy && report.sources[0].from.is_none());

    // Back to a tag.
    let report = update(&docs.view(), &["api".into()], Some("v1"), &docs.options()).unwrap();
    assert_eq!(report.sources[0].to, first);
    assert_eq!(docs.read("sources/api/src/auth.rs").as_deref(), Some(V1));
    // Forward to a commit, by its hash.
    let report = update(
        &docs.view(),
        &["api".into()],
        Some(&second),
        &docs.options(),
    )
    .unwrap();
    assert_eq!(report.sources[0].to, second);
    assert_eq!(report.sources[0].commits.as_ref().unwrap().count, 1);

    // An unknown revision fails with git's own message, and changes nothing.
    let lock = docs.read("ascribe.lock");
    let error = update(&docs.view(), &[], Some("nope"), &docs.options()).unwrap_err();
    assert!(
        matches!(&error, SourcesError::Git { name, message } if name == "api" && message.contains("nope")),
        "{error}"
    );
    assert_eq!(docs.read("ascribe.lock"), lock);
    assert!(matches!(
        update(&docs.view(), &[], Some("-x"), &docs.options()),
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
    fetch(&docs.view(), &[], &docs.options()).unwrap();

    code.write("src/auth.rs", "fn sign_in() {}\n");
    let head = code.commit("Rename login");
    let report = update(&docs.view(), &[], None, &docs.options()).unwrap();
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
    fetch(&docs.view(), &[], &docs.options()).unwrap();

    api.write("src/auth.rs", V2);
    let api_head = api.commit("second");
    cli.write("cmd/main.go", "package main\n\nfunc main() {}\n");
    let cli_head = cli.commit("second");

    // By name, only that one moves.
    let report = update(&docs.view(), &["api".into()], None, &docs.options()).unwrap();
    assert_eq!(report.sources.len(), 1);
    assert_eq!(pinned(&docs, "api"), api_head);
    assert_eq!(pinned(&docs, "cli"), cli_first);

    // Without names, every one; the one already at its head doesn't move.
    let report = update(&docs.view(), &[], None, &docs.options()).unwrap();
    let moved: Vec<(&str, bool)> = report
        .sources
        .iter()
        .map(|s| (s.name.as_str(), s.moved))
        .collect();
    assert_eq!(moved, [("api", false), ("cli", true)]);
    assert_eq!(pinned(&docs, "cli"), cli_head);
    assert!(matches!(
        update(&docs.view(), &[], Some("main"), &docs.options()),
        Err(SourcesError::ToNeedsOneSource)
    ));
}

/// An observation listing each of `files` (path and contents) as tested.
fn observation(files: &[(&str, &str)]) -> String {
    let mut text = String::from(
        "version = 1\nascribe = \"0.3.0\"\ncommand = \"npm test\"\n\n[environment]\nos = \"linux\"\narch = \"x64\"\n\n[files]\n",
    );
    for (path, contents) in files {
        text.push_str(&format!(
            "\"{path}\" = \"{}\"\n",
            ascribe_model::example_hash(contents.as_bytes())
        ));
    }
    text
}

#[test]
fn an_update_says_which_examples_went_stale_lost_their_observation_or_are_current() {
    use ascribe_sources::{ExampleState, ExampleUpdate};
    let code = Code::new();
    code.write("ascribe.examples.toml", "include = [\"*.rs\"]\n")
        .write("a.rs", "a\n")
        .write("b.rs", "b\n")
        .write("c.rs", "c\n")
        .write(
            "examples.observations/default.toml",
            observation(&[("a.rs", "a\n"), ("b.rs", "b\n")]),
        );
    code.commit("first");
    let docs = Docs::new(&source("api", &code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:a.rs\n\n@snippet: api:b.rs\n\n@snippet: api:c.rs\n",
    );
    fetch(&docs.view(), &[], &docs.options()).unwrap();
    let examples = |report: ascribe_sources::UpdateReport| report.sources[0].examples.clone();
    let example = |path: &str, status, was| ExampleUpdate {
        path: path.to_owned(),
        status,
        was: Some(was),
    };

    // `a` changes without a test run; the observation drops `b`, and lists
    // `c` as it is.
    code.write("a.rs", "a2\n").write(
        "examples.observations/default.toml",
        observation(&[("a.rs", "a\n"), ("c.rs", "c\n")]),
    );
    code.commit("second");
    let report = update(&docs.view(), &[], None, &docs.options()).unwrap();
    assert_eq!(
        examples(report),
        [
            example("a.rs", ExampleState::Stale, ExampleState::Current),
            example("b.rs", ExampleState::Unobserved, ExampleState::Current),
            example("c.rs", ExampleState::Current, ExampleState::Unobserved),
        ]
    );

    // The observation follows `a`: it's current again, and nothing else
    // changed.
    code.write(
        "examples.observations/default.toml",
        observation(&[("a.rs", "a2\n"), ("c.rs", "c\n")]),
    );
    code.commit("third");
    let report = update(&docs.view(), &[], None, &docs.options()).unwrap();
    assert_eq!(
        examples(report),
        [example("a.rs", ExampleState::Current, ExampleState::Stale)]
    );

    // No move, nothing to say.
    let report = update(&docs.view(), &[], None, &docs.options()).unwrap();
    assert!(examples(report).is_empty());
}

#[test]
fn an_update_counts_an_example_current_when_any_observation_has_it_as_it_is() {
    use ascribe_sources::{ExampleState, ExampleUpdate};
    let code = Code::new();
    code.write("ascribe.examples.toml", "include = [\"*.rs\"]\n")
        .write("a.rs", "a\n")
        .write("b.rs", "b\n")
        .write(
            "examples.observations/default.toml",
            observation(&[("a.rs", "a\n"), ("b.rs", "b\n")]),
        )
        .write(
            "examples.observations/node-22.toml",
            observation(&[("a.rs", "a\n"), ("b.rs", "b\n")]),
        );
    code.commit("first");
    let docs = Docs::new(&source("api", &code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:a.rs\n\n@snippet: api:b.rs\n",
    );
    fetch(&docs.view(), &[], &docs.options()).unwrap();

    // Both change; only `node-22` is run again, and only `a` passes it as
    // it is now. `a` stays current, by `node-22`; `b` goes stale.
    code.write("a.rs", "a2\n").write("b.rs", "b2\n").write(
        "examples.observations/node-22.toml",
        observation(&[("a.rs", "a2\n")]),
    );
    code.commit("second");
    let report = update(&docs.view(), &[], None, &docs.options()).unwrap();
    assert_eq!(
        report.sources[0].examples,
        [ExampleUpdate {
            path: "b.rs".to_owned(),
            status: ExampleState::Stale,
            was: Some(ExampleState::Current),
        }]
    );
}

#[test]
fn a_planned_update_writes_nothing_until_applied_and_reaches_the_repository_once() {
    use ascribe_sources::{ExampleState, ExampleUpdate};
    let code = Code::new();
    code.write("ascribe.examples.toml", "include = [\"*.rs\"]\n")
        .write("a.rs", "a\n")
        .write(
            "examples.observations/default.toml",
            observation(&[("a.rs", "a\n")]),
        );
    code.commit("first");
    let docs = Docs::new(&source("api", &code));
    docs.page("index.md", "---\ntitle: Home\n---\n\n@snippet: api:a.rs\n");
    fetch(&docs.view(), &[], &docs.options()).unwrap();
    let lock = docs.read("ascribe.lock").unwrap();

    code.write("a.rs", "a2\n");
    let second = code.commit("second");
    let view = docs.view().planning(true);
    let report = update(&view, &[], None, &docs.options()).unwrap();
    // The report is the one writing would give: the example went stale,
    // read from the copies as the plan leaves them.
    assert!(report.changed);
    assert_eq!(report.sources[0].to, second);
    assert_eq!(
        report.sources[0].examples,
        [ExampleUpdate {
            path: "a.rs".to_owned(),
            status: ExampleState::Stale,
            was: Some(ExampleState::Current),
        }]
    );
    let planned: Vec<String> = view
        .planned()
        .iter()
        .map(|p| {
            p.strip_prefix(docs.root())
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    assert_eq!(planned, ["sources/api/a.rs", "ascribe.lock"]);
    // Nothing is written yet.
    assert_eq!(docs.read("sources/api/a.rs").as_deref(), Some("a\n"));
    assert_eq!(docs.read("ascribe.lock").as_deref(), Some(lock.as_str()));

    // A commit after the plan isn't fetched: applying makes what was
    // planned, without reaching the repository again.
    code.write("a.rs", "a3\n");
    code.commit("third");
    view.apply().unwrap();
    assert_eq!(docs.read("sources/api/a.rs").as_deref(), Some("a2\n"));
    assert_eq!(pinned(&docs, "api"), second);
    assert!(view.planned().is_empty());
}
