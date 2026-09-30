//! Source ids, titles, and the rest of the per-file index (SPEC §5.5).

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use support::{path, project};
use tessera_resolve::{FileKind, PhrasePlace, Project};

const PAGE: &str = "---\ntitle: The page\n---\n\n";

fn ids(project: &Project, file: &str) -> Vec<String> {
    project
        .file(&path(file))
        .expect("the file")
        .headings
        .iter()
        .map(|h| h.source_id.clone())
        .collect()
}

#[test]
fn slugs_are_numbered_within_a_file() {
    let p = project(&[(
        "docs/a.md",
        &format!("{PAGE}## Steps\n\n## Other\n\n## Steps\n\n## Steps\n"),
    )]);
    assert_eq!(ids(&p, "a.md"), ["steps", "other", "steps-1", "steps-2"]);
}

#[test]
fn each_file_has_its_own_numbering() {
    let p = project(&[
        ("docs/a.md", &format!("{PAGE}## Steps\n")),
        ("docs/b.md", &format!("{PAGE}## Steps\n")),
        ("docs/_c.md", "## Steps\n"),
    ]);
    for file in ["a.md", "b.md", "_c.md"] {
        assert_eq!(ids(&p, file), ["steps"], "{file}");
    }
}

#[test]
fn an_explicit_id_is_the_source_id_and_takes_no_part_in_numbering() {
    // A slug is numbered only against earlier slugs, so the second
    // heading's slug is `setup`, the same as the first's `@id`.
    let p = project(&[(
        "docs/a.md",
        &format!("{PAGE}## Install\n@id: setup\n\n## Setup\n\n## Setup\n"),
    )]);
    assert_eq!(ids(&p, "a.md"), ["setup", "setup", "setup-1"]);
    let file = p.file(&path("a.md")).expect("a.md");
    assert_eq!(
        file.headings[0].explicit_id.as_ref().map(|e| e.id.as_str()),
        Some("setup")
    );
    // A lookup by id finds the first.
    assert_eq!(
        file.heading_by_id("setup").map(|h| h.text.as_str()),
        Some("Install")
    );
}

#[test]
fn an_explicit_id_replaces_the_slug() {
    let p = project(&[(
        "docs/a.md",
        &format!("{PAGE}## Configuration\n@id: config-setup\n"),
    )]);
    let file = p.file(&path("a.md")).expect("a.md");
    assert!(file.heading_by_id("config-setup").is_some());
    assert!(file.heading_by_id("configuration").is_none());
}

#[test]
fn a_heading_slug_is_computed_from_its_text_with_phrases_substituted() {
    let p = project(&[(
        "docs/a.md",
        &format!(
            "{PAGE}## Connect to {{product}}\n\n## Use {{unknown}} here\n\n## *Bold* and `code`\n"
        ),
    )]);
    let file = p.file(&path("a.md")).expect("a.md");
    assert_eq!(file.headings[0].text, "Connect to Quill");
    assert_eq!(file.headings[0].source_id, "connect-to-quill");
    assert!(file.headings[0].has_phrase);
    // An undeclared key is literal text.
    assert_eq!(file.headings[1].text, "Use {unknown} here");
    assert!(!file.headings[1].has_phrase);
    assert_eq!(file.headings[2].text, "Bold and code");
    assert_eq!(file.headings[2].source_id, "bold-and-code");
}

#[test]
fn a_heading_in_a_container_is_numbered_with_the_rest_of_the_file() {
    let p = project(&[(
        "docs/a.md",
        &format!("{PAGE}## Setup\n\n@variant {{deployment=x}}:\n## Setup\n\n@end\n\n## Setup\n"),
    )]);
    // `deployment` isn't declared here, but the structure is still read.
    assert_eq!(ids(&p, "a.md"), ["setup", "setup-1", "setup-2"]);
}

#[test]
fn a_heading_with_an_empty_slug_is_flagged() {
    // An empty slug, and its numbered repeats.
    let p = project(&[(
        "docs/a.md",
        &format!("{PAGE}## ???\n\n## 🎉\n\n## Real\n\n## Named\n@id: named\n"),
    )]);
    let file = p.file(&path("a.md")).expect("a.md");
    assert_eq!(ids(&p, "a.md"), ["", "-1", "real", "named"]);
    assert!(file.headings[0].empty_slug);
    assert!(file.headings[1].empty_slug);
    assert!(!file.headings[2].empty_slug);
    assert_eq!(p.empty_slug_headings().len(), 2);
    // Nothing can name an empty id.
    assert!(file.heading_by_id("").is_none());
}

#[test]
fn sections_run_to_the_next_heading_of_the_same_or_a_higher_level() {
    let source = format!("{PAGE}# One\n\nA.\n\n## Two\n\nB.\n\n### Three\n\nC.\n\n## Four\n\nD.\n");
    let p = project(&[("docs/a.md", &source)]);
    let file = p.file(&path("a.md")).expect("a.md");
    let text = |id: &str| {
        let h = file.heading_by_id(id).expect("heading");
        source[h.section.range()].to_owned()
    };
    assert_eq!(text("two"), "## Two\n\nB.\n\n### Three\n\nC.");
    assert_eq!(text("three"), "### Three\n\nC.");
    assert_eq!(text("four"), "## Four\n\nD.");
    assert!(text("one").ends_with("D."));
}

#[test]
fn titles_come_from_frontmatter_and_headings() {
    let p = project(&[
        (
            "docs/keys.md",
            "---\ntitle: API keys\n---\n\n## Rotate keys\n\nText.\n",
        ),
        ("docs/_f.md", "No frontmatter.\n"),
        ("docs/n.md", "---\ntitle: 42\n---\n\nText.\n"),
    ]);
    assert_eq!(p.title_for(&path("keys.md"), None), Some("API keys"));
    assert_eq!(
        p.title_for(&path("keys.md"), Some("rotate-keys")),
        Some("Rotate keys")
    );
    assert_eq!(p.title_for(&path("keys.md"), Some("nope")), None);
    assert_eq!(p.title_for(&path("_f.md"), None), None);
    assert_eq!(p.title_for(&path("n.md"), None), None);
}

#[test]
fn pages_and_fragments_are_told_apart() {
    let p = project(&[
        ("docs/index.md", "x\n"),
        ("docs/_f.md", "x\n"),
        ("docs/_snippets/a.md", "x\n"),
        ("docs/guides/_b.md", "x\n"),
        ("docs/guides/c.md", "x\n"),
        ("README.md", "not in the content root\n"),
        ("docs/notes.txt", "not markdown\n"),
    ]);
    let kind = |f: &str| p.file(&path(f)).expect("file").kind;
    assert_eq!(kind("index.md"), FileKind::Page);
    assert_eq!(kind("guides/c.md"), FileKind::Page);
    assert_eq!(kind("_f.md"), FileKind::Fragment);
    assert_eq!(kind("_snippets/a.md"), FileKind::Fragment);
    assert_eq!(kind("guides/_b.md"), FileKind::Fragment);
    assert_eq!(p.files().count(), 5);
    assert_eq!(p.pages().count(), 2);
    assert_eq!(p.fragments().count(), 3);
    assert!(p.file(&path("../README.md")).is_none());
}

#[test]
fn file_ids_are_stable_within_a_project_and_name_their_file() {
    let p = project(&[("docs/b.md", "x\n"), ("docs/a.md", "x\n")]);
    for file in p.files() {
        assert_eq!(p.path_of(file.file), Some(&file.path));
    }
    assert_eq!(
        p.path_of(p.file(&path("a.md")).expect("a").file),
        Some(&path("a.md"))
    );
}

#[test]
fn phrase_candidates_are_recorded_with_where_they_are() {
    let p = project(&[(
        "docs/a.md",
        &format!(
            "{PAGE}## Use {{product}}\n\nSee {{cloud}} and {{nope}}, or [x]({{api}}v1).\n\n```yaml phrases=true\nurl: {{api}}\n```\n\n`{{product}}` in code.\n"
        ),
    )]);
    let file = p.file(&path("a.md")).expect("a.md");
    let got: Vec<(&str, bool, PhrasePlace)> = file
        .phrases
        .iter()
        .map(|u| (u.phrase.key.as_str(), u.declared, u.place))
        .collect();
    assert_eq!(
        got,
        [
            ("product", true, PhrasePlace::Heading),
            ("cloud", true, PhrasePlace::Text),
            ("nope", false, PhrasePlace::Text),
            ("api", true, PhrasePlace::Destination),
            ("api", true, PhrasePlace::Code),
        ]
    );
}

#[test]
fn availability_markers_are_recorded() {
    let p = project(&[(
        "docs/a.md",
        &format!(
            "{PAGE}## Streaming\n@available: cloud, self-managed preview 3.4\n\n@available: nonsense((\n\nText.\n"
        ),
    )]);
    let file = p.file(&path("a.md")).expect("a.md");
    assert_eq!(file.availability.len(), 2);
    assert_eq!(
        file.availability[0]
            .primary
            .as_ref()
            .map(|(t, _)| t.as_str()),
        Some("cloud, self-managed preview 3.4")
    );
    assert!(matches!(file.availability[0].spec, Some(Ok(_))));
    assert!(matches!(file.availability[1].spec, Some(Err(_))));
}

#[test]
fn indexing_is_a_pure_function_of_the_file() {
    let a = project(&[
        ("docs/a.md", &format!("{PAGE}## One\n\n[x](b.md)\n")),
        ("docs/b.md", "x\n"),
    ]);
    let b = project(&[
        ("docs/a.md", &format!("{PAGE}## One\n\n[x](b.md)\n")),
        ("docs/zzz.md", "x\n"),
    ]);
    let (fa, fb) = (
        a.file(&path("a.md")).expect("a"),
        b.file(&path("a.md")).expect("a"),
    );
    assert_eq!(fa.headings, fb.headings);
    assert_eq!(fa.references, fb.references);
    assert_eq!(fa.title, fb.title);
}
