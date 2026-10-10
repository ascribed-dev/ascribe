//! The content checks across the project, on small projects built in
//! memory: what nothing uses (`fragment-unused`, `phrase-unused`,
//! `feature-unused`, `glossary-term-unused`, `image-unused`), what's too big
//! (`image-large`), and, per build, what nothing links to (`page-orphan`) and
//! what shares a title (`title-duplicate`). The conformance cases in
//! `tests/conformance/cases/checks/` hold each check's rule; these hold the
//! edges: builds that disagree, levels, and limits.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::sync::Arc;

use ascribe_check::{Diagnostic, Project, Severity, check_all_builds, diagnose};
use ascribe_core::{FileId, RelPath};
use ascribe_resolve::{Layout, MemoryFs};

/// Two builds: `site` keeps everything, `cloud` publishes only what's
/// available on `cloud`.
const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"

[types.page]
default = true

[types.page.frontmatter]
title = "string"
description = "string?"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud", "self-managed"]

[features.sso]
name = "Single sign-on"
available = "cloud"

[features.audit]
name = "Audit log"
available = "self-managed"

[phrases]
product = "Quill"
unused = "Nothing"

[glossary.terms.agent]
term = "agent"
definition = "The process that ships logs."

[glossary.terms.relay]
term = "relay"
definition = "Nothing writes this."

[glossary.terms.build]
term = "build"
definition = "Linked only where an author links it."
match = "marked"

[builds.site]
variants = "switch"
availability = "badge"

[builds.cloud]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }
"#;

fn project_with(text: &str, files: &[(&str, &str)], images: &[(&str, usize)]) -> Project {
    let model = ascribe_model::load_str(text, FileId::new(0)).expect("the model loads");
    let mut fs = MemoryFs::new(&Layout::from_model(&model));
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    for (path, size) in images {
        fs = fs.with_file(path, &"x".repeat(*size));
    }
    let sources = Project::from_sources(files.iter().map(|(path, text)| {
        (
            RelPath::parse(path).expect("a relative path"),
            (*text).to_owned(),
        )
    }));
    Project::from_parts_with_fs(
        PathBuf::from("/nonexistent-ascribe-project"),
        RelPath::parse("docs").expect("a relative path"),
        model,
        text.to_owned(),
        sources,
        Arc::new(fs),
    )
}

fn project(files: &[(&str, &str)]) -> Project {
    project_with(MODEL, files, &[])
}

/// Each diagnostic of one check: where it is, and the builds it names.
fn found(project: &Project, diagnostics: &[Diagnostic], slug: &str) -> Vec<(String, Vec<String>)> {
    diagnostics
        .iter()
        .filter(|d| d.slug.as_str() == slug)
        .map(|d| {
            let at = project.display_path(d.location.file).unwrap_or_default();
            let text = project
                .file(d.location.file)
                .and_then(|f| f.text.get(d.location.span.range()).map(str::to_owned))
                .unwrap_or_default();
            (format!("{at}: {text}"), d.builds.clone())
        })
        .collect()
}

const HOME: &str = "---\ntitle: Home\n---\n\n[Install](install.md)\n\n@available: self-managed\n[Upgrade](upgrade.md)\n";
const INSTALL: &str = "---\ntitle: Install\n---\n\nUse {product}.\n\n@include: _steps.md\n";
const UPGRADE: &str = "---\ntitle: Upgrade\n---\n\nText.\n";
const LONELY: &str = "---\ntitle: Lonely\n---\n\nText.\n";

#[test]
fn an_orphan_is_reported_once_naming_the_builds_it_is_an_orphan_in() {
    let p = project(&[
        ("index.md", HOME),
        ("install.md", INSTALL),
        ("upgrade.md", UPGRADE),
        ("lonely.md", LONELY),
    ]);
    let all = check_all_builds(&p);
    let orphans = found(&p, &all, "page-orphan");
    // `upgrade.md` is linked only from content `cloud` leaves out; nothing
    // links to `lonely.md`; `index.md` is an index page.
    assert_eq!(
        orphans,
        [
            (
                "docs/lonely.md: Lonely".to_owned(),
                vec!["site".to_owned(), "cloud".to_owned()]
            ),
            (
                "docs/upgrade.md: Upgrade".to_owned(),
                vec!["cloud".to_owned()]
            ),
        ]
    );
    // `ascribe check` says which builds when it isn't all of them.
    let report = diagnose(&p, &[]).expect("every build");
    let upgrade = report
        .diagnostics
        .iter()
        .find(|d| d.slug.as_str() == "page-orphan" && d.message.contains("only in build"))
        .expect("the orphan in one build");
    assert!(
        upgrade.message.ends_with("(only in build `cloud`)"),
        "{}",
        upgrade.message
    );
    assert_eq!(upgrade.severity, Severity::Advice);
}

#[test]
fn a_page_another_page_includes_is_not_an_orphan() {
    let p = project(&[
        ("index.md", "---\ntitle: Home\n---\n\n@include: shared.md\n"),
        ("shared.md", "---\ntitle: Shared\n---\n\nText.\n"),
    ]);
    assert!(found(&p, &check_all_builds(&p), "page-orphan").is_empty());
}

#[test]
fn a_builds_only_page_is_not_an_orphan() {
    let p = project(&[("install.md", "---\ntitle: Install\n---\n\nText.\n")]);
    assert!(found(&p, &check_all_builds(&p), "page-orphan").is_empty());
}

#[test]
fn titles_are_compared_per_build_ignoring_case() {
    let p = project(&[
        (
            "index.md",
            "---\ntitle: Home\n---\n\n[a](a.md) [b](b.md) [c](c.md)\n",
        ),
        ("a.md", "---\ntitle: Install\n---\n\nText.\n"),
        (
            "b.md",
            "---\ntitle: install\navailable: self-managed\n---\n\nText.\n",
        ),
        ("c.md", "---\ntitle: Configure\n---\n\nText.\n"),
    ]);
    let all = check_all_builds(&p);
    // `cloud` drops `b.md`, so only `site` has two pages titled Install.
    assert_eq!(
        found(&p, &all, "title-duplicate"),
        [
            ("docs/a.md: Install".to_owned(), vec!["site".to_owned()]),
            ("docs/b.md: install".to_owned(), vec!["site".to_owned()]),
        ]
    );
    let a = all
        .iter()
        .find(|d| d.slug.as_str() == "title-duplicate")
        .expect("a duplicate");
    assert!(a.message.contains("`b.md`"), "{}", a.message);
    assert_eq!(a.related.len(), 1);
}

#[test]
fn unused_fragments_phrases_features_and_terms_are_reported_where_they_are_declared() {
    let p = project(&[
        ("index.md", HOME),
        ("install.md", INSTALL),
        (
            "upgrade.md",
            "---\ntitle: Upgrade\navailable: sso\n---\n\nThe agent upgrades.\n",
        ),
        ("_steps.md", "Steps.\n"),
        ("_unused.md", "Nobody includes this.\n"),
    ]);
    let all = check_all_builds(&p);
    assert_eq!(
        found(&p, &all, "fragment-unused"),
        [("docs/_unused.md: Nobody includes this.".to_owned(), vec![])]
    );
    assert_eq!(
        found(&p, &all, "phrase-unused"),
        [("ascribe.toml: unused".to_owned(), vec![])]
    );
    assert_eq!(
        found(&p, &all, "feature-unused"),
        [("ascribe.toml: audit".to_owned(), vec![])]
    );
    // `build` is marked, so its uses are links, and it isn't reported.
    assert_eq!(
        found(&p, &all, "glossary-term-unused"),
        [("ascribe.toml: relay".to_owned(), vec![])]
    );
}

/// The counts the unused checks go by are the editor's inventory's: each
/// reported entry is one the search counts no use of, and each entry it
/// counts a use of isn't reported.
#[test]
fn what_is_unused_is_what_the_search_counts_no_use_of() {
    let p = project(&[
        ("index.md", HOME),
        ("install.md", INSTALL),
        ("upgrade.md", UPGRADE),
        ("_steps.md", "The agent.\n"),
        ("_unused.md", "Text.\n"),
    ]);
    let counts = p.held_index().use_counts();
    let all = check_all_builds(&p);
    let reported = |slug: &str| -> Vec<String> {
        all.iter()
            .filter(|d| d.slug.as_str() == slug)
            .map(|d| {
                let file = p.file(d.location.file).expect("a file");
                file.text
                    .get(d.location.span.range())
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect()
    };
    let model = p.model();
    let unused = |used: ascribe_resolve::Usable| !counts.contains_key(&used);
    let phrases: Vec<String> = model
        .phrases
        .iter()
        .filter(|ph| unused(ascribe_resolve::Usable::Phrase(ph.key.clone())))
        .map(|ph| ph.key.clone())
        .collect();
    assert_eq!(reported("phrase-unused"), phrases);
    let features: Vec<String> = model
        .features
        .iter()
        .filter(|f| unused(ascribe_resolve::Usable::Feature(f.key.clone())))
        .map(|f| f.key.clone())
        .collect();
    assert_eq!(reported("feature-unused"), features);
    let terms: Vec<String> = model
        .glossary
        .terms
        .iter()
        .filter(|t| t.match_mode != ascribe_model::GlossaryMatch::Marked)
        .filter(|t| unused(ascribe_resolve::Usable::Term(t.id.clone())))
        .map(|t| t.id.clone())
        .collect();
    assert_eq!(reported("glossary-term-unused"), terms);
}

#[test]
fn images_no_file_shows_and_images_over_the_limit_are_reported() {
    let files = [(
        "index.md",
        "---\ntitle: Home\n---\n\n![Map](img/map.png)\n\n[Download the diagram](img/diagram.svg)\n",
    )];
    let images = [
        ("docs/img/map.png", 600_000),
        ("docs/img/diagram.svg", 10),
        ("docs/img/old.PNG", 10),
        ("docs/notes.txt", 10),
        ("docs/.cache/thumb.png", 10),
    ];
    let p = project_with(MODEL, &files, &images);
    let all = check_all_builds(&p);
    let messages = |slug: &str| -> Vec<String> {
        all.iter()
            .filter(|d| d.slug.as_str() == slug)
            .map(|d| d.message.clone())
            .collect()
    };
    assert_eq!(
        messages("image-unused"),
        ["no file shows or links to `docs/img/old.PNG`"]
    );
    assert_eq!(
        messages("image-large"),
        ["`docs/img/map.png` is 600 KB, over the 500 KB limit for an image"]
    );
    // The image is a file of its own in the report.
    let large = all
        .iter()
        .find(|d| d.slug.as_str() == "image-large")
        .expect("a large image");
    assert_eq!(
        p.display_path(large.location.file).as_deref(),
        Some("docs/img/map.png")
    );

    // A project sets its own limit, and can turn a check off.
    let model = format!(
        "{MODEL}\n[checks]\nimage-unused = \"off\"\nimage-large = {{ limit = \"1 MB\", level = \"warning\" }}\n"
    );
    let p = project_with(&model, &files, &images);
    let all = check_all_builds(&p);
    assert!(all.iter().all(|d| d.slug.as_str() != "image-unused"));
    assert!(all.iter().all(|d| d.slug.as_str() != "image-large"));
    let model =
        format!("{MODEL}\n[checks]\nimage-large = {{ limit = 1000, level = \"warning\" }}\n");
    let p = project_with(&model, &files, &images);
    let report = diagnose(&p, &[]).expect("every build");
    let large = report
        .diagnostics
        .iter()
        .find(|d| d.slug.as_str() == "image-large")
        .expect("a large image");
    assert_eq!(large.severity, Severity::Warning);
}

#[test]
fn a_check_turned_off_is_not_run() {
    let model = format!(
        "{MODEL}\n[checks]\npage-orphan = \"off\"\nphrase-unused = \"off\"\ntitle-duplicate = \"error\"\n"
    );
    let p = project_with(
        &model,
        &[
            ("index.md", "---\ntitle: Home\n---\n\n[a](a.md) [b](b.md)\n"),
            ("a.md", "---\ntitle: Same\n---\n\nText.\n"),
            ("b.md", "---\ntitle: Same\n---\n\nText.\n"),
            ("lonely.md", LONELY),
        ],
        &[],
    );
    let report = diagnose(&p, &[]).expect("every build");
    let slugs: Vec<&str> = report.diagnostics.iter().map(|d| d.slug.as_str()).collect();
    assert!(!slugs.contains(&"page-orphan"), "{slugs:?}");
    assert!(!slugs.contains(&"phrase-unused"), "{slugs:?}");
    let duplicate = report
        .diagnostics
        .iter()
        .find(|d| d.slug.as_str() == "title-duplicate")
        .expect("a duplicate title");
    assert_eq!(duplicate.severity, Severity::Error);
}
