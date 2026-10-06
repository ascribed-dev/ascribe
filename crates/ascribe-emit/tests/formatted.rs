//! Fields read with `inline = "code"` (SPEC §7.2): each output carries the
//! plain text and the formatted form, and empty link text takes the
//! formatted title.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

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
                "---\ntitle: Home\n---\n\nSee [](reference/toml.md).\n",
            ),
            (
                "reference/toml.md",
                &format!("---\ntitle: {title}\n---\n\nKeys.\n"),
            ),
        ],
    )
}

#[test]
fn the_site_output_writes_the_plain_title_and_the_formatted_one() {
    let p = project("'`ascribe.toml` & <keys>'");
    assert_eq!(
        site(&p, "site", "reference/toml.md"),
        "---\ntitle: ascribe.toml & <keys>\nformatted:\n  title: <code>ascribe.toml</code> &amp; &lt;keys&gt;\n---\n\nKeys.\n"
    );
    assert_eq!(
        site(&p, "site", "index.md"),
        "---\ntitle: Home\n---\n\nSee [`ascribe.toml` & \\<keys>](/docs/reference/toml).\n"
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
fn a_title_without_code_spans_still_has_a_formatted_form() {
    let p = project("Keys");
    assert_eq!(
        site(&p, "site", "reference/toml.md"),
        "---\ntitle: Keys\nformatted:\n  title: Keys\n---\n\nKeys.\n"
    );
}
