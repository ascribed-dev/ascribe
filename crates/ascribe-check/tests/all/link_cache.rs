//! The answers `ascribe report links` reuses (`ascribe_check::links::cache`)
//! and what it remembers (`ascribe_check::links::history`): what a second
//! run asks about and holds, and that only the report's code opens either.

#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use ascribe_check::Project;
use ascribe_check::links::{self, Answer, Cache, LinkChecker, Request};
use ascribe_check::tool::ToolError;
use ascribe_core::{Date, FileId, RelPath};

const PAGE: &str = "# Links\n\n[Fine](https://fine.test/a), [gone](https://gone.test/b), \
                    [moved](https://moved.test/c), and [fine again](https://fine.test/d).\n";

/// Answers by host, as the conformance stand-in does, and keeps what it was
/// asked about.
#[derive(Default)]
struct StandIn {
    asked: Mutex<Vec<Vec<String>>>,
    /// Every address answers.
    all_fine: bool,
}

impl LinkChecker for StandIn {
    fn check(&self, request: &Request) -> Result<BTreeMap<String, Answer>, ToolError> {
        self.asked
            .lock()
            .expect("not poisoned")
            .push(request.urls.clone());
        if self.all_fine {
            return Ok(BTreeMap::new());
        }
        Ok(request
            .urls
            .iter()
            .filter_map(|url| {
                let answer = match links::host(url)? {
                    "gone.test" => Answer::Broken {
                        status: "404 Not Found".to_owned(),
                    },
                    "moved.test" => Answer::Moved {
                        to: url.replace("moved.test", "new.test"),
                        code: 301,
                    },
                    _ => return None,
                };
                Some((url.clone(), answer))
            })
            .collect())
    }
}

impl StandIn {
    fn last(&self) -> Vec<String> {
        self.asked
            .lock()
            .expect("not poisoned")
            .last()
            .cloned()
            .unwrap_or_default()
    }
}

fn project(root: &Path, model: &str) -> Project {
    let model_text = format!("spec = \"0.1\"\n{model}");
    let model = ascribe_model::load_str(&model_text, FileId::new(0)).expect("a model");
    let sources =
        Project::from_sources([(RelPath::parse("index.md").expect("a path"), PAGE.into())]);
    Project::from_parts(root.to_owned(), RelPath::root(), model, model_text, sources)
}

fn day(s: &str) -> Date {
    Date::parse(s).expect("a date")
}

fn run(project: &Project, checker: &StandIn, today: &str, recheck: bool) -> links::Checked {
    let cache = Cache::open(project, project.root(), day(today), recheck);
    links::check(
        project,
        checker,
        Duration::from_secs(5),
        Duration::ZERO,
        Some(&cache),
        None,
    )
    .expect("it runs")
}

#[test]
fn a_second_run_asks_only_about_what_had_something_to_report() {
    let dir = tempfile::tempdir().expect("a folder");
    let project = project(dir.path(), "");
    let checker = StandIn::default();

    let first = run(&project, &checker, "2026-10-01", false);
    assert_eq!((first.checked, first.asked, first.reused), (4, 4, 0));
    assert_eq!(first.oldest_answer, None);
    assert_eq!(first.diagnostics.len(), 2);

    let second = run(&project, &checker, "2026-10-03", false);
    assert_eq!((second.checked, second.asked, second.reused), (4, 2, 2));
    assert_eq!(second.oldest_answer, Some(day("2026-10-01")));
    assert_eq!(
        checker.last(),
        ["https://gone.test/b", "https://moved.test/c"]
    );
    // Every finding was asked about in this run.
    assert_eq!(second.diagnostics.len(), 2);

    // `--recheck` asks about every address.
    let rechecked = run(&project, &checker, "2026-10-03", true);
    assert_eq!((rechecked.asked, rechecked.reused), (4, 0));

    // Past every address's limit, each is asked about again, and cleanup
    // deleted the old entries before that.
    let later = run(&project, &checker, "2026-11-01", false);
    assert_eq!((later.asked, later.reused), (4, 0));
}

#[test]
fn reuse_days_of_0_asks_every_time_and_keeps_nothing() {
    let dir = tempfile::tempdir().expect("a folder");
    let project = project(dir.path(), "[checks.links]\nreuse-days = 0\n");
    let checker = StandIn::default();
    run(&project, &checker, "2026-10-01", false);
    let second = run(&project, &checker, "2026-10-01", false);
    assert_eq!((second.asked, second.reused), (4, 0));
    assert!(!dir.path().join(".ascribe").exists());
}

#[test]
fn an_address_that_fails_now_is_forgotten() {
    let dir = tempfile::tempdir().expect("a folder");
    let project = project(dir.path(), "");
    // Every address answered on the first run...
    let answered = StandIn {
        all_fine: true,
        ..StandIn::default()
    };
    run(&project, &answered, "2026-10-01", false);
    // ...and a recheck finds two that don't, so the next run asks again.
    let checker = StandIn::default();
    let rechecked = run(&project, &checker, "2026-10-02", true);
    assert_eq!(rechecked.diagnostics.len(), 2);
    let next = run(&project, &checker, "2026-10-02", false);
    assert_eq!(
        checker.last(),
        ["https://gone.test/b", "https://moved.test/c"]
    );
    assert_eq!(next.diagnostics.len(), 2);
}

#[test]
fn a_changed_lychee_toml_asks_about_everything() {
    let dir = tempfile::tempdir().expect("a folder");
    let checker = StandIn::default();
    run(&project(dir.path(), ""), &checker, "2026-10-01", false);
    fs::write(dir.path().join("lychee.toml"), "timeout = 5\n").expect("written");
    let second = run(&project(dir.path(), ""), &checker, "2026-10-02", false);
    assert_eq!((second.asked, second.reused), (4, 0));
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("a directory") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// What opens the store or the history.
const OPENS: &[&str] = &[
    ".ascribe/cache",
    "links::cache",
    "links::Cache",
    "Cache::open(",
    ".ascribe/history",
    "links::history",
    "links::History",
    "History::open(",
];

/// The report's code: the only code that opens `.ascribe/cache/` or
/// `.ascribe/history/`. `check`,
/// `build`, `diff`, `drift`, `fmt`, the language server, and the hook never
/// read or write it, so their outputs stay a function of the sources.
const REPORT: &[&str] = &[
    "crates/ascribe-check/src/links.rs",
    "crates/ascribe-check/src/links/",
    "crates/ascribe-query/src/report.rs",
    "crates/ascribe-cli/src/commands/report/",
];

#[test]
fn only_the_report_opens_the_cache_or_the_history() {
    let root = workspace();
    let mut files = Vec::new();
    for entry in fs::read_dir(root.join("crates")).expect("crates/") {
        let src = entry.expect("a crate").path().join("src");
        if src.is_dir() {
            rust_files(&src, &mut files);
        }
    }
    let mut found = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("in the workspace")
            .to_string_lossy()
            .replace('\\', "/");
        if REPORT
            .iter()
            .any(|r| rel == *r || (r.ends_with('/') && rel.starts_with(r)))
        {
            continue;
        }
        let text = fs::read_to_string(&file).expect("a source file");
        for (i, line) in text.lines().enumerate() {
            if OPENS.iter().any(|o| line.contains(o)) {
                found.push(format!("{rel}:{}: {}", i + 1, line.trim()));
            }
        }
    }
    assert!(
        found.is_empty(),
        "Only `ascribe report`'s code opens `.ascribe/cache/` or `.ascribe/history/`: these do too:\n{}",
        found.join("\n")
    );
}
