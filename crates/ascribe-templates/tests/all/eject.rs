//! `eject` in a project in a temporary folder: what it writes, what it adds
//! to `ascribe.toml`, `templates.lock`, and what it refuses, writing nothing.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;

use ascribe_check::Project;
use ascribe_core::RelPath;
use ascribe_resolve::{DiskFs, Layout};
use ascribe_templates::{Eject, EjectAnswer, EjectError, RefusalReason, eject};

const MODEL: &str = "# The docs' content model.\nspec = \"0.1\"\n\n[project]\ncontent-root = \"docs\" # the pages\n\n[types.guide.frontmatter]\ntitle = \"string\"\n";

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

/// The home page `project` writes, naming its type, `guide`.
fn typed_home(dir: &Path) {
    std::fs::write(
        dir.join("docs/index.md"),
        "---\ntype: guide\ntitle: Home\n---\n\nHello.\n",
    )
    .expect("written");
}

fn run(dir: &Path, names: &[&str], to: &str) -> Result<EjectAnswer, EjectError> {
    run_with(dir, names, to, false)
}

fn keeping(dir: &Path, names: &[&str]) -> Result<EjectAnswer, EjectError> {
    run_with(dir, names, "templates", true)
}

fn run_with(
    dir: &Path,
    names: &[&str],
    to: &str,
    keep_type: bool,
) -> Result<EjectAnswer, EjectError> {
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
            keep_type,
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
    // Added at the end, with everything before it, comments too, as it was,
    // and with no `template` or `guide`: the lock gives them.
    let model = read(dir.path(), "ascribe.toml");
    assert_eq!(
        model,
        format!(
            "{MODEL}\n[types.how-to]\n\n[types.how-to.frontmatter]\ndescription = \"string\"\n"
        )
    );
    // The project loads with its files checked, and the type's template and
    // guide are the ones the lock records.
    let file = Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
    let how_to = file
        .model
        .types
        .iter()
        .find(|t| t.name == "how-to")
        .expect("the type");
    let template = how_to.template.as_ref().expect("a template");
    assert_eq!(template.path, "templates/how-to.md");
    assert_eq!(template.lock.as_deref(), Some("templates/templates.lock"));
    let guide: Vec<&str> = how_to.guide.iter().map(|g| g.path.as_str()).collect();
    assert_eq!(
        guide,
        ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]
    );
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
    let model = format!("{MODEL}\n[types.how-to.frontmatter]\ntitle = \"string\"\n");
    let dir = project(&model);
    let before = files(dir.path());
    let answer = run(dir.path(), &["how-to"], "templates").expect("answered");
    assert_eq!(answer.refused.len(), 1);
    assert_eq!(answer.refused[0].reason, RefusalReason::Declared);
    assert!(
        answer.refused[0]
            .message
            .ends_with("`ascribe templates eject how-to --keep-type` takes the template for it"),
        "{}",
        answer.refused[0].message
    );
    assert!(answer.written.is_empty() && answer.added.is_empty());
    assert_eq!(files(dir.path()), before);
    assert_eq!(read(dir.path(), "ascribe.toml"), model);
}

/// A model with `how-to` declared as `table`, after `[types.guide]`, and a
/// comment and a type after it.
fn declared(table: &str) -> String {
    format!(
        "{MODEL}\n# How-tos.\n{table}\n# Last.\n[types.concept.frontmatter]\ntitle = \"string\"\n"
    )
}

#[test]
fn a_type_kept_gets_its_template_and_guide_from_the_lock_and_ascribe_toml_is_untouched() {
    let model = declared(
        "[types.how-to] # How-tos\npurpose = \"Doing a task.\"\n\n[types.how-to.frontmatter]\ntitle = { type = \"string\", inline = \"code\" }\ndescription = { type = \"string\", role = \"description\" }\n",
    );
    let dir = project(&model);
    let answer = keeping(dir.path(), &["how-to"]).expect("ejected");
    assert!(answer.refused.is_empty() && answer.added.is_empty());
    assert_eq!(
        answer.written,
        [
            "templates/how-to.md",
            "templates/how-to.guide.md",
            "templates/how-to.ascribe.md",
            "templates/templates.lock",
        ]
    );
    let kept = &answer.kept[0];
    assert_eq!(kept.name, "how-to");
    assert_eq!(kept.template, "templates/how-to.md");
    assert_eq!(
        kept.guide,
        ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]
    );
    assert_eq!(kept.left, ["purpose", "frontmatter"]);
    assert_eq!(read(dir.path(), "ascribe.toml"), model);
    let lock = read(dir.path(), "templates/templates.lock");
    assert!(lock.contains("name = \"how-to\""), "{lock}");
    let file = Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
    let how_to = file.model.types.iter().find(|t| t.name == "how-to");
    assert!(how_to.is_some_and(|t| {
        t.template
            .as_ref()
            .is_some_and(|f| f.path == "templates/how-to.md")
    }));
}

#[test]
fn a_type_kept_in_another_folder_is_given_its_template_and_guide() {
    let model = declared(
        "[types.how-to] # How-tos\npurpose = \"Doing a task.\"\n\n[types.how-to.frontmatter]\ntitle = { type = \"string\", inline = \"code\" }\ndescription = { type = \"string\", role = \"description\" }\n",
    );
    let dir = project(&model);
    let answer = run_with(dir.path(), &["how-to"], "writing", true).expect("ejected");
    assert_eq!(
        answer.written.last().map(String::as_str),
        Some("ascribe.toml")
    );
    assert_eq!(
        read(dir.path(), "ascribe.toml"),
        model.replace(
            "purpose = \"Doing a task.\"\n",
            "purpose = \"Doing a task.\"\ntemplate = \"writing/how-to.md\"\nguide = [\"writing/how-to.guide.md\", \"writing/how-to.ascribe.md\"]\n"
        )
    );
}

#[test]
fn keeping_a_type_adds_the_ones_not_declared_as_eject_does() {
    let model = declared(
        "[types.how-to]\n\n[types.how-to.frontmatter]\ntitle = \"string\"\ndescription = \"string\"\n",
    );
    let dir = project(&model);
    let answer = keeping(dir.path(), &["tutorial", "how-to"]).expect("ejected");
    assert_eq!(answer.kept.len(), 1);
    assert_eq!(answer.added.len(), 1);
    assert_eq!(answer.added[0].name, "tutorial");
    let text = read(dir.path(), "ascribe.toml");
    assert!(
        text.ends_with("[types.tutorial.frontmatter]\ndescription = \"string\"\n"),
        "{text}"
    );
    assert!(text.starts_with(&model), "{text}");
    assert!(!text.contains("template = "), "{text}");
    // Written in the order named.
    assert_eq!(answer.written[0], "templates/tutorial.md");
}

#[test]
fn the_docs_reference_is_what_keeping_the_type_leaves_apart_from_its_purpose() {
    let docs = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/ascribe.toml"),
    )
    .expect("the docs' ascribe.toml");
    let start = docs
        .find("[types.reference]\n")
        .expect("the docs declare the reference");
    let end = docs[start..]
        .find("\n\n#")
        .map_or(docs.len(), |i| start + i + 1);
    let table: String = docs[start..end]
        .lines()
        .filter(|l| !l.starts_with("purpose = "))
        .map(|l| format!("{l}\n"))
        .collect();
    // The fields every type has, which the template's frontmatter needs.
    let shared_start = docs
        .find("[frontmatter]\n")
        .expect("the docs declare shared frontmatter");
    let shared_end = docs[shared_start..]
        .find("\n\n")
        .map_or(docs.len(), |i| shared_start + i + 1);
    let shared = &docs[shared_start..shared_end];
    let model = format!("{MODEL}\n{shared}\n{table}");
    let dir = project(&model);
    keeping(dir.path(), &["reference"]).expect("ejected");
    assert_eq!(read(dir.path(), "ascribe.toml"), model);
    let file = Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
    let reference = file.model.types.iter().find(|t| t.name == "reference");
    assert!(reference.is_some_and(|t| {
        t.template
            .as_ref()
            .is_some_and(|f| f.path == "templates/reference.md")
    }));
}

#[test]
fn a_type_kept_must_be_a_table_of_its_own() {
    for table in [
        "[types]\nhow-to = { purpose = \"Doing a task.\", frontmatter = { title = \"string\", description = \"string\" } }\n",
        "[types]\nhow-to.purpose = \"Doing a task.\"\nhow-to.frontmatter.title = \"string\"\nhow-to.frontmatter.description = \"string\"\n",
    ] {
        let model = declared(table);
        let dir = project(&model);
        let answer = keeping(dir.path(), &["how-to"]).expect("answered");
        assert_eq!(answer.refused.len(), 1, "{table}");
        assert_eq!(answer.refused[0].reason, RefusalReason::TypeNotTable);
        assert!(answer.written.is_empty() && answer.kept.is_empty());
        assert!(!dir.path().join("templates").exists());
        assert_eq!(read(dir.path(), "ascribe.toml"), model);
    }
}

#[test]
fn a_type_kept_with_a_template_or_guide_already_is_refused() {
    for key in [
        "template = \"ours/how-to.md\"",
        "guide = [\"ours/how-to.guide.md\"]",
    ] {
        let model = declared(&format!(
            "[types.how-to]\n{key}\n\n[types.how-to.frontmatter]\ntitle = \"string\"\ndescription = \"string\"\n"
        ));
        let dir = project(&model);
        std::fs::create_dir(dir.path().join("ours")).expect("made");
        std::fs::write(
            dir.path().join("ours/how-to.md"),
            "---\ntitle: T\n---\n\nText.\n",
        )
        .expect("written");
        std::fs::write(dir.path().join("ours/how-to.guide.md"), "Read me.\n").expect("written");
        let answer = keeping(dir.path(), &["how-to"]).expect("answered");
        assert_eq!(answer.refused.len(), 1, "{key}");
        assert_eq!(answer.refused[0].reason, RefusalReason::TypeHasTemplate);
        assert!(!dir.path().join("templates").exists());
        assert_eq!(read(dir.path(), "ascribe.toml"), model);
    }
}

#[test]
fn a_type_kept_whose_frontmatter_doesn_t_take_the_template_s_is_refused() {
    for (frontmatter, field) in [
        (
            "title = \"string\"\n",
            "`description`, which [types.how-to] doesn't declare",
        ),
        (
            "title = \"string\"\ndescription = \"list(string)\"\n",
            "declares `description` as list(string)",
        ),
    ] {
        let model = declared(&format!(
            "[types.how-to]\n\n[types.how-to.frontmatter]\n{frontmatter}"
        ));
        let dir = project(&model);
        let answer = keeping(dir.path(), &["how-to"]).expect("answered");
        assert_eq!(answer.refused.len(), 1, "{frontmatter}");
        assert_eq!(answer.refused[0].reason, RefusalReason::FrontmatterDiffers);
        assert!(
            answer.refused[0].message.contains(field),
            "{}",
            answer.refused[0].message
        );
        assert!(!dir.path().join("templates").exists());
        assert_eq!(read(dir.path(), "ascribe.toml"), model);
    }
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
    let model = "spec = \"0.1\"\ntypes = { guide = { frontmatter = { title = \"string\" } } }\n";
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
        text.starts_with("spec = \"0.1\"\n\n[types.page]\n\n[types.how-to]\n"),
        "{text}"
    );
    let file = Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
    let names: Vec<&str> = file.model.types.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["page", "how-to"]);
}

#[test]
fn an_empty_types_table_declares_no_type_and_keeps_the_page_type() {
    let model = "spec = \"0.1\"\n\n[types]\n";
    let dir = project(model);
    let answer = run(dir.path(), &["how-to"], "templates").expect("ejected");
    assert!(answer.page_declared);
    let file = Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
    let names: Vec<&str> = file.model.types.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["page", "how-to"]);
    // The page there names no type: it's listed, to be given `type: page`.
    assert_eq!(answer.untyped, ["docs/index.md"]);
}

#[test]
fn an_empty_inline_types_table_can_t_be_added_to_and_nothing_is_written() {
    let model = "spec = \"0.1\"\ntypes = {}\n";
    let dir = project(model);
    let error = run(dir.path(), &["how-to"], "templates").expect_err("refused");
    assert!(matches!(error, EjectError::Model { .. }), "{error}");
    assert!(!dir.path().join("templates").exists());
    assert_eq!(read(dir.path(), "ascribe.toml"), model);
}

/// A project with only its home page, which eject with no name gives every
/// type of the set, and which then checks clean: no errors, warnings, or
/// advice, the templates and guides being outside the content.
#[test]
fn a_project_given_the_whole_set_checks_clean() {
    let dir = project(MODEL);
    typed_home(dir.path());
    let answer = run(dir.path(), &[], "templates").expect("ejected");
    let names: Vec<&str> = answer.added.iter().map(|a| a.name.as_str()).collect();
    let set: Vec<String> = ascribe_templates::set()
        .templates
        .into_iter()
        .map(|t| t.name)
        .collect();
    assert_eq!(names, set);
    let project = Project::load(&dir.path().join("ascribe.toml")).expect("the project loads");
    let types: Vec<&str> = project
        .model()
        .types
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(types[0], "guide");
    assert_eq!(&types[1..], names.as_slice());
    let found: Vec<_> = ascribe_check::check_all_builds(&project)
        .into_iter()
        .map(|d| format!("{}: {}", d.slug, d.message))
        .collect();
    assert!(found.is_empty(), "{found:#?}");
}

/// For each type of the set, a page copied from its template whole: `ascribe
/// check` reports a placeholder left on every line the template opens one
/// with `\{`, and the title and the description, a section left empty only
/// at a heading the template follows with a memo, and nothing else. That a
/// page filled in reports nothing is `examples/templates`, which has one of
/// each, and `crates/ascribe-cli/tests/all/check.rs`.
#[test]
fn a_page_copied_from_each_template_reports_its_placeholders() {
    let model = format!("{MODEL}\n[checks]\npage-orphan = \"off\"\n");
    let dir = project(&model);
    typed_home(dir.path());
    run(dir.path(), &[], "templates").expect("ejected");
    for template in ascribe_templates::set().templates {
        let text = template.text(&template.template);
        let page = format!("docs/{}/page.md", template.name);
        std::fs::create_dir_all(dir.path().join("docs").join(&template.name)).expect("made");
        std::fs::write(dir.path().join(&page), text).expect("written");
        let project = Project::load(&dir.path().join("ascribe.toml")).expect("the project loads");
        let found = ascribe_check::check_all_builds(&project);
        std::fs::remove_file(dir.path().join(&page)).expect("removed");

        let others: Vec<_> = found
            .iter()
            .filter(|d| {
                !matches!(
                    d.slug.as_str(),
                    "template-placeholder-left" | "template-section-empty"
                )
            })
            .map(|d| format!("{}: {}", d.slug, d.message))
            .collect();
        assert!(others.is_empty(), "{}: {others:#?}", template.name);
        let lines = ascribe_core::LineIndex::new(text);
        let source: Vec<&str> = text.lines().collect();
        for d in found
            .iter()
            .filter(|d| d.slug.as_str() == "template-section-empty")
        {
            let at = lines
                .line_col(d.location.span.start())
                .and_then(|at| usize::try_from(at.line).ok())
                .unwrap_or_default();
            let heading = source.get(at).is_some_and(|l| l.starts_with('#'));
            let memo = source.get(at + 1).is_some_and(|l| l.starts_with("@memo"));
            assert!(
                heading && memo,
                "{}: a section reported empty at line {at}, from 0, that isn't a heading the template follows with a memo",
                template.name
            );
        }
        let mut reported: Vec<u32> = found
            .iter()
            .filter(|d| d.slug.as_str() == "template-placeholder-left")
            .filter_map(|d| lines.line_col(d.location.span.start()))
            .map(|at| at.line)
            .collect();
        reported.sort_unstable();
        reported.dedup();
        let mut expected: Vec<u32> = text
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                line.contains("\\{")
                    || line.starts_with("title: ")
                    || line.starts_with("description: ")
            })
            .filter_map(|(at, _)| u32::try_from(at).ok())
            .collect();
        expected.sort_unstable();
        assert_eq!(
            reported, expected,
            "{}: the lines reported, from 0, aren't those with a placeholder",
            template.name
        );
    }
}

/// A project with one type, `guide`, which declares its title and
/// description with more than their type.
const DECLARED: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.guide.frontmatter]\ntitle = { type = \"string\", inline = \"code\" }\ndescription = { type = \"string\", role = \"description\" }\n";

#[test]
fn a_type_added_takes_another_type_s_declarations_of_its_fields() {
    let dir = project(DECLARED);
    let answer = run(dir.path(), &["how-to"], "templates").expect("ejected");
    assert_eq!(answer.added[0].copied, ["title", "description"]);
    let model = read(dir.path(), "ascribe.toml");
    assert!(
        model.ends_with(
            "[types.how-to.frontmatter]\ntitle = { type = \"string\", inline = \"code\" }\ndescription = { type = \"string\", role = \"description\" }\n"
        ),
        "{model}"
    );
    let file = Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
    let types = &file.model.types;
    let field = |ty: &str, name: &str| {
        types
            .iter()
            .find(|t| t.name == ty)
            .and_then(|t| t.frontmatter.fields.iter().find(|f| f.name == name))
            .cloned()
    };
    for name in ["title", "description"] {
        assert_eq!(field("how-to", name), field("guide", name), "{name}");
    }
}

#[test]
fn a_declaration_under_a_header_of_its_own_is_copied_inline() {
    let model = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.guide]\n\n[types.guide.frontmatter.title]\ntype = \"string\" # the page's\ninline = \"code\"\n";
    let dir = project(model);
    let answer = run(dir.path(), &["how-to"], "templates").expect("ejected");
    assert_eq!(answer.added[0].copied, ["title"]);
    let text = read(dir.path(), "ascribe.toml");
    assert!(
        text.ends_with(
            "[types.how-to.frontmatter]\ntitle = { type = \"string\", inline = \"code\" }\ndescription = \"string\"\n"
        ),
        "{text}"
    );
}

#[test]
fn a_field_another_type_declares_as_something_else_is_a_plain_string() {
    let model = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.guide.frontmatter]\ntitle = \"string\"\ndescription = \"date\"\n";
    let dir = project(model);
    let answer = run(dir.path(), &["how-to"], "templates").expect("ejected");
    assert!(answer.added[0].copied.is_empty());
    let text = read(dir.path(), "ascribe.toml");
    // `title`, a plain string, is every type's without a line of its own.
    assert!(
        text.ends_with("[types.how-to.frontmatter]\ndescription = \"string\"\n"),
        "{text}"
    );
}

#[test]
fn the_pages_that_name_no_type_are_listed() {
    let dir = project(DECLARED);
    std::fs::create_dir_all(dir.path().join("docs/guides/_parts")).expect("made");
    for (page, text) in [
        ("guides/b.md", "---\ntitle: B\n---\n"),
        ("guides/a.md", "No frontmatter.\n"),
        ("guides/typed.md", "---\ntype: guide\ntitle: T\n---\n"),
        ("guides/_parts/note.md", "---\ntitle: A\n---\n"),
    ] {
        std::fs::write(dir.path().join("docs").join(page), text).expect("written");
    }
    let answer = run(dir.path(), &["how-to", "concept"], "templates").expect("ejected");
    // In order, from the folder ascribe.toml is in, and no fragment.
    assert_eq!(
        answer.untyped,
        ["docs/guides/a.md", "docs/guides/b.md", "docs/index.md"]
    );
}

#[test]
fn a_type_added_leaves_out_the_fields_every_type_has() {
    let model = "spec = \"0.1\"\n\n[frontmatter]\ntitle = { type = \"string\", inline = \"code\" }\ndescription = \"string\"\n";
    let dir = project(model);
    run(dir.path(), &["how-to"], "templates").expect("ejected");
    let text = read(dir.path(), "ascribe.toml");
    assert!(
        text.ends_with("\n[types.page]\n\n[types.how-to]\n"),
        "{text}"
    );
}
