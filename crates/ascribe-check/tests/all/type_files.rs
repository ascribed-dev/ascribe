//! The files a content type names, `template` and `guide`
//! (`ascribe_check::type_files`): where each path may lead, and that a
//! template reads as a page. The conformance cases under
//! `tests/conformance/cases/model/type-*` hold the rule's examples; these are
//! its edges, and the links, which a case can't hold on every platform.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ascribe_check::type_files::{check, load_model};
use ascribe_core::{FileId, Issue};
use ascribe_model::SourceBound;
use ascribe_resolve::{DiskFs, Layout, MemoryFs};

const MODEL: &str = r#"spec = "0.1"

[types.how-to]
template = "templates/how-to.md"
guide = ["templates/how-to.guide.md", "templates/notes.md"]

[types.how-to.frontmatter]
title = "string"
"#;

const TEMPLATE: &str = "---\ntitle: \"Title\"\n---\n\nTo \\{do something}, follow these steps.\n";

fn issues(model: &str, files: &[(&str, &str)]) -> Vec<Issue> {
    let model = ascribe_model::load_str(model, FileId::new(0)).expect("the model loads");
    let mut fs = MemoryFs::new(&Layout::from_model(&model));
    for (path, text) in files {
        fs = fs.with_file(path, text);
    }
    check(&model, &fs)
}

fn variants(issues: &[Issue]) -> Vec<(&str, Option<&str>)> {
    issues
        .iter()
        .map(|i| (i.slug.as_str(), i.variant))
        .collect()
}

fn complete() -> Vec<(&'static str, &'static str)> {
    vec![
        ("templates/how-to.md", TEMPLATE),
        ("templates/how-to.guide.md", "# How-to guide\n"),
        ("templates/notes.md", "Our notes.\n"),
    ]
}

#[test]
fn a_type_whose_files_are_there_has_no_problem() {
    assert_eq!(issues(MODEL, &complete()), []);
}

#[test]
fn a_type_that_names_no_files_reads_nothing() {
    let model = "spec = \"0.1\"\n";
    assert_eq!(issues(model, &[]), []);
}

#[test]
fn a_problem_is_at_the_path_in_ascribe_toml() {
    let mut files = complete();
    files.retain(|(p, _)| *p != "templates/notes.md");
    let found = issues(MODEL, &files);
    assert_eq!(variants(&found), [("model-type-file", None)]);
    let span = found[0].location.span;
    assert_eq!(&MODEL[span.range()], "\"templates/notes.md\"");
}

#[test]
fn a_name_that_differs_only_in_case_is_missing_and_says_so() {
    let model = MODEL.replace("templates/notes.md", "templates/Notes.md");
    let found = issues(&model, &complete());
    assert_eq!(variants(&found), [("model-type-file", Some("case"))]);
    assert_eq!(found[0].arg("actual"), Some("templates/notes.md"));
}

#[test]
fn a_path_above_the_project_or_into_the_content_root_or_output_is_refused() {
    let model = MODEL.replace(
        "guide = [\"templates/how-to.guide.md\", \"templates/notes.md\"]",
        "guide = [\"../notes.md\", \"docs/notes.md\", \".ascribe/build/notes.md\", \"templates/../docs/a.md\"]",
    );
    let mut files = complete();
    files.extend([
        ("docs/notes.md", "x\n"),
        (".ascribe/build/notes.md", "x\n"),
        ("docs/a.md", "x\n"),
    ]);
    assert_eq!(
        variants(&issues(&model, &files)),
        [
            ("model-type-file", Some("outside")),
            ("model-type-file", Some("inside-content")),
            ("model-type-file", Some("output")),
            ("model-type-file", Some("inside-content")),
        ]
    );
}

#[test]
fn a_content_root_above_the_project_holds_every_file_in_it() {
    let model = MODEL.replace(
        "spec = \"0.1\"\n",
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"..\"\n",
    );
    let found = issues(&model, &complete());
    assert_eq!(found.len(), 3);
    assert!(found.iter().all(|i| i.variant == Some("inside-content")));
}

#[test]
fn a_template_that_isnt_a_page_is_reported_with_its_line() {
    let mut files = complete();
    files[0].1 = "---\ntitle: \"Title\"\n---\n\n@steps:\n1. Do it.\n";
    let found = issues(MODEL, &files);
    assert_eq!(variants(&found), [("model-type-template", Some("parse"))]);
    assert_eq!(found[0].arg("line"), Some("5"));
}

#[test]
fn a_template_whose_frontmatter_isnt_yaml_is_reported() {
    let mut files = complete();
    files[0].1 = "---\ntitle: Install: the agent\n---\n\nText.\n";
    let found = issues(MODEL, &files);
    assert_eq!(variants(&found), [("model-type-template", Some("parse"))]);
    assert!(found[0].arg("detail").unwrap().contains("YAML"));
}

#[test]
fn a_warning_in_a_template_isnt_a_problem() {
    // `@warning:` is a directive-shaped line with an unknown name: a warning
    // on a page, so the template still reads.
    let mut files = complete();
    files[0].1 = "---\ntitle: \"Title\"\n---\n\n@warning: Back up first.\n";
    assert_eq!(issues(MODEL, &files), []);
}

#[test]
fn a_problem_fails_the_load_with_the_models_warnings() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("docs")).unwrap();
    let model = format!("{MODEL}\n[dimensions.a]\nvalues = [\"Cloud\", \"cloud\"]\n");
    let issues = load_model(&model, dir.path(), SourceBound::Anywhere).unwrap_err();
    let slugs: Vec<&str> = issues.iter().map(|i| i.slug.as_str()).collect();
    assert_eq!(
        slugs,
        [
            "model-name-case",
            "model-type-file",
            "model-type-file",
            "model-type-file"
        ]
    );
}

#[test]
fn a_link_out_of_the_project_or_into_the_content_root_is_followed() {
    let outside = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("docs")).unwrap();
    std::fs::create_dir_all(root.join("templates")).unwrap();
    std::fs::write(outside.path().join("how-to.md"), TEMPLATE).unwrap();
    std::fs::write(root.join("docs/guide.md"), "x\n").unwrap();
    std::fs::write(root.join("templates/notes.md"), "x\n").unwrap();
    if !crate::links::file(
        outside.path().join("how-to.md"),
        root.join("templates/how-to.md"),
    ) || !crate::links::file("../docs/guide.md", root.join("templates/how-to.guide.md"))
    {
        return;
    }
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).unwrap();
    let disk = DiskFs::new(root, &Layout::from_model(&model));
    assert_eq!(
        variants(&check(&model, &disk)),
        [
            ("model-type-file", Some("link")),
            ("model-type-file", Some("inside-content")),
        ]
    );
}
