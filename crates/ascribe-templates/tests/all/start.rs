//! `start` in a project in a temporary folder: a page of a type with a
//! template, of one without, where a page goes when no path is given, and
//! the files it never starts a page over.

#![allow(clippy::expect_used)]

use std::path::Path;

use ascribe_check::Project;
use ascribe_core::{Coded, RelPath};
use ascribe_resolve::{DiskFs, Layout};
use ascribe_templates::{Eject, NewPage, StartError, eject, set, start, takes_folder};

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.guide.frontmatter]\ntitle = \"string\"\naudience = \"string\"\nowner = \"string?\"\n";

/// A project with the type `guide`, which has no template, and the how-to,
/// as `ascribe templates eject how-to` gives it.
fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary folder");
    std::fs::write(dir.path().join("ascribe.toml"), MODEL).expect("written");
    std::fs::create_dir(dir.path().join("docs")).expect("made");
    let file = ascribe_check::LOADER
        .load_model(&dir.path().join("ascribe.toml"))
        .expect("the model loads");
    let fs = DiskFs::new(dir.path(), &Layout::from_model(&file.model));
    let names = ["how-to".to_owned()];
    let answer = eject(
        &Eject {
            root: dir.path(),
            model: &file.model,
            model_text: &file.text,
            names: &names,
            to: &RelPath::parse("templates").expect("a relative path"),
            ascribe_version: "9.9.9",
            keep_type: false,
        },
        &fs,
    )
    .expect("ejected");
    assert!(answer.refused.is_empty());
    dir
}

fn run(
    dir: &Path,
    type_name: &str,
    title: &str,
    path: Option<&str>,
) -> Result<NewPage, StartError> {
    let project = ascribe_check::LOADER
        .load(&dir.join("ascribe.toml"))
        .map(Project::from_loaded)
        .expect("the project loads");
    let path = path.map(|p| RelPath::parse(p).expect("a relative path"));
    start(&project, type_name, Some(title), path.as_ref())
}

#[test]
fn a_type_with_a_template_starts_from_it_with_the_title_set() {
    let dir = project();
    let page = run(dir.path(), "how-to", "Rotate keys", None).expect("started");
    // With no path, a page goes in the content root.
    assert_eq!(page.path.as_str(), "rotate-keys.md");
    let how_to = set()
        .templates
        .into_iter()
        .find(|t| t.name == "how-to")
        .expect("the set has the how-to");
    let template = how_to.text(&how_to.template);
    // The template names its type already.
    assert!(template.starts_with("---\ntype: how-to\n"), "{template}");
    // It starts as a draft.
    assert_eq!(
        page.text,
        template
            .replacen("title: \"Title\"", "title: \"Rotate keys\"", 1)
            .replacen("\n---\n", "\ndraft: true\n---\n", 1)
    );
    assert_eq!(page.template.as_deref(), Some("templates/how-to.md"));
    assert_eq!(
        page.guide,
        ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]
    );
}

#[test]
fn a_type_without_a_template_starts_with_its_frontmatter() {
    let dir = project();
    let page = run(dir.path(), "guide", "Keys: \"why\"", None).expect("started");
    assert_eq!(page.path.as_str(), "keys-why.md");
    assert_eq!(
        page.text,
        "---\ntype: guide\ntitle: \"Keys: \\\"why\\\"\"\naudience:\ndraft: true\n---\n"
    );
    assert_eq!(page.template, None);
    assert!(page.guide.is_empty());
}

#[test]
fn a_page_is_never_started_over_a_file() {
    let dir = project();
    std::fs::write(
        dir.path().join("docs/rotate-keys.md"),
        "---\ntitle: Mine\ndescription: Mine.\n---\n",
    )
    .expect("written");
    let error = run(dir.path(), "how-to", "Rotate Keys!", None).expect_err("refused");
    assert_eq!(
        error,
        StartError::FileInTheWay {
            path: "rotate-keys.md".to_owned()
        }
    );
    assert_eq!(error.code(), "page_exists");
    // Nothing was written over it.
    assert!(
        std::fs::read_to_string(dir.path().join("docs/rotate-keys.md"))
            .expect("read")
            .contains("Mine")
    );
}

#[test]
fn a_path_or_a_folder_given_is_where_the_page_goes_when_it_s_a_page_s() {
    let dir = project();
    let page = run(
        dir.path(),
        "how-to",
        "Rotate keys",
        Some("how-to/keys/rotate.md"),
    )
    .expect("started");
    assert_eq!(page.path.as_str(), "how-to/keys/rotate.md");
    assert!(
        page.text
            .starts_with("---\ntype: how-to\ntitle: \"Rotate keys\"\n")
    );
    // A page names its type, so any folder takes it.
    let page = run(
        dir.path(),
        "how-to",
        "Rotate keys",
        Some("guides/rotate.md"),
    )
    .expect("started");
    assert_eq!(page.path.as_str(), "guides/rotate.md");

    let refused =
        |path: &str| run(dir.path(), "how-to", "Rotate keys", Some(path)).expect_err("refused");
    assert_eq!(
        refused("how-to/_rotate.md"),
        StartError::NotTheType {
            path: "how-to/_rotate.md".to_owned(),
            type_name: "how-to".to_owned(),
            reason: "it's a fragment's path".to_owned()
        }
    );
    // A folder: the page is named for the title in it.
    let page = run(dir.path(), "how-to", "Rotate keys", Some("how-to/keys")).expect("started");
    assert_eq!(page.path.as_str(), "how-to/keys/rotate-keys.md");
    assert!(matches!(
        refused("../elsewhere"),
        StartError::NotTheType { .. }
    ));
}

#[test]
fn an_unknown_type_or_a_title_with_no_name_in_it_is_refused() {
    let dir = project();
    assert_eq!(
        run(dir.path(), "recipe", "X", None).expect_err("refused"),
        StartError::UnknownType {
            name: "recipe".to_owned(),
            types: "`guide`, `how-to`".to_owned()
        }
    );
    assert!(matches!(
        run(dir.path(), "how-to", "?!", None).expect_err("refused"),
        StartError::NoPath { .. }
    ));
}

#[test]
fn a_folder_takes_the_types_a_page_started_in_it_would_be_of() {
    let dir = project();
    let project = ascribe_check::LOADER
        .load(&dir.path().join("ascribe.toml"))
        .map(Project::from_loaded)
        .expect("the project loads");
    let model = project.model();
    let takes = |folder: &str| -> Vec<&str> {
        let folder = RelPath::parse(folder).expect("a relative path");
        model
            .types
            .iter()
            .filter(|t| takes_folder(model, t, &folder))
            .map(|t| t.name.as_str())
            .collect()
    };
    assert_eq!(takes("how-to"), ["guide", "how-to"]);
    assert_eq!(takes("how-to/keys"), ["guide", "how-to"]);
    assert_eq!(takes(""), ["guide", "how-to"]);
    assert!(takes("../elsewhere").is_empty());
}

#[test]
fn with_no_title_the_page_keeps_the_templates_and_needs_a_path() {
    let dir = project();
    let project = ascribe_check::LOADER
        .load(&dir.path().join("ascribe.toml"))
        .map(Project::from_loaded)
        .expect("the project loads");
    let path = RelPath::parse("rotate.md").expect("a relative path");
    let page = start(&project, "how-to", None, Some(&path)).expect("started");
    let how_to = set()
        .templates
        .into_iter()
        .find(|t| t.name == "how-to")
        .expect("the set has the how-to");
    assert_eq!(
        page.text,
        how_to
            .text(&how_to.template)
            .replacen("\n---\n", "\ndraft: true\n---\n", 1)
    );
    // A type with no template has an empty title.
    let page = start(&project, "guide", None, Some(&path)).expect("started");
    assert!(page.text.contains("\ntitle:\n"), "{}", page.text);
    // A folder's page is named for its title, so there's none to name.
    let folder = RelPath::parse("guides").expect("a relative path");
    let error = start(&project, "how-to", None, Some(&folder)).expect_err("refused");
    assert_eq!(error.code(), "page_path_needed");
}
