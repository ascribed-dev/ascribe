//! A permalink's section in its page: what `_ascribe/help.json` renders for
//! it (output-layout contract §1.7).

#![allow(clippy::expect_used, clippy::panic)]

use crate::build_support;

use build_support::{blocks_summary, project, resolve, router};

const PAGE: &str = "---\ntype: page\ntitle: The page\n---\n\n";

/// The section of the heading `anchor` (or the page's opening) on `page`
/// in the `site` build, summarized.
fn section(files: &[(&str, &str)], page: &str, anchor: Option<&str>) -> Vec<String> {
    let p = project(files);
    let resolved = resolve(&p, page, "site");
    let section = resolved.section(anchor).expect("the page has the heading");
    blocks_summary(section.blocks)
}

#[test]
fn a_heading_s_section_ends_at_the_next_heading_of_its_level() {
    let body = format!(
        "{PAGE}## Install\n@id: install\n\nRun it.\n\n### Check\n\nIt runs.\n\n## Use\n\nUse it.\n"
    );
    assert_eq!(
        section(&[("a.md", &body)], "a.md", Some("install")),
        ["@id install", "p Run it.", "h3 Check", "p It runs."]
    );
}

#[test]
fn a_heading_s_section_ends_at_a_higher_heading() {
    let body = format!("{PAGE}## Install\n\n### Check\n\nIt runs.\n\n## Use\n\nUse it.\n");
    assert_eq!(
        section(&[("a.md", &body)], "a.md", Some("check")),
        ["p It runs."]
    );
}

#[test]
fn a_heading_s_section_ends_at_the_end_of_its_arm() {
    let body = format!(
        "{PAGE}## Install\n\n@variant {{pm=npm}}:\n### With npm\n\nRun npm.\n\
         @variant {{pm=pnpm}}:\nRun pnpm.\n@end\n\nAfter.\n"
    );
    let p = project(&[("a.md", &body)]);
    let page = resolve(&p, "a.md", "site");
    let section = page.section(Some("with-npm")).expect("the heading");
    assert_eq!(blocks_summary(section.blocks), ["p Run npm."]);
    // The group and its arm the heading is in, which a renderer's headings
    // there name.
    assert_eq!(section.arms.len(), 1);
    assert_eq!(
        section.arms[0].1.opener.name, "variant",
        "the arm that holds the heading"
    );
}

#[test]
fn a_heading_s_section_ends_at_the_end_of_the_page() {
    let body = format!("{PAGE}## Install\n\nRun it.\n\n```sh\nquill\n```\n");
    assert_eq!(
        section(&[("a.md", &body)], "a.md", Some("install")),
        ["p Run it.", "code quill"]
    );
}

#[test]
fn a_page_s_opening_is_its_blocks_before_the_first_heading() {
    let body = format!(
        "{PAGE}The opening.\n\nMore of it.\n\n@note\nA note with a heading.\n\n## Install\n\nRun it.\n"
    );
    let p = project(&[("a.md", &body)]);
    let page = resolve(&p, "a.md", "site");
    let opening = page.section(None).expect("an opening");
    assert!(opening.heading.is_none());
    assert_eq!(
        blocks_summary(opening.blocks)[..2],
        ["p The opening.", "p More of it."]
    );
    // A page that opens with a heading has an empty opening.
    let body = format!("{PAGE}## Install\n\nRun it.\n");
    assert!(section(&[("b.md", &body)], "b.md", None).is_empty());
}

#[test]
fn an_unknown_anchor_has_no_section() {
    let p = project(&[("a.md", &format!("{PAGE}## Install\n"))]);
    assert!(resolve(&p, "a.md", "site").section(Some("use")).is_none());
}

#[test]
fn a_fragment_s_heading_is_cut_from_the_page_its_permalink_names() {
    let p = project(&[
        (
            "a.md",
            &format!("{PAGE}## Setup\n\n@include: _keys.md\n\n## After\n\nDone.\n"),
        ),
        (
            "b.md",
            &format!("{PAGE}## Keys again\n\n@include: _keys.md\n"),
        ),
        (
            "_keys.md",
            "### Rotate keys\n@id {permalink=true}: rotate-keys\n\nRotate them.\n",
        ),
    ]);
    let build = p.model().build("site").expect("a build");
    let resolved = p.resolve_build(build, &router(&p));
    let permalinks = resolved.permalinks();
    let first = permalinks
        .iter()
        .find(|l| l.name == "rotate-keys")
        .expect("the permalink");
    let (page, section) = resolved.section(first).expect("its section");
    assert_eq!(page.path.as_str(), "a.md");
    assert_eq!(
        blocks_summary(section.blocks),
        ["@id rotate-keys", "p Rotate them."]
    );
    assert_eq!(
        section
            .heading
            .and_then(|h| h.heading.as_ref())
            .map(|h| h.text.as_str()),
        Some("Rotate keys")
    );
}
