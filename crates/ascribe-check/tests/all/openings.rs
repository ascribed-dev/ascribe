//! How a page opens and whether a how-to has steps: `page-opening-missing`,
//! a file-level check of a page's first block, and `how-to-steps-missing`,
//! a page-level check of a `how-to` page once includes are expanded. The
//! conformance cases in `tests/conformance/cases/content/` hold the cases a
//! reader meets; these hold the edges of what counts.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use ascribe_check::{Project, check_all_builds, check_files};
use ascribe_core::{FileId, RelPath};

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"

[fragments]
patterns = ["_*"]

[types.how-to.frontmatter]
title = "string"

[types.concept.frontmatter]
title = "string"

[builds.site]

[checks]
page-orphan = "off"
fragment-unused = "off"
page-opening-missing = "advice"
"#;

fn project_with(model: &str, files: &[(&str, &str)]) -> Project {
    let sources: Vec<(RelPath, String)> = files
        .iter()
        .map(|(path, text)| {
            (
                RelPath::parse(path).expect("a relative path"),
                (*text).to_owned(),
            )
        })
        .collect();
    Project::new(
        ascribe_check::in_memory(
            PathBuf::from("/nonexistent-ascribe-project"),
            RelPath::parse("docs").expect("a relative path"),
            ascribe_model::load_str(model, FileId::new(0)).expect("the model loads"),
            model.to_owned(),
            sources,
        )
        .snapshot(),
    )
}

fn project(files: &[(&str, &str)]) -> Project {
    project_with(MODEL, files)
}

fn page(ty: &str, body: &str) -> String {
    format!("---\ntype: {ty}\ntitle: T\n---\n\n{body}\n")
}

/// The messages of every diagnostic with `slug`, at file and page level.
fn messages(project: &Project, slug: &str) -> Vec<String> {
    check_all_builds(project)
        .into_iter()
        .filter(|d| d.slug.to_string() == slug)
        .map(|d| d.message)
        .collect()
}

fn opening(body: &str) -> Vec<String> {
    messages(
        &project(&[("a.md", &page("concept", body))]),
        "page-opening-missing",
    )
}

fn steps(body: &str) -> bool {
    !messages(
        &project(&[
            ("a.md", &page("how-to", body)),
            ("_s.md", "@steps\n1. Go.\n"),
        ]),
        "how-to-steps-missing",
    )
    .is_empty()
}

#[test]
fn the_message_names_what_the_page_opens_with() {
    assert_eq!(
        opening("@note: Read this first.\n\nText."),
        [
            "this page opens with an `@note`, not a paragraph; open the page with a sentence that says what it covers"
        ]
    );
    assert!(opening("> Quoted.")[0].contains("a block quote"));
    assert!(opening("---\n\nText.")[0].contains("a thematic break"));
}

#[test]
fn a_bound_directive_is_passed_over_and_what_it_binds_to_is_the_opening() {
    assert!(opening("@summary\nQuill keeps notes in sync.").is_empty());
    assert!(opening("@available: ascribe planned\nNew text.").is_empty());
    assert!(opening("@intended {check=page-size}\n- One\n- Two")[0].contains("a list"));
}

#[test]
fn only_one_level_1_heading_stands_for_the_title() {
    assert!(opening("# T\n\nText.").is_empty());
    assert!(opening("# T\n\n# Again\n\nText.")[0].contains("a heading"));
}

#[test]
fn a_page_of_any_type_with_nothing_but_frontmatter_has_no_opening_to_check() {
    assert!(opening("").is_empty());
}

#[test]
fn the_opening_check_is_off_unless_named() {
    let model = MODEL.replace("page-opening-missing = \"advice\"\n", "");
    let p = project_with(&model, &[("a.md", &page("concept", "## Install"))]);
    assert!(check_files(&p).is_empty(), "{:#?}", check_files(&p));
}

#[test]
fn a_heading_counts_as_a_step_only_when_numbered_as_one() {
    assert!(!steps("Text.\n\n## 1. Install\n\nText."));
    assert!(!steps("Text.\n\n## 2) Install\n\nText."));
    assert!(!steps("Text.\n\n## Step 3 Install\n\nText."));
    assert!(!steps("Text.\n\n## STEP 3: Install\n\nText."));
    assert!(steps("Text.\n\n## 1.5 features\n\nText."));
    assert!(steps("Text.\n\n## Steps for Linux\n\nText."));
    assert!(steps("Text.\n\n## 2026 release\n\nText."));
}

#[test]
fn steps_an_include_brings_in_count() {
    assert!(!steps("Text.\n\n@include: _s.md"));
    assert!(steps("Text.\n\n1. An ordered list without `@steps`."));
}

#[test]
fn a_page_of_another_type_needs_no_steps() {
    let p = project(&[("a.md", &page("concept", "Text."))]);
    assert!(messages(&p, "how-to-steps-missing").is_empty());
}
