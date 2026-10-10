//! The prose checked by a real Vale: the one named by `ASCRIBE_TEST_VALE`,
//! which CI installs on Linux. Elsewhere, without it, these tests pass
//! without running; `crates/ascribe-check/src/prose/` tests the rest with a
//! stand-in.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use ascribe_check::prose::{MIN_VERSION, PROJECT_TIMEOUT, Program, lint};
use ascribe_check::{Diagnostic, MODEL_FILE, Project, Severity};
use ascribe_core::diagnostics;

/// The Vale to run, or `None` to skip. CI on Linux must have one.
fn vale() -> Option<PathBuf> {
    match std::env::var_os("ASCRIBE_TEST_VALE") {
        Some(path) => Some(PathBuf::from(path)),
        None if std::env::var_os("CI").is_some() && cfg!(target_os = "linux") => {
            panic!("CI on Linux sets ASCRIBE_TEST_VALE to a Vale for these tests")
        }
        None => None,
    }
}

fn project(model: &str, files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    fs::write(dir.path().join(MODEL_FILE), model).expect("write the model");
    fs::create_dir_all(dir.path().join("docs")).expect("create docs");
    for (path, text) in files {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().expect("a folder")).expect("create its folder");
        fs::write(path, text).expect("write a file");
    }
    dir
}

fn checked(dir: &Path) -> (Project, Vec<Diagnostic>) {
    let project = Project::load(&dir.join(MODEL_FILE)).expect("the project loads");
    let found = lint(&project, None, &Program, PROJECT_TIMEOUT);
    (project, found)
}

/// Each diagnostic as `file: text at it: message`.
fn shown(project: &Project, found: &[Diagnostic]) -> Vec<String> {
    found
        .iter()
        .map(|d| {
            let entry = project.file(d.location.file).expect("a file");
            format!(
                "{}: {}: {}",
                entry.display_path,
                &entry.text[d.location.span.range()],
                d.message
            )
        })
        .collect()
}

#[test]
fn the_quiet_preset_finds_prose_problems_and_nothing_else() {
    let Some(vale) = vale() else { return };
    let model = format!(
        "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[phrases]\nproduct = \"Acme the the Cloud\"\n\
         [checks.vale]\npreset = \"quiet\"\ncommand = '{}'\n",
        vale.display()
    );
    let page = "---\ntitle: the the title\n---\n\n# Using {product}\n\nIt is is ready.\n\n@note {type=the}: Back up up first.\n\n```sh\necho the the\n```\n";
    let dir = project(&model, &[("docs/page.md", page)]);
    let (project, found) = checked(dir.path());
    // A phrase's value is in the project's vocabulary, so Vale accepts it.
    assert_eq!(
        shown(&project, &found),
        [
            "docs/page.md: is is: Ascribe.Repeated: 'is' is repeated.",
            "docs/page.md: up up: Ascribe.Repeated: 'up' is repeated.",
        ]
    );
    assert!(found.iter().all(|d| d.severity == Severity::Advice));
    let fix = &found[0].fixes[0];
    assert_eq!(fix.edits[0].new_text, "is");
    assert!(
        dir.path().join(".ascribe/vale/quiet/.vale.ini").is_file(),
        "the preset is written under .ascribe/"
    );
}

#[test]
fn a_project_s_own_config_gets_the_project_s_words() {
    let Some(vale) = vale() else { return };
    let model = format!(
        "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[phrases]\nproduct = \"Zorbtastic\"\n\
         [checks.vale]\nconfig = \".vale.ini\"\ncommand = '{}'\n",
        vale.display()
    );
    let ini = "MinAlertLevel = suggestion\n\n[*.md]\nBasedOnStyles = Vale\n";
    let page = "Zorbtastic is the product, and Quuxbar isn't a word.\n";
    let dir = project(&model, &[(".vale.ini", ini), ("docs/page.md", page)]);
    let (project, found) = checked(dir.path());
    let shown = shown(&project, &found);
    assert!(
        shown
            .iter()
            .any(|s| s.starts_with("docs/page.md: Quuxbar: Vale.Spelling")),
        "{shown:?}"
    );
    assert!(!shown.iter().any(|s| s.contains("Zorbtastic")), "{shown:?}");
}

#[test]
fn a_broken_config_is_one_advice() {
    let Some(vale) = vale() else { return };
    let model = format!(
        "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[checks.vale]\nconfig = \"missing.ini\"\ncommand = '{}'\n",
        vale.display()
    );
    let dir = project(&model, &[("docs/page.md", "Some text.\n")]);
    let (_, found) = checked(dir.path());
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].slug, diagnostics::PROSE_NOT_CHECKED);
    assert!(found[0].message.contains("failed"), "{}", found[0].message);
}

#[test]
fn the_docs_name_the_oldest_vale_that_works() {
    let (major, minor, _) = MIN_VERSION;
    let wanted = format!("Vale](https://vale.sh) {major}.{minor} or later");
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry =
        fs::read_to_string(repo.join("tests/conformance/diagnostics.toml")).expect("the registry");
    assert!(
        registry.contains(&wanted),
        "the prose-not-checked fix says {wanted}"
    );
    let guide = fs::read_to_string(repo.join("docs/content/guides/vale.md")).expect("the guide");
    let wanted = format!("Install Vale {major}.{minor} or later");
    assert!(guide.contains(&wanted), "the Vale guide says {wanted}");
}
