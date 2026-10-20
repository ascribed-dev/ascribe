//! A resolved page as its readers read it (`ascribe_resolve::reading`): each
//! block's text and the map from it back to the source, the place a match
//! is, and the places of a query across pages and builds, merged by line.
//! That every block agrees with the plain output is held in `ascribe-emit`
//! (`tests/all/reading.rs`); the query's grammar, in the module's unit tests.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::PathBuf;
use std::sync::Arc;

use ascribe_resolve::reading::{At, Found, In, PieceKind, Query, Reading, Through, find};
use ascribe_resolve::{DiskFs, Layout, Project};
use proptest::prelude::*;

use crate::build_support::{project, project_with, resolve, router};

/// A page's reading in a build.
fn reading(project: &Project, page: &str, build: &str) -> Reading {
    Reading::of(project, &resolve(project, page, build))
}

/// Every page of every build, read, the builds in the model's order.
fn every_reading(project: &Project) -> Vec<Reading> {
    let mut out = Vec::new();
    for build in &project.model().builds {
        let project = project.for_build(build);
        let resolved = project.resolve_build(build, &router(&project));
        out.extend(resolved.pages.iter().map(|p| Reading::of(&project, p)));
    }
    out
}

fn search(project: &Project, query: &str) -> Found {
    find(
        project,
        &every_reading(project),
        &Query::parse(query).unwrap(),
    )
}

/// The source of the line a place is at, and its line number.
fn line_of(project: &Project, at: &At) -> (u32, String) {
    let At::Source { file, line, .. } = at else {
        panic!("{at:?} isn't in a source file");
    };
    let source = &project.file_by_id(*file).unwrap().source;
    let text = source.lines().nth(*line as usize - 1).unwrap().to_owned();
    (*line, text)
}

/// Every byte of every block's text is in exactly one piece; a copied
/// piece's text is the source's at its span, and a phrase's is its value.
fn every_character_has_a_place(project: &Project, reading: &Reading) {
    for block in &reading.blocks {
        let mut at = 0;
        for piece in &block.pieces {
            assert_eq!(piece.range.start, at, "a gap or overlap in {block:#?}");
            at = piece.range.end;
            let text = &block.text[piece.range.clone()];
            let source = &project.file_by_id(block.file).unwrap().source;
            match &piece.kind {
                PieceKind::Copied => {
                    assert_eq!(&source[piece.source.range()], text, "in {block:#?}");
                }
                PieceKind::Phrase(key) => {
                    let value = project.model().phrase(key).unwrap();
                    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
                    assert_eq!(text, value, "in {block:#?}");
                }
                PieceKind::Made | PieceKind::Borrowed { .. } => {
                    assert!(piece.source.end() <= source.len(), "in {block:#?}");
                }
            }
        }
        assert_eq!(at, block.text.len(), "the pieces end early in {block:#?}");
        assert_eq!(
            block.text,
            block.text.split_whitespace().collect::<Vec<_>>().join(" "),
            "white space isn't collapsed in {block:#?}"
        );
    }
}

const LINKED: &str = "---\ntitle: The Quill guide\n---\n\n## Setup\n\nSet it up.\n";

fn page() -> impl Strategy<Value = String> {
    let word = prop::sample::select(vec![
        "word",
        "café",
        "{product}",
        "{cloud}",
        "{unknown}",
        "*em*",
        "**strong**",
        "`code`",
        "[link](linked.md)",
        "[](linked.md)",
        "[](linked.md#setup)",
        "![alt text](a.png)",
        "<kbd>Ctrl</kbd>",
        "a\\*b",
        "&amp;",
        "—",
    ]);
    let line = prop::collection::vec(word, 1..6).prop_map(|w| w.join(" "));
    let block = prop_oneof![
        line.clone().prop_map(|l| format!("{l}\n")),
        (line.clone(), line.clone()).prop_map(|(a, b)| format!("{a}\n{b}\n")),
        line.clone().prop_map(|l| format!("## {l}\n")),
        (line.clone(), line.clone()).prop_map(|(a, b)| format!("- {a}\n  {b}\n")),
        (line.clone(), line.clone())
            .prop_map(|(a, b)| format!("| {a} | {b} |\n|---|---|\n| {b} | {a} |\n")),
        line.clone().prop_map(|l| format!("@note: {l}\n")),
        line.clone().prop_map(|l| format!("> {l}\n> {l}\n")),
        line.clone()
            .prop_map(|l| format!("- item\n\n  ```sh phrases=true\n  {l}\n  ```\n")),
        line.prop_map(|l| format!(".{l}\n@details:\nInside.\n@end\n")),
    ];
    prop::collection::vec(block, 1..6)
        .prop_map(|b| format!("---\ntitle: T\n---\n\n{}", b.join("\n")))
}

proptest! {
    #[test]
    fn every_character_of_any_page_has_a_place(body in page()) {
        let project = project(&[("index.md", &body), ("linked.md", LINKED)]);
        let reading = reading(&project, "index.md", "site");
        every_character_has_a_place(&project, &reading);
    }
}

#[test]
fn a_block_reads_without_markup_with_phrases_and_spaces_collapsed() {
    let project = project(&[(
        "index.md",
        "---\ntitle: T\n---\n\n## Use {product}\n\nSign in to *{cloud}*, and run\n`quill   sync` [now](x.md).\n\n| Name | Says |\n|---|---|\n| {product} | **Hi** |\n",
    )]);
    let r = reading(&project, "index.md", "site");
    let texts: Vec<(In, &str)> = r.blocks.iter().map(|b| (b.kind, b.text.as_str())).collect();
    assert_eq!(
        texts,
        [
            (In::Heading, "Use Quill"),
            (In::Text, "Sign in to Quill Cloud, and run quill sync now."),
            (In::Table, "Name Says"),
            (In::Table, "Quill Hi"),
        ]
    );
    every_character_has_a_place(&project, &r);
    assert_eq!(r.blocks[1].under.as_ref().unwrap().text, "Use Quill");
}

#[test]
fn a_link_with_no_text_reads_as_its_target_title() {
    let project = project(&[
        (
            "index.md",
            "---\ntitle: T\n---\n\nSee [](linked.md) or [](linked.md#setup).\n",
        ),
        ("linked.md", LINKED),
    ]);
    let r = reading(&project, "index.md", "site");
    assert_eq!(r.blocks[0].text, "See The Quill guide or Setup.");
    every_character_has_a_place(&project, &r);
}

#[test]
fn only_the_page_own_blocks_are_read() {
    let project = project(&[
        (
            "index.md",
            "---\ntitle: T\n---\n\nOwn text.\n\n@include: _part.md\n",
        ),
        ("_part.md", "Included text.\n"),
    ]);
    let r = reading(&project, "index.md", "site");
    let texts: Vec<&str> = r.blocks.iter().map(|b| b.text.as_str()).collect();
    assert_eq!(texts, ["Own text."]);
}

const PAGE: &str = "---\ntitle: T\n---\n\n## Connect\n\n@variant {deployment=cloud}:\nSign in to {cloud} and copy\na key.\n@variant {deployment=self-managed}:\nPoint it at your server.\n@end\n\nUse {product} with {cloud}.\n\nRead [](linked.md) first.\n";

fn located(query: &str) -> (Project, Found) {
    let project = project(&[("index.md", PAGE), ("linked.md", LINKED)]);
    let found = find(
        &project,
        &[reading(&project, "index.md", "site")],
        &Query::parse(query).unwrap(),
    );
    (project, found)
}

#[test]
fn a_match_in_copied_text_is_its_own_line() {
    let (project, found) = located("\"copy a key\"");
    let place = &found.places[0];
    assert_eq!(
        line_of(&project, &place.at).1,
        "Sign in to {cloud} and copy"
    );
    assert_eq!(&place.text[place.matched.clone()], "copy a key");
    assert!(place.through.is_empty());
    assert_eq!(place.arm.as_deref(), Some("deployment=cloud"));
    assert_eq!(place.under.as_ref().unwrap().id, "connect");
}

#[test]
fn a_match_from_copied_text_into_a_phrase_is_the_line_that_uses_it() {
    let (project, found) = located("\"sign in to quill cloud\"");
    assert_eq!(found.places.len(), 1);
    let place = &found.places[0];
    let (line, text) = line_of(&project, &place.at);
    assert_eq!((line, text.as_str()), (8, "Sign in to {cloud} and copy"));
    assert_eq!(place.through, [Through::Phrase("cloud".into())]);
    let At::Source { column, .. } = place.at else {
        unreachable!()
    };
    assert_eq!(column, 1);
}

#[test]
fn a_match_over_two_phrases_names_both() {
    let (_, found) = located("\"quill with quill\"");
    assert_eq!(
        found.places[0].through,
        [
            Through::Phrase("product".into()),
            Through::Phrase("cloud".into())
        ]
    );
}

#[test]
fn a_match_wholly_inside_a_value_is_the_phrase_in_the_model() {
    let (project, found) = located("\"quill cloud\"");
    let at_phrase: Vec<&str> = found
        .places
        .iter()
        .filter_map(|p| match &p.at {
            At::Phrase { key, .. } => Some(key.as_str()),
            At::Source { .. } => None,
        })
        .collect();
    assert_eq!(at_phrase, ["cloud"]);
    let place = found.places.iter().find(|p| p.kind == In::Phrase).unwrap();
    assert_eq!(place.text, "Quill Cloud");
    let At::Phrase { span, .. } = place.at else {
        unreachable!()
    };
    assert_eq!(span, project.model().phrases[1].span);
    // Every line that uses the phrase is in that one place.
    assert_eq!(found.places.len(), 1);
}

#[test]
fn a_match_in_a_borrowed_title_is_the_link() {
    let (project, found) = located("\"quill guide\"");
    let place = &found.places[0];
    assert_eq!(line_of(&project, &place.at).1, "Read [](linked.md) first.");
    assert_eq!(
        place.through,
        [Through::Title {
            page: ascribe_core::RelPath::parse("linked.md").unwrap(),
            id: None
        }]
    );
}

#[test]
fn a_match_never_crosses_blocks() {
    let (_, found) = located("server use");
    assert!(found.places.is_empty());
}

fn quill() -> Project {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let model = ascribe_model::load(root.join("ascribe.toml")).expect("the Quill model loads");
    let layout = Layout::from_model(&model);
    let fs = DiskFs::new(&root, &layout);
    Project::load(Arc::new(model), layout, &fs)
}

#[test]
fn the_thread_place_is_line_62_of_install_agent_through_the_phrase() {
    let project = quill();
    let found = search(&project, "\"sign in to quill cloud\"");
    // `keys.md` says it too, on line 10.
    assert_eq!(found.places.len(), 2, "{found:#?}");
    let place = &found.places[0];
    let At::Source { file, line, .. } = place.at else {
        panic!("{place:?}")
    };
    assert_eq!(project.path_of(file).unwrap().as_str(), "install-agent.md");
    assert_eq!(line, 62);
    assert_eq!(place.through, [Through::Phrase("cloud".into())]);
    assert_eq!(place.arm.as_deref(), Some("deployment=cloud"));
    let under = place.under.as_ref().unwrap();
    assert_eq!(
        (under.text.as_str(), under.id.as_str()),
        ("Connect to Quill", "connect")
    );
    let all: Vec<&str> = project
        .model()
        .builds
        .iter()
        .map(|b| b.name.as_str())
        .collect();
    assert_eq!(place.builds, all);
    assert_eq!(place.pages.len(), 1);
    let At::Source { file, line, .. } = found.places[1].at else {
        panic!("{found:?}")
    };
    assert_eq!(
        (project.path_of(file).unwrap().as_str(), line),
        ("keys.md", 10)
    );
    assert_eq!(found.pages, 2);
}

#[test]
fn a_line_one_build_publishes_names_that_build() {
    let project = project(&[(
        "index.md",
        "---\ntitle: T\n---\n\n@variant {pm=npm}:\nRun npm install.\n@variant {pm=pnpm}:\nRun pnpm add.\n@end\n",
    )]);
    let found = search(&project, "\"pnpm add\"");
    assert_eq!(found.places.len(), 1);
    let builds = &found.places[0].builds;
    assert!(!builds.iter().any(|b| b == "npm-only"), "{builds:?}");
    assert!(builds.iter().any(|b| b == "site"), "{builds:?}");
}

#[test]
fn a_phrase_that_differs_by_release_line_matches_only_where_it_reads_so() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/conformance/cases/builds/lines/phrase-names-the-line");
    let text = std::fs::read_to_string(root.join("ascribe.toml")).unwrap();
    let project = project_with(&text, &[("index.md", "This is Self-managed {release}.\n")]);
    let found = search(&project, "\"self-managed 3.4\"");
    assert_eq!(found.places.len(), 1);
    assert_eq!(found.places[0].builds, ["site@3.4"]);
    let found = search(&project, "\"self-managed 3.5\"");
    assert_eq!(found.places[0].builds, ["site", "plain"]);
}

#[test]
fn places_are_headings_first_then_by_path_and_line() {
    let project = project(&[
        ("b.md", "---\ntitle: B\n---\n\nAn apple.\n\n## Apple\n"),
        ("a.md", "---\ntitle: A\n---\n\nApple one.\n\nApple two.\n"),
    ]);
    let found = search(&project, "apple");
    let order: Vec<(String, u32, In)> = found
        .places
        .iter()
        .map(|p| {
            let At::Source { file, line, .. } = p.at else {
                unreachable!()
            };
            (project.path_of(file).unwrap().to_string(), line, p.kind)
        })
        .collect();
    assert_eq!(
        order,
        [
            ("b.md".to_owned(), 7, In::Heading),
            ("a.md".to_owned(), 5, In::Text),
            ("a.md".to_owned(), 7, In::Text),
            ("b.md".to_owned(), 5, In::Text),
        ]
    );
    assert_eq!(found.pages, 2);
}

#[test]
fn a_line_with_two_matches_is_one_place() {
    let project = project(&[("a.md", "---\ntitle: A\n---\n\nApple, apple.\n")]);
    let found = search(&project, "apple");
    assert_eq!(found.places.len(), 1);
}

#[test]
fn text_that_spells_a_phrase_value_before_the_phrase_is_still_copied() {
    let project = project(&[("a.md", "---\ntitle: A\n---\n\nUse Quill {product} now.\n")]);
    let r = reading(&project, "a.md", "site");
    every_character_has_a_place(&project, &r);
    let found = find(
        &project,
        std::slice::from_ref(&r),
        &Query::parse("\"use quill\"").unwrap(),
    );
    assert!(found.places[0].through.is_empty(), "{found:#?}");
    let found = find(&project, &[r], &Query::parse("\"quill now\"").unwrap());
    assert_eq!(found.places[0].through, [Through::Phrase("product".into())]);
}
