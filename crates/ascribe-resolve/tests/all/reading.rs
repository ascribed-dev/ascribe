//! A resolved page as its readers read it (`ascribe_resolve::reading`): each
//! block's text and the map from it back to the source, the place a match
//! is, and the places of a query across pages and builds, merged by line.
//! That every block agrees with the plain output is held in `ascribe-emit`
//! (`tests/all/reading.rs`); the query's grammar, in the module's unit tests.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::PathBuf;
use std::sync::Arc;

use ascribe_core::FileId;
use ascribe_resolve::reading::{At, Found, In, Origin, PieceKind, Query, Reading, Through, find};
use ascribe_resolve::{DiskFs, Layout, MemoryFs, Project};
use proptest::prelude::*;

use crate::build_support::{MODEL, project, project_with, resolve, router};

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

/// The path of the file a place is at, and its line.
fn path_line(project: &Project, at: &At) -> (String, u32) {
    match at {
        At::Source { file, line, .. } => (project.path_of(*file).unwrap().to_string(), *line),
        At::Code { path, line, .. } => (path.clone(), *line),
        At::Phrase { key, .. } => (format!("phrase:{key}"), 0),
    }
}

/// A project with content files and other files, such as code files, both
/// relative to the project's folder.
fn project_with_files(model: &str, files: &[(&str, &str)]) -> Project {
    let model = Arc::new(ascribe_model::load_str(model, FileId::new(0)).unwrap());
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (name, text) in files {
        fs = fs.with_file(name, text);
    }
    Project::load(model, layout, &fs)
}

/// Every byte of every block's text is in exactly one piece; a copied
/// piece's text is the source's at its span, and a phrase's is its value.
fn every_character_has_a_place(project: &Project, reading: &Reading) {
    for block in &reading.blocks {
        let mut at = 0;
        let source = block.source(project);
        assert!(!source.is_empty(), "no file for {block:#?}");
        for piece in &block.pieces {
            assert_eq!(piece.range.start, at, "a gap or overlap in {block:#?}");
            at = piece.range.end;
            let text = &block.text[piece.range.clone()];
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

/// [`MODEL`], with a source of code files and a type whose description is
/// searched.
fn code_model(code: &str) -> String {
    MODEL.replace(
        "[types.page.frontmatter]\ntitle = { type = \"string\", phrases = true }\ndescription = \"string?\"",
        "[types.page.frontmatter]\ntitle = { type = \"string\", phrases = true }\ndescription = { type = \"string?\", role = \"description\", phrases = true }",
    ) + &format!("\n[sources.code]\npath = \"{code}\"\n")
}

/// A code file with a region, a line taken out of it, and indentation.
const CODE: &str = "import os\n\n# :snippet-start: main\n\ndef main():\n    print(\"{product}\")  # :remove:\n    print(\"hi there\")\n\n    return os.sep\n# :snippet-end:\nmain()\n";

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
        line.clone()
            .prop_map(|l| format!(".{l}\n@details:\nInside.\n@end\n")),
        Just("@include: _part.md\n".to_owned()),
        Just("@snippet: code:app.py#main\n".to_owned()),
        Just("@snippet {phrases=true}: code:app.py\n".to_owned()),
    ];
    let field = prop_oneof![
        Just("A title".to_owned()),
        Just("\"Use {product}\"".to_owned()),
        Just("Use {product} with {cloud}".to_owned()),
        Just("'Say it' # a comment".to_owned()),
        Just(">\n  Folded {product}\n  text".to_owned()),
    ];
    (field.clone(), field, prop::collection::vec(block, 1..6)).prop_map(
        |(title, description, b)| {
            format!(
                "---\ntype: page\ntitle: {title}\ndescription: {description}\n---\n\n{}",
                b.join("\n")
            )
        },
    )
}

proptest! {
    #[test]
    fn every_character_of_any_page_has_a_place(body in page()) {
        let project = project_with_files(&code_model("code"), &[
            ("docs/index.md", &body),
            ("docs/linked.md", LINKED),
            ("docs/_part.md", "Included {product} text.\n\n## Part\n\n| {cloud} | b |\n|---|---|\n"),
            ("code/app.py", CODE),
        ]);
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
            (In::Title, "T"),
            (In::Heading, "Use Quill"),
            (In::Text, "Sign in to Quill Cloud, and run quill sync now."),
            (In::Table, "Name Says"),
            (In::Table, "Quill Hi"),
        ]
    );
    every_character_has_a_place(&project, &r);
    assert_eq!(r.blocks[2].under.as_ref().unwrap().text, "Use Quill");
}

#[test]
fn html_blocks_and_directive_lines_are_read_and_never_searched() {
    let project = project(&[(
        "index.md",
        "---\ntitle: T\n---\n\n<div class=\"quill\">\n  Quill   here\n</div>\n\n@id: quill\n## Quill setup\n",
    )]);
    let r = reading(&project, "index.md", "site");
    let texts: Vec<(In, &str)> = r.blocks.iter().map(|b| (b.kind, b.text.as_str())).collect();
    assert_eq!(
        texts,
        [
            (In::Title, "T"),
            (In::Html, "<div class=\"quill\"> Quill here </div>"),
            (In::Directive, "quill"),
            (In::Heading, "Quill setup"),
        ]
    );
    every_character_has_a_place(&project, &r);
    let found = search(&project, "quill");
    let kinds: Vec<In> = found.places.iter().map(|p| p.kind).collect();
    assert_eq!(kinds, [In::Heading]);
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
    assert_eq!(r.blocks[1].text, "See The Quill guide or Setup.");
    every_character_has_a_place(&project, &r);
}

#[test]
fn included_blocks_are_read_where_they_are_written() {
    let project = project(&[
        (
            "index.md",
            "---\ntitle: T\n---\n\nOwn text.\n\n@include: _part.md\n",
        ),
        ("_part.md", "Included {product} text.\n"),
    ]);
    let r = reading(&project, "index.md", "site");
    let texts: Vec<(In, &str)> = r.blocks.iter().map(|b| (b.kind, b.text.as_str())).collect();
    assert_eq!(
        texts,
        [
            (In::Title, "T"),
            (In::Text, "Own text."),
            (In::Text, "Included Quill text.")
        ]
    );
    let part = &r.blocks[2];
    assert!(matches!(part.origin, Origin::Fragment), "{part:?}");
    assert_eq!(project.path_of(part.file).unwrap().as_str(), "_part.md");
    every_character_has_a_place(&project, &r);
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
            At::Source { .. } | At::Code { .. } => None,
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
    assert_eq!(place.shown.len(), 1);
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
        .filter(|p| p.kind != In::Phrase)
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

#[test]
fn a_match_after_a_phrase_that_matches_is_the_line_too() {
    let project = project(&[(
        "a.md",
        "---\ntitle: A\n---\n\nSign in to {cloud}, then edit `quill.yaml`.\n\nUse {cloud}.\n",
    )]);
    let found = search(&project, "quill");
    let mut kinds: Vec<In> = found.places.iter().map(|p| p.kind).collect();
    kinds.sort_by_key(|k| *k != In::Phrase);
    assert_eq!(kinds, [In::Phrase, In::Text], "{found:#?}");
    let place = found.places.iter().find(|p| p.kind == In::Text).unwrap();
    let (line, _) = line_of(&project, &place.at);
    assert_eq!(line, 5);
    assert_eq!(&place.text[place.matched.clone()], "quill");
    assert_eq!(place.matched.start, place.text.find("quill.yaml").unwrap());
    assert!(place.through.is_empty());
    let At::Source { column, .. } = place.at else {
        unreachable!()
    };
    assert_eq!(column, 32);
}

#[test]
fn searching_quill_finds_the_lines_that_name_quill_yaml_after_the_phrase() {
    let project = quill();
    let found = search(&project, "quill");
    let lines: Vec<(String, u32)> = found
        .places
        .iter()
        .filter_map(|p| match p.at {
            At::Source { file, line, .. } => {
                Some((project.path_of(file).unwrap().to_string(), line))
            }
            At::Phrase { .. } | At::Code { .. } => None,
        })
        .collect();
    let has = |want: (&str, u32)| lines.iter().any(|(p, l)| (p.as_str(), *l) == want);
    assert!(has(("install-agent.md", 62)), "{lines:?}");
    // `keys.md:10` says `quill` only in `{cloud}`'s value: the phrase's place.
    assert!(!has(("keys.md", 10)), "{lines:?}");
    assert!(
        found
            .places
            .iter()
            .any(|p| matches!(&p.at, At::Phrase { key, .. } if key == "cloud"))
    );
}

/// Every page of a project that includes `_part.md`, in every build.
fn includes(pages: usize) -> Project {
    let mut files: Vec<(String, String)> = (0..pages)
        .map(|n| {
            (
                format!("p{n:02}.md"),
                format!("---\ntitle: P{n}\n---\n\n## Before {n}\n\n@include: _part.md\n"),
            )
        })
        .collect();
    files.push((
        "_part.md".into(),
        "Make sure you have a docs repository.\n".into(),
    ));
    let files: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    project(&files)
}

#[test]
fn a_fragment_line_is_one_place_with_the_page_that_shows_it() {
    let project = includes(1);
    let found = search(&project, "\"docs repository\"");
    assert_eq!(found.places.len(), 1, "{found:#?}");
    let place = &found.places[0];
    assert_eq!(path_line(&project, &place.at), ("_part.md".into(), 1));
    assert!(place.shared);
    let shown: Vec<(&str, &str)> = place
        .shown
        .iter()
        .map(|s| (s.page.as_str(), s.under.as_ref().unwrap().id.as_str()))
        .collect();
    assert_eq!(shown, [("p00.md", "before-0")]);
    assert_eq!(found.pages, 1);
}

#[test]
fn a_fragment_on_thirty_pages_is_one_place_on_thirty_pages() {
    let project = includes(30);
    let found = search(&project, "\"docs repository\"");
    assert_eq!(found.places.len(), 1);
    let place = &found.places[0];
    assert_eq!(place.shown.len(), 30);
    assert_eq!(place.shown[0].page.as_str(), "p00.md");
    assert_eq!(place.shown[29].page.as_str(), "p29.md");
    assert_eq!(place.shown[7].under.as_ref().unwrap().id, "before-7");
    assert_eq!(found.pages, 30);
}

#[test]
fn a_fragment_in_a_fragment_names_the_page() {
    let project = project(&[
        (
            "index.md",
            "---\ntitle: T\n---\n\n## Start\n\n@include: _outer.md\n",
        ),
        ("_outer.md", "Outer text.\n\n@include: _inner.md\n"),
        ("_inner.md", "Inner docs repository.\n"),
    ]);
    let found = search(&project, "\"docs repository\"");
    assert_eq!(found.places.len(), 1);
    let place = &found.places[0];
    assert_eq!(path_line(&project, &place.at), ("_inner.md".into(), 1));
    let pages: Vec<&str> = place.shown.iter().map(|s| s.page.as_str()).collect();
    assert_eq!(pages, ["index.md"]);
}

#[test]
fn a_fragment_line_one_build_shows_on_a_page_names_the_build_there() {
    let project = project(&[
        (
            "a.md",
            "---\ntitle: A\n---\n\n@variant {pm=pnpm}:\n@include: _part.md\n@variant {pm=npm}:\nOther.\n@end\n",
        ),
        ("b.md", "---\ntitle: B\n---\n\n@include: _part.md\n"),
        ("_part.md", "Run the docs repository.\n"),
    ]);
    let found = search(&project, "\"docs repository\"");
    assert_eq!(found.places.len(), 1);
    let place = &found.places[0];
    assert!(place.builds.iter().any(|b| b == "npm-only"), "{place:#?}");
    let a = &place.shown[0];
    assert_eq!(a.page.as_str(), "a.md");
    assert!(!a.builds.iter().any(|b| b == "npm-only"), "{a:?}");
    let b = &place.shown[1];
    assert_eq!(b.builds, place.builds);
}

#[test]
fn a_page_own_places_come_before_its_fragments() {
    let project = project(&[
        (
            "index.md",
            "---\ntitle: T\n---\n\nYour docs repository.\n\n@include: _part.md\n",
        ),
        ("_part.md", "A docs repository.\n"),
    ]);
    let found = search(&project, "\"docs repository\"");
    let order: Vec<(String, u32)> = found
        .places
        .iter()
        .map(|p| path_line(&project, &p.at))
        .collect();
    assert_eq!(order, [("index.md".into(), 5), ("_part.md".into(), 1)]);
}

/// A page that shows `code/app.py`'s region `main`, through `code`.
fn snippet_project(code: &str, snippet: &str) -> Project {
    project_with_files(
        &code_model(code),
        &[
            (
                "docs/index.md",
                &format!("---\ntitle: T\n---\n\n## Run\n\n@snippet: {snippet}\n"),
            ),
            (&format!("{code}/app.py"), CODE),
        ],
    )
}

#[test]
fn a_match_in_a_snippet_is_at_the_code_file_line() {
    let project = snippet_project("code", "code:app.py#main");
    let r = reading(&project, "index.md", "site");
    every_character_has_a_place(&project, &r);
    let found = search(&project, "\"hi there\"");
    assert_eq!(found.places.len(), 1, "{found:#?}");
    let place = &found.places[0];
    // Line 6 is taken out with `:remove:`; the match is on line 7.
    assert_eq!(path_line(&project, &place.at), ("code/app.py".into(), 7));
    let At::Code { column, .. } = place.at else {
        unreachable!()
    };
    assert_eq!(column, 12);
    assert_eq!(place.kind, In::Code);
    assert_eq!(place.through, [Through::Code("code:app.py#main".into())]);
    assert!(place.shared && place.end_line.is_none());
    assert_eq!(place.shown[0].page.as_str(), "index.md");
    assert_eq!(place.shown[0].under.as_ref().unwrap().id, "run");
    let found = search(&project, "\"return os\"");
    assert_eq!(
        path_line(&project, &found.places[0].at),
        ("code/app.py".into(), 9)
    );
}

#[test]
fn a_match_in_a_whole_file_snippet_is_at_its_line() {
    let project = snippet_project("code", "code:app.py");
    let found = search(&project, "main()");
    let lines: Vec<(String, u32)> = found
        .places
        .iter()
        .map(|p| path_line(&project, &p.at))
        .collect();
    // The code's first match is on line 5, `def main():`, and the block's
    // other `main()` is the same place.
    assert_eq!(lines, [("code/app.py".into(), 5)]);
}

#[test]
fn a_code_file_outside_the_project_folder_is_named_as_the_snippet_names_it() {
    let project = snippet_project("../code", "code:app.py#main");
    let found = search(&project, "\"hi there\"");
    assert_eq!(found.places.len(), 1, "{found:#?}");
    assert_eq!(
        path_line(&project, &found.places[0].at),
        ("code:app.py".into(), 7)
    );
}

#[test]
fn a_title_and_a_description_are_at_their_keys() {
    let project = project_with_files(
        &code_model("code"),
        &[
            (
                "docs/a.md",
                "---\ntype: page\ntitle: \"Use {product}\"\ndescription: Set up {cloud} fast.\n---\n\nUse Quill.\n\n## Quill\n",
            ),
            (
                "docs/b.md",
                "---\ntype: page\ntitle: B\ndescription: >\n  Folded\n  Quill text.\n---\n",
            ),
        ],
    );
    let r = reading(&project, "a.md", "site");
    every_character_has_a_place(&project, &r);
    assert_eq!(r.blocks[0].text, "Use Quill");
    assert_eq!(r.blocks[1].text, "Set up Quill Cloud fast.");
    let places = |query: &str| -> Vec<(String, u32, In)> {
        search(&project, query)
            .places
            .iter()
            .map(|p| {
                let (path, line) = path_line(&project, &p.at);
                (path, line, p.kind)
            })
            .collect()
    };
    // Titles first, then headings, then the rest.
    assert_eq!(
        places("use"),
        [("a.md".into(), 3, In::Title), ("a.md".into(), 7, In::Text)]
    );
    assert_eq!(
        places("quill"),
        [
            ("a.md".into(), 9, In::Heading),
            // `ascribe.toml` comes before `docs/`.
            ("phrase:product".into(), 0, In::Phrase),
            ("phrase:cloud".into(), 0, In::Phrase),
            ("a.md".into(), 7, In::Text),
            ("b.md".into(), 4, In::Description),
        ]
    );
    let found = search(&project, "\"use quill\"");
    assert_eq!(found.places[0].kind, In::Title);
    assert_eq!(found.places[0].through, [Through::Phrase("product".into())]);
    assert_eq!(
        places("\"set up quill\""),
        [("a.md".into(), 4, In::Description)]
    );
}

#[test]
fn a_type_with_no_description_field_has_only_its_title_read() {
    // A field named `description` would be the page's description by its
    // name.
    let model = MODEL.replace("description = \"string?\"", "summary = \"string?\"");
    let project = project_with(
        &model,
        &[(
            "a.md",
            "---\ntype: page\ntitle: Quill guide\nsummary: Quill text.\n---\n\nBody.\n",
        )],
    );
    let r = reading(&project, "a.md", "site");
    let kinds: Vec<In> = r.blocks.iter().map(|b| b.kind).collect();
    assert_eq!(kinds, [In::Title, In::Text]);
}

#[test]
fn snippet_code_that_cannot_be_lined_up_is_at_its_first_line() {
    // A phrase whose value has a line break gives the code a line its file
    // doesn't have.
    let model = code_model("code").replace("[phrases]\n", "[phrases]\nsteps = \"one\\ntwo\"\n");
    let project = project_with_files(
        &model,
        &[
            (
                "docs/index.md",
                "---\ntitle: T\n---\n\n@snippet {phrases=true}: code:run.sh\n",
            ),
            ("code/run.sh", "echo start\n# {steps}\necho done\n"),
        ],
    );
    let r = reading(&project, "index.md", "site");
    every_character_has_a_place(&project, &r);
    let found = search(&project, "\"echo done\"");
    assert_eq!(found.places.len(), 1, "{found:#?}");
    let place = &found.places[0];
    assert_eq!(path_line(&project, &place.at), ("code/run.sh".into(), 1));
    assert_eq!(place.end_line, Some(3), "the lines shown");
}
