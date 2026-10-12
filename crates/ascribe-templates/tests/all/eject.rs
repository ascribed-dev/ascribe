//! `eject` in a project in a temporary folder: what it writes, what it adds
//! to `ascribe.toml`, `templates.lock`, and what it refuses, writing nothing.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;

use ascribe_check::Project;
use ascribe_core::RelPath;
use ascribe_resolve::{DiskFs, Layout};
use ascribe_templates::{Eject, EjectAnswer, EjectError, RefusalReason, eject};

const MODEL: &str = "# The docs' content model.\nspec = \"0.1\"\n\n[project]\ncontent-root = \"docs\" # the pages\n\n[types.guide]\ndefault = true\n\n[types.guide.frontmatter]\ntitle = \"string\"\n";

fn project(model: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary folder");
    std::fs::write(dir.path().join("ascribe.toml"), model).expect("written");
    std::fs::create_dir(dir.path().join("docs")).expect("made");
    std::fs::write(
        dir.path().join("docs/index.md"),
        "---\ntitle: Home\n---\n\nHello.\n",
    )
    .expect("written");
    dir
}

fn run(dir: &Path, names: &[&str], to: &str) -> Result<EjectAnswer, EjectError> {
    let file = Project::load_model(&dir.join("ascribe.toml")).expect("the model loads");
    let names: Vec<String> = names.iter().map(|n| (*n).to_owned()).collect();
    let to = RelPath::parse(to).expect("a relative path");
    let fs = DiskFs::new(dir, &Layout::from_model(&file.model));
    eject(
        &Eject {
            root: dir,
            model: &file.model,
            model_text: &file.text,
            names: &names,
            to: &to,
            ascribe_version: "9.9.9",
        },
        &fs,
    )
}

fn read(dir: &Path, path: &str) -> String {
    std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Every file under `dir`, from it, in order.
fn files(dir: &Path) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("a folder") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                let rel = path.strip_prefix(base).expect("under base");
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn a_clean_project_gets_the_template_its_guide_ascribe_s_notes_the_lock_and_the_type() {
    let dir = project(MODEL);
    let answer = run(dir.path(), &["how-to"], "templates").expect("ejected");
    assert!(answer.refused.is_empty());
    assert!(!answer.page_declared);
    assert_eq!(
        answer.written,
        [
            "templates/how-to.md",
            "templates/how-to.guide.md",
            "templates/how-to.ascribe.md",
            "templates/templates.lock",
            "ascribe.toml"
        ]
    );
    let added = &answer.added[0];
    assert_eq!(added.name, "how-to");
    assert_eq!(added.folder, "docs/how-to");
    assert_eq!(added.files, "how-to/**");
    assert_eq!(
        added.guide,
        ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]
    );
    assert_eq!(
        read(dir.path(), "templates/how-to.ascribe.md"),
        ascribe_templates::set().templates[0]
            .notes()
            .expect("the how-to has notes")
            .text
    );
    assert_eq!(
        read(dir.path(), "templates/how-to.md"),
        ascribe_templates::set().templates[0].text("how-to.md")
    );
    // Added at the end, with everything before it, comments too, as it was.
    let model = read(dir.path(), "ascribe.toml");
    assert_eq!(
        model,
        format!(
            "{MODEL}\n[types.how-to]\nfiles = [\"how-to/**\"]\ntemplate = \"templates/how-to.md\"\nguide = [\"templates/how-to.guide.md\", \"templates/how-to.ascribe.md\"]\n\n[types.how-to.frontmatter]\ntitle = \"string\"\ndescription = \"string\"\n"
        )
    );
    // The project loads with its files checked, and a page copied from the
    // template is of the type.
    let file = Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
    let how_to = file.model.types.iter().find(|t| t.name == "how-to");
    assert!(how_to.is_some_and(|t| t.template.is_some()));
}

#[test]
fn the_lock_records_each_file_and_a_second_eject_adds_to_it() {
    let dir = project(MODEL);
    std::fs::create_dir(dir.path().join("templates")).expect("made");
    std::fs::write(
        dir.path().join("templates/templates.lock"),
        "# Kept.\nversion = 1\n\n[[template]]\nname = \"concept\"\nsummary = \"A concept.\"\n\n[template.files]\n\"templates/concept.md\" = \"sha256:00\"\n",
    )
    .expect("written");
    run(dir.path(), &["how-to"], "templates").expect("ejected");
    let lock = read(dir.path(), "templates/templates.lock");
    let parsed: toml::Table = toml::from_str(&lock).expect("the lock is TOML");
    let entries = parsed["template"].as_array().expect("a list");
    assert_eq!(entries.len(), 2, "{lock}");
    assert!(lock.starts_with("# Kept.\n"), "{lock}");
    let how_to = entries[1].as_table().expect("a table");
    let set = ascribe_templates::set();
    assert_eq!(how_to["name"].as_str(), Some("how-to"));
    assert_eq!(how_to["tag"].as_str(), Some(set.tag.as_str()));
    assert_eq!(how_to["ascribe"].as_str(), Some("9.9.9"));
    assert_eq!(
        how_to["summary"].as_str(),
        Some(set.templates[0].summary.as_str())
    );
    let template = read(dir.path(), "templates/how-to.md");
    assert_eq!(
        how_to["files"]["templates/how-to.md"].as_str(),
        Some(ascribe_model::file_hash(template.as_bytes()).as_str())
    );
    let mut theirs: Vec<&String> = how_to["files"]
        .as_table()
        .expect("a table")
        .keys()
        .collect();
    theirs.sort();
    assert_eq!(theirs, ["templates/how-to.guide.md", "templates/how-to.md"]);
    // Ascribe's notes are recorded as Ascribe's, apart from their files.
    let ours: Vec<&String> = how_to["ascribe-files"]
        .as_table()
        .expect("a table")
        .keys()
        .collect();
    assert_eq!(ours, ["templates/how-to.ascribe.md"]);
}

#[test]
fn a_new_lock_says_what_it_is() {
    let dir = project(MODEL);
    run(dir.path(), &[], "templates").expect("ejected");
    let lock = read(dir.path(), "templates/templates.lock");
    assert!(
        lock.starts_with("# Written by `ascribe templates eject`"),
        "{lock}"
    );
    assert!(lock.contains("\nversion = 1\n"), "{lock}");
}

#[test]
fn a_type_declared_already_is_refused_and_nothing_is_written() {
    let model = format!(
        "{MODEL}\n[types.how-to]\nfiles = [\"how-to/**\"]\n\n[types.how-to.frontmatter]\ntitle = \"string\"\n"
    );
    let dir = project(&model);
    let before = files(dir.path());
    let answer = run(dir.path(), &["how-to"], "templates").expect("answered");
    assert_eq!(answer.refused.len(), 1);
    assert_eq!(answer.refused[0].reason, RefusalReason::Declared);
    assert!(answer.written.is_empty() && answer.added.is_empty());
    assert_eq!(files(dir.path()), before);
    assert_eq!(read(dir.path(), "ascribe.toml"), model);
}

#[test]
fn a_file_in_the_way_is_refused_and_nothing_is_written() {
    let dir = project(MODEL);
    std::fs::create_dir(dir.path().join("templates")).expect("made");
    std::fs::write(dir.path().join("templates/how-to.guide.md"), "Ours.\n").expect("written");
    let before = files(dir.path());
    let answer = run(dir.path(), &["how-to"], "templates").expect("answered");
    assert_eq!(answer.refused.len(), 1);
    assert_eq!(answer.refused[0].reason, RefusalReason::FileInTheWay);
    assert_eq!(
        answer.refused[0].path.as_deref(),
        Some("templates/how-to.guide.md")
    );
    assert_eq!(files(dir.path()), before);
    assert_eq!(read(dir.path(), "templates/how-to.guide.md"), "Ours.\n");
    assert_eq!(read(dir.path(), "ascribe.toml"), MODEL);
}

#[test]
fn ascribe_s_notes_in_the_way_are_refused_too() {
    let dir = project(MODEL);
    std::fs::create_dir(dir.path().join("templates")).expect("made");
    std::fs::write(dir.path().join("templates/how-to.ascribe.md"), "Ours.\n").expect("written");
    let answer = run(dir.path(), &["how-to"], "templates").expect("answered");
    assert_eq!(answer.refused.len(), 1);
    assert_eq!(
        answer.refused[0].path.as_deref(),
        Some("templates/how-to.ascribe.md")
    );
    assert!(!dir.path().join("templates/how-to.md").exists());
}

#[test]
fn several_named_with_one_refused_write_nothing() {
    let dir = project(MODEL);
    let before = files(dir.path());
    let answer = run(
        dir.path(),
        &["how-to", "no-such-thing", "how-to"],
        "templates",
    )
    .expect("answered");
    assert_eq!(answer.refused.len(), 1);
    assert_eq!(answer.refused[0].name, "no-such-thing");
    assert_eq!(answer.refused[0].reason, RefusalReason::Unknown);
    assert!(answer.refused[0].message.contains("`how-to`"));
    assert!(answer.written.is_empty());
    assert_eq!(files(dir.path()), before);
}

#[test]
fn another_folder_is_written_into_and_named_from_the_project_s_folder() {
    let dir = project(MODEL);
    let answer = run(dir.path(), &["how-to"], "writing/starters").expect("ejected");
    assert_eq!(answer.written[0], "writing/starters/how-to.md");
    assert_eq!(answer.written[3], "writing/starters/templates.lock");
    assert!(
        read(dir.path(), "ascribe.toml").contains("template = \"writing/starters/how-to.md\"\n")
    );
    Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
}

#[test]
fn a_folder_no_type_can_name_a_file_in_is_refused() {
    let dir = project(MODEL);
    for to in ["docs/templates", "..", ".ascribe/build/templates"] {
        let error = run(dir.path(), &["how-to"], to).expect_err(to);
        assert!(matches!(error, EjectError::Folder { .. }), "{to}: {error}");
    }
    assert_eq!(read(dir.path(), "ascribe.toml"), MODEL);
    assert!(!dir.path().join("templates").exists());
}

#[test]
fn a_lock_eject_can_t_read_stops_it() {
    let dir = project(MODEL);
    std::fs::create_dir(dir.path().join("templates")).expect("made");
    std::fs::write(dir.path().join("templates/templates.lock"), "version = 2\n").expect("written");
    let error = run(dir.path(), &["how-to"], "templates").expect_err("refused");
    assert!(matches!(error, EjectError::Lock { .. }), "{error}");
    assert!(!dir.path().join("templates/how-to.md").exists());
}

#[test]
fn types_written_inline_can_t_be_added_to_and_nothing_is_written() {
    let model = "spec = \"0.1\"\ntypes = { guide = { default = true, frontmatter = { title = \"string\" } } }\n";
    let dir = project(model);
    let error = run(dir.path(), &["how-to"], "templates").expect_err("refused");
    assert!(matches!(error, EjectError::Model { .. }), "{error}");
    assert!(!dir.path().join("templates").exists());
    assert_eq!(read(dir.path(), "ascribe.toml"), model);
}

#[test]
fn a_project_with_no_types_keeps_the_page_type_its_pages_had() {
    let model = "spec = \"0.1\"\n";
    let dir = project(model);
    let answer = run(dir.path(), &["how-to"], "templates").expect("ejected");
    assert!(answer.page_declared);
    let text = read(dir.path(), "ascribe.toml");
    assert!(
        text.starts_with(
            "spec = \"0.1\"\n\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = \"string\"\n\n[types.how-to]\n"
        ),
        "{text}"
    );
    let file = Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
    let names: Vec<&str> = file.model.types.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["page", "how-to"]);
}
