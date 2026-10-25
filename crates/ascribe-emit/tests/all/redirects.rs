//! `_ascribe/redirects.json` (output-layout contract §1.8), from
//! `ascribe.addresses` alone, and Netlify's files in `_ascribe/host/`
//! (§1.9) with `[consumer] host = "netlify"`.

#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use ascribe_core::FileId;
use ascribe_emit::SiteEmitter;
use ascribe_emit::site::{HOST_HEADERS_PATH, HOST_REDIRECTS_PATH, REDIRECTS_PATH};
use ascribe_resolve::{Layout, MemoryFs, Project};
use serde_json::{Value, json};

use crate::support::emit_build;

const MODEL: &str = "spec = \"0.1\"\n\
    [project]\ncontent-root = \"docs\"\n\
    [types.page]\n\
    [types.page.frontmatter]\ntitle = \"string\"\n\
    [consumer]\nsite = \"https://docs.example.com\"\nbase-path = \"/docs/\"\n\
    trailing-slash = \"never\"\nagents = true\n";

const PAGE: &str = "---\ntype: page\ntitle: Page\n---\n\n## Install\n@id: install\n\nText.\n";

fn project(model: &str, addresses: Option<&str>) -> Project {
    let model = ascribe_model::load_str(model, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout)
        .with_source("guides/setup.md", PAGE)
        .with_source("index.md", PAGE);
    if let Some(text) = addresses {
        fs = fs.with_file("ascribe.addresses", text);
    }
    Project::load(Arc::new(model), layout, &fs)
}

fn output(project: &Project) -> BTreeMap<String, String> {
    emit_build(
        Path::new("/nowhere"),
        project,
        "site",
        &SiteEmitter::new(project.model()),
    )
}

fn redirects(project: &Project) -> Value {
    let text = output(project)
        .remove(REDIRECTS_PATH)
        .expect("the site output has the redirects");
    serde_json::from_str(&text).expect("the redirects are JSON")
}

const ENTRIES: &str = "version = 1\n\n[moved]\n\
    \"guides/my-setup.md\" = \"guides/setup.md\"\n\
    \"guides/old.md\" = \"guides/setup.md#install\"\n\
    \"/docs/start\" = \"index.md\"\n\
    \"guides/setup.md#installing\" = \"guides/setup.md#install\"\n\
    \"Guides/Setup.md\" = \"guides/setup.md\"\n\
    \"gone.md\" = \"nowhere.md\"\n\
    \n[retired]\n\"guides/beta.md\" = \"The beta ended.\"\n";

#[test]
fn each_move_is_a_pair_for_the_page_and_its_markdown_version() {
    assert_eq!(
        redirects(&project(MODEL, Some(ENTRIES))),
        json!({
            "version": 1,
            "build": "site",
            "redirects": [
                { "from": "/docs/guides/my-setup", "to": "/docs/guides/setup", "status": 301, "entry": "guides/my-setup.md" },
                { "from": "/docs/guides/my-setup.md", "to": "/docs/guides/setup.md", "status": 301, "entry": "guides/my-setup.md" },
                { "from": "/docs/guides/old", "to": "/docs/guides/setup#install", "status": 301, "entry": "guides/old.md" },
                { "from": "/docs/guides/old.md", "to": "/docs/guides/setup.md", "status": 301, "entry": "guides/old.md" },
                { "from": "/docs/start", "to": "/docs", "status": 301, "entry": "/docs/start" },
            ],
            "gone": [
                { "path": "/docs/guides/beta", "reason": "The beta ended.", "entry": "guides/beta.md", "page": "/docs/_ascribe/gone/guides/beta" },
                { "path": "/docs/guides/beta.md", "reason": "The beta ended.", "entry": "guides/beta.md", "page": "/docs/_ascribe/gone/guides/beta" },
            ],
        })
    );
}

#[test]
fn without_the_file_the_lists_are_empty_and_nothing_is_written_for_a_host() {
    let output = output(&project(MODEL, None));
    let text = output.get(REDIRECTS_PATH).expect("the redirects");
    assert_eq!(
        serde_json::from_str::<Value>(text).expect("JSON"),
        json!({ "version": 1, "build": "site", "redirects": [], "gone": [] })
    );
    assert!(!output.contains_key(HOST_REDIRECTS_PATH));
    assert!(!output.contains_key(HOST_HEADERS_PATH));
}

#[test]
fn with_netlify_as_the_host_its_files_are_written_beside() {
    let model = MODEL.replace("agents = true\n", "agents = true\nhost = \"netlify\"\n");
    let output = output(&project(&model, Some(ENTRIES)));
    assert_eq!(
        output.get(HOST_REDIRECTS_PATH).map(String::as_str),
        Some(
            "/docs/guides/my-setup /docs/guides/setup 301\n\
             /docs/guides/my-setup.md /docs/guides/setup.md 301\n\
             /docs/guides/old /docs/guides/setup#install 301\n\
             /docs/guides/old.md /docs/guides/setup.md 301\n\
             /docs/start /docs 301\n\
             /docs/guides/beta /docs/_ascribe/gone/guides/beta 404\n\
             /docs/guides/beta.md /docs/_ascribe/gone/guides/beta 404\n"
        )
    );
    let headers = output.get(HOST_HEADERS_PATH).expect("the host's headers");
    assert!(
        headers.starts_with("/docs/guides/setup.md\n  Link: <https://docs.example.com/docs/guides/setup>; rel=\"canonical\"\n"),
        "{headers}"
    );
}

/// A retired address's page is under the build's base path, where a site
/// generator serves it: at the root with no base path, and an address from
/// the host's root outside the base path keeps its whole path there.
#[test]
fn a_retired_address_s_page_is_under_the_base_path() {
    let entries = "version = 1\n\n[retired]\n\"guides/beta.md\" = \"The beta ended.\"\n\
        \"/old/intro\" = \"Gone.\"\n\"index.md\" = \"Moved away.\"\n";
    let pages = |model: &str| -> Vec<(String, String)> {
        redirects(&project(model, Some(entries)))["gone"]
            .as_array()
            .expect("gone")
            .iter()
            .map(|g| {
                (
                    g["path"].as_str().expect("a path").to_owned(),
                    g["page"].as_str().expect("a page").to_owned(),
                )
            })
            .collect()
    };
    // `index.md` is published, so it isn't gone: only the others are.
    let pair = |a: &str, b: &str| (a.to_owned(), b.to_owned());
    assert_eq!(
        pages(MODEL),
        [
            pair("/docs/guides/beta", "/docs/_ascribe/gone/guides/beta"),
            pair("/docs/guides/beta.md", "/docs/_ascribe/gone/guides/beta"),
            pair("/old/intro", "/docs/_ascribe/gone/old/intro"),
        ]
    );
    let at_root = MODEL.replace("base-path = \"/docs/\"\n", "");
    assert_eq!(
        pages(&at_root),
        [
            pair("/guides/beta", "/_ascribe/gone/guides/beta"),
            pair("/guides/beta.md", "/_ascribe/gone/guides/beta"),
            pair("/old/intro", "/_ascribe/gone/old/intro"),
        ]
    );
}
