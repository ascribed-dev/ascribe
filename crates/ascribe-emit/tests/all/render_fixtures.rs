//! `render_site_html()` against every fixture in `tests/render/`: each
//! `input.md` renders to its `expected.html`, compared as parsed HTML, not as
//! text (`tests/render/README.md`).

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use crate::support;

use ascribe_emit::render_site_html;
use support::{first_difference, html_tree};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/render")
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
        let (want, got) = (html_tree(&expected, false), html_tree(&actual, false));
        if let Some((at, w, g)) = first_difference(&want, &got) {
            panic!(
                "{name}: the HTML differs at node {at}\nexpected: {w}\nactual:   {g}\n--- actual HTML\n{actual}"
            );
        }
        checked += 1;
    }
    assert!(checked >= 11, "only {checked} fixtures were found");
}

#[test]
fn the_comparison_notices_a_missing_attribute() {
    assert_ne!(
        html_tree("<img src=\"a\" width=\"1\" />", false),
        html_tree("<img src=\"a\" />", false)
    );
    assert_eq!(
        html_tree("<img width=\"1\" src=\"a\">", false),
        html_tree("<img src=\"a\" width=\"1\" />", false)
    );
}
