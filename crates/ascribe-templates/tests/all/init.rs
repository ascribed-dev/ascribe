//! `init` in temporary folders: the `ascribe.toml` and `ascribe.addresses`
//! it writes and the content root it makes, with the templates and
//! without, in a folder named by a path, and what it refuses, writing
//! nothing.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;

use ascribe_core::Coded;
use ascribe_templates::{Init, InitAnswer, InitError, MODEL_TEXT, init};

fn run(root: &Path, templates: bool) -> Result<InitAnswer, InitError> {
    init(&Init {
        root,
        templates,
        ascribe_version: "9.9.9",
    })
}

fn read(dir: &Path, path: &str) -> String {
    std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Every entry under `dir`, from it, in order: a folder ends in `/`.
fn entries(dir: &Path) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("a folder") {
            let path = entry.expect("an entry").path();
            let rel = path
                .strip_prefix(base)
                .expect("under base")
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                out.push(format!("{rel}/"));
                walk(&path, base, out);
            } else {
                out.push(rel);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn the_first_ascribe_toml_loads_with_nothing_to_say_and_a_page_type() {
    let model = ascribe_model::load_str(MODEL_TEXT, ascribe_core::FileId::new(0))
        .unwrap_or_else(|issues| panic!("{issues:#?}"));
    assert!(model.warnings.is_empty(), "{:#?}", model.warnings);
    assert_eq!(model.types.len(), 1);
    let page = &model.types[0];
    assert_eq!(page.name, "page");
    assert!(!page.implicit);
    let fields: Vec<&str> = page
        .frontmatter
        .fields
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(fields, ["title", "description"]);
    // It doesn't guess: no site address, builds, or dimensions.
    for absent in ["[consumer]", "[builds", "[dimensions", "[phrases]"] {
        assert!(
            !MODEL_TEXT.lines().any(|l| l.starts_with(absent)),
            "{absent}"
        );
    }
}

#[test]
fn an_empty_folder_gets_ascribe_toml_and_its_content_root() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let answer = run(dir.path(), false).expect("started");
    assert_eq!(answer.written, ["ascribe.toml", "ascribe.addresses"]);
    assert_eq!(answer.content_root, "docs");
    assert!(answer.content_root_made);
    assert_eq!(answer.templates, None);
    assert_eq!(answer.next, "https://ascribed-dev.com/go/astro-guide/");
    assert_eq!(
        entries(dir.path()),
        ["ascribe.addresses", "ascribe.toml", "docs/"]
    );
    assert_eq!(read(dir.path(), "ascribe.toml"), MODEL_TEXT);
    assert_eq!(
        read(dir.path(), "ascribe.addresses"),
        ascribe_model::AddressFile::default().to_toml()
    );
    let project =
        ascribe_check::Project::load(&dir.path().join("ascribe.toml")).expect("the project loads");
    assert!(
        project.addresses().is_some_and(Result::is_ok),
        "the addresses are kept"
    );
}

#[test]
fn with_the_templates_it_writes_what_eject_writes() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let answer = run(dir.path(), true).expect("started");
    let ejected = answer.templates.expect("the templates");
    assert!(ejected.refused.is_empty());
    assert!(!ejected.page_declared, "the file declares page itself");
    assert_eq!(
        ejected.added.len(),
        ascribe_templates::set().templates.len()
    );
    assert_eq!(
        answer.written[..answer.written.len() - 1],
        ejected.written[..]
    );
    assert_eq!(
        ejected.written.last().map(String::as_str),
        Some("ascribe.toml")
    );
    assert_eq!(
        answer.written.last().map(String::as_str),
        Some("ascribe.addresses")
    );
    assert!(answer.content_root_made);
    let model = read(dir.path(), "ascribe.toml");
    assert!(model.starts_with(MODEL_TEXT), "{model}");
    assert!(model.contains("\n[types.how-to]\n"), "{model}");
    assert!(dir.path().join("templates/templates.lock").is_file());
    assert!(dir.path().join("docs").is_dir());
}

#[test]
fn a_path_given_is_made_and_the_project_is_started_there() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let root = dir.path().join("handbook/docs-site");
    let answer = run(&root, false).expect("started");
    assert_eq!(answer.written, ["ascribe.toml", "ascribe.addresses"]);
    assert_eq!(
        entries(dir.path()),
        [
            "handbook/",
            "handbook/docs-site/",
            "handbook/docs-site/ascribe.addresses",
            "handbook/docs-site/ascribe.toml",
            "handbook/docs-site/docs/"
        ]
    );
}

/// A folder that has pages already gets `ascribe.addresses` too, so the
/// pages there are kept from the first commit.
#[test]
fn a_content_root_already_there_is_kept() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    std::fs::create_dir(dir.path().join("docs")).expect("made");
    std::fs::write(dir.path().join("docs/index.md"), "Hello.\n").expect("written");
    let answer = run(dir.path(), false).expect("started");
    assert!(!answer.content_root_made);
    assert_eq!(answer.written, ["ascribe.toml", "ascribe.addresses"]);
    assert_eq!(read(dir.path(), "docs/index.md"), "Hello.\n");
}

#[test]
fn an_ascribe_addresses_already_there_is_left_as_it_is() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    std::fs::write(dir.path().join("ascribe.addresses"), "Mine.\n").expect("written");
    let answer = run(dir.path(), false).expect("started");
    assert_eq!(answer.written, ["ascribe.toml"]);
    assert_eq!(read(dir.path(), "ascribe.addresses"), "Mine.\n");
}

#[test]
fn a_folder_inside_a_project_is_refused_with_nothing_written() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    std::fs::write(dir.path().join("ascribe.toml"), "spec = \"0.1\"\n").expect("written");
    let inner = dir.path().join("guides");
    std::fs::create_dir(&inner).expect("made");
    for templates in [false, true] {
        let error = run(&inner, templates).expect_err("refused");
        assert_eq!(error.code(), "project_exists");
        let message = error.to_string();
        assert!(message.contains("ascribe templates eject"), "{message}");
        match error {
            InitError::Project { found } => assert_eq!(found, dir.path().join("ascribe.toml")),
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(entries(dir.path()), ["ascribe.toml", "guides/"]);
}

#[test]
fn an_ascribe_toml_that_doesn_t_parse_is_a_project_too() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    std::fs::write(dir.path().join("ascribe.toml"), "this isn't [toml\n").expect("written");
    let error = run(dir.path(), false).expect_err("refused");
    assert_eq!(error.code(), "project_exists");
    assert_eq!(read(dir.path(), "ascribe.toml"), "this isn't [toml\n");
    assert_eq!(entries(dir.path()), ["ascribe.toml"]);
}

#[test]
fn a_file_named_as_the_folder_is_refused() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    std::fs::write(dir.path().join("notes"), "Hello.\n").expect("written");
    let error = run(&dir.path().join("notes"), false).expect_err("refused");
    assert_eq!(error.code(), "init_folder_refused");
    assert_eq!(entries(dir.path()), ["notes"]);
}

#[test]
fn a_template_in_the_way_is_refused_with_nothing_written() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    std::fs::create_dir(dir.path().join("templates")).expect("made");
    std::fs::write(dir.path().join("templates/how-to.md"), "Mine.\n").expect("written");
    let answer = run(dir.path(), true).expect("answered");
    let ejected = answer.templates.expect("the templates");
    assert_eq!(ejected.refused.len(), 1, "{:#?}", ejected.refused);
    assert!(answer.written.is_empty());
    assert!(!answer.content_root_made);
    assert_eq!(entries(dir.path()), ["templates/", "templates/how-to.md"]);
}
