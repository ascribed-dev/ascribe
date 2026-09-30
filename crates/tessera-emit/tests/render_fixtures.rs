//! `render_site_html()` against every fixture in `tests/render/`: each
//! `input.md` renders to its `expected.html`, compared as parsed HTML, not as
//! text (`tests/render/README.md`).

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use html5ever::tendril::TendrilSink;
use html5ever::{ParseOpts, QualName, local_name, ns, parse_fragment};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use tessera_emit::render_site_html;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/render")
}

/// Parses an HTML fragment in a `<body>` context and writes it as a
/// normalized tree, one node per line: element names, attributes as a sorted
/// set, text exactly, comments exactly; whitespace-only text is dropped except
/// inside `<pre>`.
fn tree(html: &str) -> Vec<String> {
    let dom = parse_fragment(
        RcDom::default(),
        ParseOpts::default(),
        QualName::new(None, ns!(html), local_name!("body")),
        Vec::new(),
        false,
    )
    .one(html);
    let mut lines = Vec::new();
    // The fragment's nodes are the children of the one root element.
    for root in dom.document.children.borrow().iter() {
        for child in root.children.borrow().iter() {
            walk(child, 0, false, &mut lines);
        }
    }
    lines
}

fn walk(node: &Handle, depth: usize, in_pre: bool, out: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    match &node.data {
        NodeData::Element { name, attrs, .. } => {
            let mut attributes: Vec<String> = attrs
                .borrow()
                .iter()
                .map(|a| format!("{}={:?}", a.name.local, a.value.to_string()))
                .collect();
            attributes.sort();
            let tag = name.local.to_string();
            out.push(format!("{pad}<{tag} {}>", attributes.join(" ")));
            let pre = in_pre || tag == "pre";
            for child in node.children.borrow().iter() {
                walk(child, depth + 1, pre, out);
            }
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            if in_pre || !text.trim().is_empty() {
                out.push(format!("{pad}{text:?}"));
            }
        }
        NodeData::Comment { contents } => out.push(format!("{pad}<!--{contents}-->")),
        _ => {}
    }
}

#[test]
fn every_fixture_renders_to_its_expected_html() {
    let dir = fixtures_dir();
    let mut checked = 0;
    for entry in fs::read_dir(&dir).expect("tests/render exists") {
        let path = entry.expect("a directory entry").path();
        let input = path.join("input.md");
        if !input.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .expect("a name")
            .to_string_lossy()
            .into_owned();
        let markdown = fs::read_to_string(&input).expect("input.md reads");
        let expected = fs::read_to_string(path.join("expected.html")).expect("expected.html reads");
        let actual = render_site_html(&markdown);
        let (want, got) = (tree(&expected), tree(&actual));
        if want != got {
            let at = want
                .iter()
                .zip(&got)
                .position(|(w, g)| w != g)
                .unwrap_or(want.len().min(got.len()));
            panic!(
                "{name}: the HTML differs at node {at}\nexpected: {:?}\nactual:   {:?}\n--- actual HTML\n{actual}",
                want.get(at),
                got.get(at)
            );
        }
        checked += 1;
    }
    assert!(checked >= 11, "only {checked} fixtures were found");
}

#[test]
fn the_comparison_notices_a_missing_attribute() {
    assert_ne!(
        tree("<img src=\"a\" width=\"1\" />"),
        tree("<img src=\"a\" />")
    );
    assert_eq!(
        tree("<img width=\"1\" src=\"a\">"),
        tree("<img src=\"a\" width=\"1\" />")
    );
}
