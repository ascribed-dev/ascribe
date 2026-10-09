//! Where pages, headings, and the content model's entries are used.

#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use ascribe_core::FileId;
use ascribe_resolve::{Layout, MemoryFs, Project, Usable, UseKind};

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"
output-dir = ".ascribe/build"

[dimensions.pm]
values = ["npm", "pnpm"]

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[features.sso]
name = "Single sign-on"
available = "cloud"

[notes.security]
label = "Security"

[phrases]
product = "Quill"

[glossary.terms.api-key]
term = "API key"
aliases = ["API keys"]
definition = "A secret."
link = "/reference.md#api-key"

[glossary.terms.agent]
term = "agent"
definition = "The process that runs builds."
link = "/reference.md#agent"
match = "marked"

[widgets.quill-lab]
forms = ["line"]
primary = "none"
binding = "self"
"#;

fn project(files: &[(&str, &str)]) -> Project {
    let model = match ascribe_model::load_str(MODEL, FileId::new(0)) {
        Ok(model) => Arc::new(model),
        Err(issues) => panic!("the test model doesn't load: {issues:?}"),
    };
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (name, text) in files {
        fs = fs.with_file(name, text);
    }
    Project::load(model, layout, &fs)
}

fn path(text: &str) -> ascribe_core::RelPath {
    ascribe_core::RelPath::parse(text).expect("a valid path")
}

/// Each use of `target` as its file, its text, and its kind.
fn uses(project: &Project, target: &Usable) -> Vec<(String, String, UseKind)> {
    project
        .uses(target)
        .into_iter()
        .map(|u| {
            let file = project.file(&u.file).expect("a source file");
            (
                u.file.to_string(),
                file.source[u.span.range()].to_owned(),
                u.kind,
            )
        })
        .collect()
}

fn quill() -> Project {
    project(&[
        (
            "docs/index.md",
            "---\ntitle: Home\n---\n\nSee [setup](guide.md#setup), [the guide](guide.md), and [keys](reference.md#api-key).\n\nCreate an API key, then more API keys.\n",
        ),
        (
            "docs/guide.md",
            "---\ntitle: Guide\navailable: sso\n---\n\n# Guide\n\n@include: _setup.md\n\nRead [about {product}](#install).\n\n@note {type=security}: Keep your api key safe.\n\n@note: A plain note.\n\n@quill-lab\n",
        ),
        (
            "docs/_setup.md",
            "## Setup\n\n### Install\n\n@variant {pm=npm}:\nRun `npm i`.\n@end\n@variant {pm=pnpm}:\nRun `pnpm i`.\n@end\n\n@available: cloud\nCloud only.\n",
        ),
        (
            "docs/reference.md",
            "---\ntitle: Reference\nvariant:\n  deployment: cloud\n---\n\n## API key\n\n@include: _setup.md#install\n\n`API key` in code and [API key](index.md) in a link aren't occurrences.\n",
        ),
    ])
}

#[test]
fn a_page_is_used_by_the_links_to_it() {
    let p = quill();
    assert_eq!(
        uses(&p, &Usable::File(path("guide.md"))),
        [
            (
                "index.md".into(),
                "[setup](guide.md#setup)".into(),
                UseKind::Link
            ),
            (
                "index.md".into(),
                "[the guide](guide.md)".into(),
                UseKind::Link
            ),
        ]
    );
}

#[test]
fn a_fragment_is_used_by_its_includes() {
    let p = quill();
    assert_eq!(
        uses(&p, &Usable::File(path("_setup.md"))),
        [
            (
                "guide.md".into(),
                "@include: _setup.md".into(),
                UseKind::Include
            ),
            (
                "reference.md".into(),
                "@include: _setup.md#install".into(),
                UseKind::Include
            ),
        ]
    );
}

#[test]
fn a_heading_is_used_by_links_through_the_page_that_includes_it() {
    let p = quill();
    let setup = Usable::Heading {
        file: path("_setup.md"),
        id: "setup".into(),
    };
    assert_eq!(
        uses(&p, &setup),
        [(
            "index.md".into(),
            "[setup](guide.md#setup)".into(),
            UseKind::Link
        )]
    );
    let install = Usable::Heading {
        file: path("_setup.md"),
        id: "install".into(),
    };
    assert_eq!(
        uses(&p, &install),
        [
            (
                "guide.md".into(),
                "[about {product}](#install)".into(),
                UseKind::Link
            ),
            (
                "reference.md".into(),
                "@include: _setup.md#install".into(),
                UseKind::Include
            ),
        ]
    );
}

#[test]
fn model_entries_are_used_where_pages_name_them() {
    let p = quill();
    assert_eq!(
        uses(&p, &Usable::Phrase("product".into())),
        [("guide.md".into(), "{product}".into(), UseKind::Phrase)]
    );
    assert_eq!(
        uses(&p, &Usable::Feature("sso".into())),
        [(
            "guide.md".into(),
            "available: sso".into(),
            UseKind::Availability
        )]
    );
    assert_eq!(
        uses(&p, &Usable::Dimension("pm".into())),
        [
            ("_setup.md".into(), "pm=npm".into(), UseKind::Variant),
            ("_setup.md".into(), "pm=pnpm".into(), UseKind::Variant),
        ]
    );
    assert_eq!(
        uses(&p, &Usable::Dimension("deployment".into())),
        [
            ("_setup.md".into(), "cloud".into(), UseKind::Availability),
            ("reference.md".into(), "variant:".into(), UseKind::Variant),
        ]
    );
    assert_eq!(
        uses(&p, &Usable::Note("security".into())),
        [("guide.md".into(), "security".into(), UseKind::Note)]
    );
    assert_eq!(
        uses(&p, &Usable::Note("note".into())),
        [("guide.md".into(), "@note".into(), UseKind::Note)]
    );
    assert_eq!(
        uses(&p, &Usable::Widget("quill-lab".into())),
        [("guide.md".into(), "@quill-lab".into(), UseKind::Widget)]
    );
}

#[test]
fn a_glossary_term_is_used_where_its_text_is_in_prose() {
    let p = quill();
    assert_eq!(
        uses(&p, &Usable::Term("api-key".into())),
        [
            ("guide.md".into(), "api key".into(), UseKind::Term),
            ("index.md".into(), "API key".into(), UseKind::Term),
            ("index.md".into(), "API keys".into(), UseKind::Term),
        ]
    );
}

#[test]
fn a_marked_glossary_term_has_no_uses_in_prose() {
    // A marked term isn't linked automatically: "agent" in prose is just a
    // word, and an author's link to the term's page is a use of that page.
    let p = project(&[
        (
            "docs/index.md",
            "# Home\n\nThe agent runs. See [the agent](reference.md#agent).\n",
        ),
        (
            "docs/reference.md",
            "# Reference\n\n## Agent\n\nIt runs builds.\n",
        ),
    ]);
    assert_eq!(uses(&p, &Usable::Term("agent".into())), []);
    assert_eq!(p.use_counts().get(&Usable::Term("agent".into())), None);
    assert_eq!(
        uses(
            &p,
            &Usable::Heading {
                file: path("reference.md"),
                id: "agent".into()
            }
        ),
        [(
            "index.md".into(),
            "[the agent](reference.md#agent)".into(),
            UseKind::Link
        )]
    );
}

#[test]
fn counts_are_the_lengths_of_the_lists() {
    let p = quill();
    let counts = p.use_counts();
    assert!(!counts.is_empty());
    for (target, count) in &counts {
        assert_eq!(p.uses(target).len(), *count, "{target:?}");
    }
    assert_eq!(counts.get(&Usable::File(path("index.md"))), Some(&1));
    let mut total = 0;
    for file in p.files() {
        total += p.uses_in(&file.path).len();
    }
    assert_eq!(total, counts.values().sum::<usize>());
    assert_eq!(counts.get(&Usable::Phrase("missing".into())), None);
}
