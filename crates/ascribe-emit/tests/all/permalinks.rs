//! `_ascribe/permalinks.json`: the permalinks each build publishes, in the
//! site output (output-layout contract §1.2).

#![allow(clippy::expect_used, clippy::panic)]

use crate::support;

use std::path::Path;

use ascribe_emit::SiteEmitter;
use ascribe_emit::site::PERMALINKS_PATH;
use support::{emit_build, memory_project};

const MODEL: &str = "spec = \"0.1\"\n\
    [types.page]\ndefault = true\n[types.page.frontmatter]\ntitle = \"string\"\n\
    [dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n\
    [builds.site]\n\
    [builds.cloud]\nvariants = { deployment = \"cloud\" }\navailability = \"badge\"\n\
    [consumer]\nbase-path = \"/docs/\"\n";

fn list(model: &str, files: &[(&str, &str)], build: &str) -> serde_json::Value {
    let project = memory_project(model, files);
    let emitter = SiteEmitter::new(project.model());
    let text = emit_build(Path::new("/nowhere"), &project, build, &emitter)
        .remove(PERMALINKS_PATH)
        .expect("the site output has the list");
    serde_json::from_str(&text).expect("the list is JSON")
}

#[test]
fn each_page_and_marked_heading_is_listed_with_its_route_and_anchor() {
    let files = [
        (
            "reference/cli.md",
            "---\ntitle: CLI\n---\n\n## `ascribe check`\n@id {permalink=true}: cli-check\n\n\
             ## `ascribe build`\n@id: cli-build\n",
        ),
        (
            "guides/keys.md",
            "---\ntitle: Keys\npermalink: keys-guide\n---\n\n@include: _rotate.md\n",
        ),
        (
            "guides/_rotate.md",
            "## Rotate keys\n@id {permalink=true}: rotate-keys\n",
        ),
    ];
    assert_eq!(
        list(MODEL, &files, "site"),
        serde_json::json!({
            "version": 1,
            "path": "/docs/go/",
            "dimensions": { "deployment": ["cloud", "self-managed"] },
            "permalinks": {
                "cli-check": {
                    "route": "/docs/reference/cli/",
                    "fragment": "cli-check",
                    "source": "reference/cli.md",
                },
                "keys-guide": {
                    "route": "/docs/guides/keys/",
                    "source": "guides/keys.md",
                },
                "rotate-keys": {
                    "route": "/docs/guides/keys/",
                    "fragment": "rotate-keys",
                    "source": "guides/keys.md",
                },
            },
        })
    );
}

#[test]
fn a_build_lists_only_what_it_publishes() {
    let files = [(
        "self-managed.md",
        "---\ntitle: Server\nvariant:\n  deployment: self-managed\n---\n\n\
         ## Install\n@id {permalink=true}: install-server\n",
    )];
    assert_eq!(
        list(MODEL, &files, "site")["permalinks"]["install-server"]["route"],
        "/docs/self-managed/"
    );
    assert_eq!(
        list(MODEL, &files, "cloud")["permalinks"],
        serde_json::json!({})
    );
}

#[test]
fn a_build_with_no_permalinks_has_an_empty_table_at_the_path_it_sets() {
    let model = MODEL.replace("[consumer]\n", "[consumer]\npermalink-path = \"/go/\"\n");
    let files = [(
        "index.md",
        "---\ntitle: Home\n---\n\n## Hello\n@id: hello\n",
    )];
    assert_eq!(
        list(&model, &files, "site"),
        serde_json::json!({
            "version": 1,
            "path": "/go/",
            "dimensions": { "deployment": ["cloud", "self-managed"] },
            "permalinks": {},
        })
    );
}

#[test]
fn the_list_names_each_dimension_with_its_values_in_declaration_order() {
    let model = MODEL.replace(
        "[builds.site]",
        "[dimensions.platform]\nvalues = [\"macos\", \"linux\", \"windows\"]\n[builds.site]",
    );
    let files = [("index.md", "---\ntitle: Home\n---\n\nHello.\n")];
    assert_eq!(
        list(&model, &files, "cloud")["dimensions"],
        serde_json::json!({
            "deployment": ["cloud", "self-managed"],
            "platform": ["macos", "linux", "windows"],
        })
    );
}

#[test]
fn a_page_s_permalink_reaches_its_frontmatter_in_the_site_output() {
    let project = memory_project(
        MODEL,
        &[(
            "reference/cli.md",
            "---\ntitle: CLI\npermalink: cli-reference\n---\n\nText.\n",
        )],
    );
    let emitter = SiteEmitter::new(project.model());
    let page = emit_build(Path::new("/nowhere"), &project, "site", &emitter)
        .remove("reference/cli.md")
        .expect("the page is in the site output");
    assert!(
        page.starts_with("---\ntitle: CLI\npermalink: cli-reference\n---\n"),
        "{page}"
    );
}
