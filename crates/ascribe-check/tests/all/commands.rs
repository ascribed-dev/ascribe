//! What the commands call before and instead of printing: finding and
//! loading a project, choosing builds, and the diagnostics `ascribe check`
//! reports, each with a typed result.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;

use ascribe_check::{
    LoadError, LocateError, MODEL_FILE, Project, Severity, UnknownBuild, count_errors, diagnose,
    select_builds,
};

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
