//! What the commands call before and instead of printing: finding and
//! loading a project, choosing builds, and the diagnostics `ascribe check`
//! reports, each with a typed result.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;

use ascribe_check::{
    BASELINE_FILE, DiagnoseError, Diagnostic, LoadError, LocateError, MODEL_FILE, Project, Request,
    Scope, Severity, UnknownBuild, baseline, count_errors, diagnose, diagnose_with, select_builds,
};
use ascribe_core::RelPath;

const MODEL: &str =
    "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[builds.site]\n[builds.pdf]\n";

fn project(model: &str, pages: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    fs::write(dir.path().join(MODEL_FILE), model).expect("write the model");
    fs::create_dir_all(dir.path().join("docs")).expect("create docs");
    for (path, text) in pages {
        fs::write(dir.path().join("docs").join(path), text).expect("write a page");
    }
    dir
}

fn load(dir: &Path) -> Project {
    Project::load(&dir.join(MODEL_FILE)).expect("the project loads")
}

#[test]
fn locate_takes_a_file_or_the_folder_that_holds_one() {
    let dir = project(MODEL, &[]);
    let config = dir.path().join(MODEL_FILE);
    assert_eq!(Project::locate(Some(dir.path())).expect("found"), config);
    assert_eq!(Project::locate(Some(&config)).expect("found"), config);

    let missing = dir.path().join("nowhere.toml");
    let err = Project::locate(Some(&missing)).expect_err("not a file");
    assert!(matches!(&err, LocateError::NotAFile { path } if *path == missing));
    assert_eq!(
        err.to_string(),
        format!("{} doesn't exist or isn't a file", missing.display())
    );
}

#[test]
fn load_model_reads_only_the_content_model() {
    // A page that isn't UTF-8 doesn't matter to it.
    let dir = project(MODEL, &[]);
    fs::write(dir.path().join("docs/broken.md"), [0xff, 0xfe]).expect("write");
    let loaded = Project::load_model(&dir.path().join(MODEL_FILE)).expect("the model loads");
    assert_eq!(loaded.root, dir.path());
    assert_eq!(loaded.text, MODEL);
    assert_eq!(loaded.model.builds.len(), 2);

    let bad = project("spec = \"0.1\"\n[project\n", &[]);
    match Project::load_model(&bad.path().join(MODEL_FILE)) {
        Err(LoadError::Model { diagnostics, .. }) => {
            assert_eq!(diagnostics[0].slug.as_str(), "model-toml-syntax");
        }
        other => panic!("expected the model's errors, got {other:?}"),
    }
}

#[test]
fn select_builds_keeps_the_order_given_and_names_each_once() {
    let dir = project(MODEL, &[]);
    let project = load(dir.path());
    let names = |builds: Vec<&ascribe_model::Build>| -> Vec<String> {
        builds.iter().map(|b| b.name.clone()).collect()
    };
    let all = select_builds(&project, &[]).expect("every build");
    assert_eq!(names(all), ["site", "pdf"]);
    let asked = ["pdf".to_owned(), "site".to_owned(), "pdf".to_owned()];
    assert_eq!(
        names(select_builds(&project, &asked).expect("both")),
        ["pdf", "site"]
    );

    let err = select_builds(&project, &["web".to_owned()]).expect_err("no such build");
    assert_eq!(
        err,
        UnknownBuild {
            name: "web".to_owned(),
            known: vec!["site".to_owned(), "pdf".to_owned()],
        }
    );
    assert_eq!(
        err.to_string(),
        "the content model has no build `web`; its builds are site, pdf"
    );
}

#[test]
fn diagnose_returns_what_check_reports_and_the_builds() {
    let dir = project(
        MODEL,
        &[("index.md", "# Home\n\nSee [the guide](missing.md).\n")],
    );
    let project = load(dir.path());
    let found = diagnose(&project, &["site".to_owned()]).expect("checked");
    assert_eq!(found.builds.len(), 1);
    assert!(
        found
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error && d.slug.as_str() == "link-target-missing"),
        "{:?}",
        found.diagnostics
    );
    assert!(diagnose(&project, &["web".to_owned()]).is_err());
}

#[test]
fn count_errors_counts_what_diagnose_reports_as_errors() {
    // A broken link is an error; both pages are over a small size limit, and the
    // two pages nothing links to and the fragment nothing includes are
    // reported by the checks across the project, all advice unless
    // `[checks]` raises them.
    let page = "---\ntitle: Big\n---\n\nSee [nothing](missing.md). Some words to pass the limit.\n";
    let pages = [
        ("big.md", page),
        ("lonely.md", "---\ntitle: Lonely\n---\n\nHi.\n"),
        ("_part.md", "A part.\n"),
    ];
    let errors = |size: &str, across: &str| {
        let model = format!(
            "{MODEL}[checks]\npage-orphan = \"{across}\"\nfragment-unused = \"{across}\"\n\
             [checks.page-size]\nlevel = \"{size}\"\nlimit = 10\n"
        );
        let dir = project(&model, &pages);
        let project = load(dir.path());
        let reported = diagnose(&project, &[])
            .expect("every build")
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let counted = count_errors(&project, &[]).expect("every build");
        assert_eq!(counted, reported, "page-size {size}, across {across}");
        counted
    };
    assert_eq!(errors("advice", "advice"), 1);
    assert_eq!(errors("error", "advice"), 3);
    assert_eq!(errors("advice", "error"), 4);
}

#[test]
fn find_below_lists_the_projects_in_the_folders_below() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    for folder in [
        "b",
        "a",
        "a/nested",
        "node_modules/c",
        ".hidden",
        "none/deeper",
    ] {
        fs::create_dir_all(dir.path().join(folder)).expect("create a folder");
    }
    for folder in ["b", "a", "a/nested", "node_modules/c", ".hidden"] {
        fs::write(dir.path().join(folder).join(MODEL_FILE), MODEL).expect("write the model");
    }
    let found = Project::find_below(dir.path(), usize::MAX);
    let a = dir.path().join("a");
    assert_eq!(found, [a.clone(), a.join("nested"), dir.path().join("b")]);
    assert_eq!(
        Project::find_below(dir.path(), 2),
        [a.clone(), a.join("nested")]
    );

    // The error names them; with none, it says what it said before.
    let err = LocateError::not_found(dir.path().to_owned());
    assert!(
        matches!(&err, LocateError::NotFound { below, .. } if *below == found),
        "{err:?}"
    );
    let empty = dir.path().join("none");
    assert_eq!(
        LocateError::not_found(empty.clone()).to_string(),
        format!(
            "no {MODEL_FILE} found in {} or any parent directory; run ascribe from a project, \
             or pass --config",
            empty.display()
        )
    );
}

#[test]
fn a_request_applies_the_baseline_and_the_scope_for_every_surface() {
    // Two pages whose long `nav-label`s are advice (`nav-label-long`), and
    // a home page that links to both.
    let page = |title: &str, label: &str| {
        format!("---\ntitle: {title}\nnav-label: {label}\n---\n\nText.\n")
    };
    let sync = page("Sync", "Synchronize two copies");
    let keys = page("Keys", "All about the API keys");
    let home = "---\ntitle: Home\n---\n\nSee [sync](sync.md) and [keys](keys.md).\n";
    let model = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n";
    let dir = project(
        model,
        &[("index.md", home), ("sync.md", &sync), ("keys.md", &keys)],
    );
    let before = load(dir.path());
    let found = diagnose(&before, &[]).expect("every build");
    let written = baseline::write(&before, None, &found.diagnostics);
    assert_eq!(written.entries, 2, "{}", written.text);
    fs::write(dir.path().join(BASELINE_FILE), &written.text).expect("write the baseline");
    let project = load(dir.path());
    let long = |d: &Diagnostic| d.slug.as_str() == "nav-label-long";

    // The whole project: both held back, and counted.
    let whole = Request {
        baseline: true,
        ..Request::default()
    };
    let checked = diagnose_with(&project, &whole).expect("checked");
    assert!(!checked.reported.iter().any(|r| long(&r.diagnostic)));
    assert_eq!(checked.baselined.len(), 2);
    assert_eq!(checked.builds, ["site"]);

    // Without the baseline, both are reported, as `diagnose` finds them.
    let without = diagnose_with(&project, &Request::default()).expect("checked");
    assert_eq!(without.baselined.len(), 0);
    assert_eq!(without.into_diagnostics(), found.diagnostics);

    // With a scope, only what counts for it is reported or counted.
    let scope = Scope::new([RelPath::parse("docs/sync.md").expect("a path")]);
    let scoped = Request {
        scope: Some(&scope),
        ..whole
    };
    let checked = diagnose_with(&project, &scoped).expect("checked");
    assert_eq!(checked.baselined.len(), 1);
    assert!(
        checked
            .reported
            .iter()
            .all(|r| scope.contains_file(&project, r.diagnostic.location.file)),
        "{:?}",
        checked.reported
    );

    // An entry that matches nothing is stale for the whole project only.
    fs::write(dir.path().join("docs/sync.md"), page("Sync", "Sync")).expect("fix the label");
    let project = load(dir.path());
    let stale = |request: &Request<'_>| {
        diagnose_with(&project, request)
            .expect("checked")
            .into_diagnostics()
            .iter()
            .filter(|d| d.slug.as_str() == "baseline-entry-unmatched")
            .count()
    };
    assert_eq!(stale(&whole), 1);
    assert_eq!(stale(&scoped), 0);
    let editor = Request {
        editor_build: true,
        ..whole
    };
    assert_eq!(
        stale(&editor),
        0,
        "the editor's build alone isn't the whole"
    );

    // A baseline that can't be read stops a request that asks for it.
    fs::write(dir.path().join(BASELINE_FILE), "not a baseline\n").expect("break it");
    let project = load(dir.path());
    assert!(matches!(
        diagnose_with(&project, &whole),
        Err(DiagnoseError::Baseline(_))
    ));
    assert!(diagnose_with(&project, &Request::default()).is_ok());
}
