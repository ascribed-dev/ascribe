//! Fields read with `inline = "code"` (SPEC §7.2): each output carries the
//! plain text and the formatted form, and empty link text takes the
//! formatted title.

#![allow(clippy::expect_used, clippy::panic)]

use crate::support;

use std::path::Path;

use ascribe_emit::JsonEmitter;
use ascribe_resolve::Project;
use support::{FULL_MODEL, emit_build, memory_project, plain, site};

/// `reference/toml.md` has a title with code spans (the `reference` type sets
/// `inline = "code"` on `title`), and `index.md` links to it with no text.
fn project(title: &str) -> Project {
    memory_project(
        FULL_MODEL,
        &[
            (
                "index.md",
                "---\ntype: guide\ntitle: Home\n---\n\nSee [](reference/toml.md).\n",
            ),
            (
                "reference/toml.md",
                &format!("---\ntype: reference\ntitle: {title}\n---\n\nKeys.\n"),
            ),
        ],
    )
}

#[test]
fn the_site_output_writes_the_plain_title_and_the_formatted_one() {
    let p = project("'`ascribe.toml` & <keys>'");
    assert_eq!(
        site(&p, "site", "reference/toml.md"),
        "---\ntype: reference\ntitle: ascribe.toml & <keys>\nformatted:\n  title: <code>ascribe.toml</code> &amp; &lt;keys&gt;\nsource: reference/toml.md\n---\n\nKeys.\n"
    );
    assert_eq!(
        site(&p, "site", "index.md"),
        "---\ntype: guide\ntitle: Home\nsource: index.md\n---\n\nSee [`ascribe.toml` & \\<keys>](/docs/reference/toml).\n"
    );
}

#[test]
fn the_plain_output_heads_the_page_with_the_formatted_title() {
    let p = project("'`ascribe.toml`   *reference*'");
    assert_eq!(
        plain(&p, "site", "reference/toml.md"),
        "# `ascribe.toml` \\*reference\\*\n\nKeys.\n"
    );
    assert!(
        plain(&p, "site", "index.md").contains("See [`ascribe.toml`   \\*reference\\*]("),
        "{}",
        plain(&p, "site", "index.md")
    );
}

#[test]
fn the_json_output_lists_the_formatted_fields() {
    let p = project("'`ascribe.toml` reference'");
    let out = emit_build(Path::new("/nowhere"), &p, "site", &JsonEmitter);
    let doc: serde_json::Value = serde_json::from_str(&out["reference/toml.json"]).expect("JSON");
    assert_eq!(doc["title"], "ascribe.toml reference");
    assert_eq!(doc["frontmatter"]["title"], "ascribe.toml reference");
    assert_eq!(
        doc["formatted"]["title"],
        serde_json::json!([
            { "type": "code", "value": "ascribe.toml" },
            { "type": "text", "value": " reference" },
        ])
    );
    // A page whose fields don't set `inline` has no `formatted`.
    let home: serde_json::Value = serde_json::from_str(&out["index.json"]).expect("JSON");
    assert!(home.get("formatted").is_none());
}

#[test]
fn the_json_output_s_navigation_is_the_site_s_with_labels_in_pieces() {
    let p = project("'`ascribe.toml` reference'");
    let json = emit_build(Path::new("/nowhere"), &p, "site", &JsonEmitter);
    let json: serde_json::Value =
        serde_json::from_str(&json["_ascribe/nav.json"]).expect("the JSON output's navigation");
    let site = emit_build(
        Path::new("/nowhere"),
        &p,
        "site",
        &ascribe_emit::SiteEmitter::new(p.model()),
    );
    let site: serde_json::Value =
        serde_json::from_str(&site["_ascribe/nav.json"]).expect("the site's navigation");
    let toml = &json["root"]["sections"][0]["pages"][0];
    assert_eq!(toml["source"], "reference/toml.md");
    assert_eq!(toml["label"], "ascribe.toml reference");
    assert_eq!(
        toml["formatted"]["label"],
        serde_json::json!([
            { "type": "code", "value": "ascribe.toml" },
            { "type": "text", "value": " reference" },
        ])
    );
    assert_eq!(
        site["root"]["sections"][0]["pages"][0]["formatted"]["label"],
        "<code>ascribe.toml</code> reference"
    );
    // The same tree but for the formatted labels.
    let mut plain = json.clone();
    plain["root"]["sections"][0]["pages"][0]["formatted"] = serde_json::Value::Null;
    let mut site_plain = site.clone();
    site_plain["root"]["sections"][0]["pages"][0]["formatted"] = serde_json::Value::Null;
    assert_eq!(plain, site_plain);
}

#[test]
fn a_title_without_code_spans_still_has_a_formatted_form() {
    let p = project("Keys");
    assert_eq!(
        site(&p, "site", "reference/toml.md"),
        "---\ntype: reference\ntitle: Keys\nformatted:\n  title: Keys\nsource: reference/toml.md\n---\n\nKeys.\n"
    );
}
