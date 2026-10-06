//! Source anchors in the site output (site-render contract §7): the anchor
//! fixtures in `tests/render/` and the Quill corpus are the emitter's output,
//! anchors off is the output without them byte for byte, anchors on renders
//! the same page, and every block carries an anchor.
//!
//! The fixtures' `input.md` and `unanchored.md`, and the corpus, are written by
//! the emitter from each fixture's `source/`; set `ASCRIBE_BLESS=1` to rewrite
//! them after a change to the emitter. `expected.html` is checked by hand.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ascribe_emit::{SiteEmitter, render_site_html};
use ascribe_resolve::Project;
use support::{FULL_MODEL, emit_build, first_difference, html_tree, load, memory_project, quill};

const QUILL_BUILDS: [&str; 3] = ["site", "cloud", "self-managed-3.3"];

fn render_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/render")
}

fn bless() -> bool {
    std::env::var_os("ASCRIBE_BLESS").is_some()
}

/// The fixtures written from a source: the directories with a `source/`.
fn anchor_fixtures() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(render_dir())
        .expect("tests/render exists")
        .map(|e| e.expect("a directory entry").path())
        .filter(|p| p.join("source").is_dir())
        .collect();
    dirs.sort();
    assert!(dirs.len() >= 4, "only {} anchor fixtures", dirs.len());
    dirs
}

/// Every file under `dir`, as `(path relative to dir, text)`.
fn files_under(dir: &Path, prefix: &str, out: &mut Vec<(String, String)>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("a source directory")
        .map(|e| e.expect("a directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().expect("a name").to_string_lossy();
        let rel = format!("{prefix}{name}");
        if path.is_dir() {
            files_under(&path, &format!("{rel}/"), out);
        } else {
            out.push((rel, fs::read_to_string(&path).expect("a source file")));
        }
    }
}

/// A fixture's source as a project under the full content model.
fn source_project(fixture: &Path) -> Project {
    let mut files = Vec::new();
    files_under(&fixture.join("source"), "", &mut files);
    let files: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    memory_project(FULL_MODEL, &files)
}

fn site_pages(
    root: &Path,
    project: &Project,
    build: &str,
    anchors: bool,
) -> BTreeMap<String, String> {
    let emitter = SiteEmitter::new(project.model()).with_anchors(anchors);
    emit_build(root, project, build, &emitter)
        .into_iter()
        .filter(|(path, _)| path.ends_with(".md"))
        .collect()
}

/// A page's markdown after its frontmatter.
fn body(text: &str) -> &str {
    let Some(rest) = text.strip_prefix("---\n") else {
        return text;
    };
    match rest.find("\n---\n") {
        Some(at) => rest[at + 5..].trim_start_matches('\n'),
        None => text,
    }
}

/// Checks that `path` holds `text`, or writes it when blessing.
fn check_file(path: &Path, text: &str) {
    if bless() {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).expect("the directory is created");
        }
        fs::write(path, text).expect("the file is written");
        return;
    }
    let current = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("{} is missing; run with ASCRIBE_BLESS=1", path.display()));
    assert!(
        current == text,
        "{} isn't what the emitter writes; run with ASCRIBE_BLESS=1 and check the diff\n--- emitter\n{text}",
        path.display()
    );
}

/// The anchored and unanchored markdown of every page the tests compare:
/// each fixture's page and the Quill corpus.
fn pairs() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for fixture in anchor_fixtures() {
        let name = fixture
            .file_name()
            .expect("a name")
            .to_string_lossy()
            .into_owned();
        let read = |file: &str| {
            fs::read_to_string(fixture.join(file)).unwrap_or_else(|_| panic!("{name}/{file} reads"))
        };
        out.push((name.clone(), read("input.md"), read("unanchored.md")));
    }
    let corpus = render_dir().join("corpus/quill");
    for build in QUILL_BUILDS {
        for page in ["install-agent.md", "keys.md", "quickstart.md"] {
            let dir = corpus.join(build);
            let anchored = fs::read_to_string(dir.join(page)).expect("a corpus page");
            let unanchored = fs::read_to_string(dir.join(page.replace(".md", ".unanchored.md")))
                .expect("a corpus page");
            out.push((format!("quill/{build}/{page}"), anchored, unanchored));
        }
    }
    out
}

#[test]
fn the_anchor_fixtures_are_the_emitters_output() {
    for fixture in anchor_fixtures() {
        let project = source_project(&fixture);
        for (anchors, file) in [(true, "input.md"), (false, "unanchored.md")] {
            let pages = site_pages(Path::new("/nowhere"), &project, "site", anchors);
            let page = pages
                .get("index.md")
                .expect("the fixture's page is index.md");
            check_file(&fixture.join(file), body(page));
        }
    }
}

#[test]
fn the_quill_corpus_is_the_emitters_output() {
    let root = quill();
    let project = load(&root);
    let corpus = render_dir().join("corpus/quill");
    for build in QUILL_BUILDS {
        for anchors in [true, false] {
            for (path, text) in site_pages(&root, &project, build, anchors) {
                let file = if anchors {
                    path
                } else {
                    path.replace(".md", ".unanchored.md")
                };
                check_file(&corpus.join(build).join(file), body(&text));
            }
        }
    }
}

#[test]
fn without_anchors_the_output_is_unchanged() {
    let root = quill();
    let project = load(&root);
    for build in QUILL_BUILDS {
        let default = emit_build(&root, &project, build, &SiteEmitter::new(project.model()));
        let off = emit_build(
            &root,
            &project,
            build,
            &SiteEmitter::new(project.model()).with_anchors(false),
        );
        assert_eq!(default, off, "{build}");
        for (path, text) in &default {
            assert!(
                !text.contains("ascribe-anchor") && !text.contains("data-ascribe-"),
                "{build}/{path} has an anchor"
            );
        }
    }
    for fixture in anchor_fixtures() {
        let project = source_project(&fixture);
        let default = emit_build(
            Path::new("/nowhere"),
            &project,
            "site",
            &SiteEmitter::new(project.model()),
        );
        let off = site_pages(Path::new("/nowhere"), &project, "site", false);
        for (path, text) in off {
            assert_eq!(default.get(&path), Some(&text), "{}", fixture.display());
        }
    }
}

#[test]
fn with_anchors_the_page_is_the_same_page() {
    for (name, anchored, unanchored) in pairs() {
        let with = render_site_html(&anchored);
        let without = render_site_html(&unanchored);
        assert!(
            !with.contains("ascribe-anchor"),
            "{name}: an anchor comment is left in the HTML:\n{with}"
        );
        let (want, got) = (html_tree(&without, false), html_tree(&with, true));
        if let Some((at, w, g)) = first_difference(&want, &got) {
            panic!(
                "{name}: the page differs at node {at}\nwithout: {w}\nwith:    {g}\n--- with\n{with}"
            );
        }
    }
}

/// The elements a reviewer can point at.
fn is_block(tag: &str) -> bool {
    matches!(
        tag,
        "p" | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "pre"
            | "ul"
            | "ol"
            | "li"
            | "table"
            | "blockquote"
            | "hr"
            | "div"
            | "details"
    ) || ((tag.starts_with("ascribe-") || tag.starts_with("quill-"))
        && !matches!(tag, "ascribe-attributes" | "ascribe-availability-target"))
}

#[test]
fn every_block_has_an_anchor() {
    for (name, anchored, _) in pairs() {
        let html = render_site_html(&anchored);
        for line in html_tree(&html, false) {
            let node = line.trim_start();
            let Some(rest) = node.strip_prefix('<') else {
                continue;
            };
            let tag = rest.split([' ', '>']).next().unwrap_or("");
            if is_block(tag) {
                assert!(
                    node.contains("data-ascribe-source="),
                    "{name}: a block has no anchor: {node}\n--- HTML\n{html}"
                );
            }
        }
    }
}

#[test]
fn an_anchor_names_the_element_it_applies_to() {
    // The anchor says `p`, but the next element is a list: it applies to
    // nothing, and is still removed.
    let html = render_site_html("<!--ascribe-anchor tag=\"p\" source=\"a.md:1-1\"-->\n- one\n");
    assert_eq!(html, "<ul>\n<li>one</li>\n</ul>\n");
    // Text after the anchor (a paragraph in a tight list) has no element.
    let html = render_site_html("- <!--ascribe-anchor tag=\"p\" source=\"a.md:1-1\"-->\n  one\n");
    assert_eq!(html, "<ul>\n<li>\none</li>\n</ul>\n");
    // A raw HTML block gets the anchor on its first tag.
    let html = render_site_html(
        "<!--ascribe-anchor tag=\"div\" source=\"a%20b.md:3-4\" via=\"c.md:1\"-->\n<div class=\"x\">\nhi\n</div>\n",
    );
    assert_eq!(
        html,
        "<div data-ascribe-source=\"a%20b.md:3-4\" data-ascribe-via=\"c.md:1\" class=\"x\">\nhi\n</div>\n"
    );
}

#[test]
fn a_list_anchor_gives_each_item_its_lines() {
    let html = render_site_html(
        "<!--ascribe-anchor tag=\"ol\" source=\"a.md:3-6\" items=\"3-4 5-6\"-->\n3. one\n   - inner\n4. two\n",
    );
    assert_eq!(
        html,
        "<ol data-ascribe-source=\"a.md:3-6\" start=\"3\">\n\
         <li data-ascribe-source=\"a.md:3-4\">one\n<ul>\n<li>inner</li>\n</ul>\n</li>\n\
         <li data-ascribe-source=\"a.md:5-6\">two</li>\n</ol>\n"
    );
    // When the items don't match the list, none of them gets one.
    let html = render_site_html(
        "<!--ascribe-anchor tag=\"ul\" source=\"a.md:3-6\" items=\"3-4\"-->\n- one\n- two\n",
    );
    assert_eq!(
        html,
        "<ul data-ascribe-source=\"a.md:3-6\">\n<li>one</li>\n<li>two</li>\n</ul>\n"
    );
}
