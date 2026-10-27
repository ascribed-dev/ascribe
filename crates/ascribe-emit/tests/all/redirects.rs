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

/// The pairs the build writes for `model`, with the setup guide's
/// permalink, and `entries` in `ascribe.addresses`: each `from`, `to`, and
/// `entry`.
fn pairs(model: &str, entries: &str) -> Vec<(String, String, String)> {
    let model = ascribe_model::load_str(model, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let fs = MemoryFs::new(&layout)
        .with_source(
            "guides/setup.md",
            "---\ntype: page\ntitle: Setup\npermalink: setup\n---\n\nText.\n",
        )
        .with_source("index.md", PAGE)
        .with_file("ascribe.addresses", &format!("version = 1\n{entries}"));
    let project = Project::load(Arc::new(model), layout, &fs);
    redirects(&project)["redirects"]
        .as_array()
        .expect("the pairs")
        .iter()
        .map(|r| {
            let field = |key: &str| r[key].as_str().expect("a string").to_owned();
            (field("from"), field("to"), field("entry"))
        })
        .collect()
}

fn pair(from: &str, to: &str, entry: &str) -> (String, String, String) {
    (from.to_owned(), to.to_owned(), entry.to_owned())
}

/// Every address under the former base path, each page's, its Markdown
/// version's, the permalink's, and `llms.txt`'s, forwards to today's.
#[test]
fn a_former_base_path_forwards_every_address_under_it() {
    let model = MODEL.replace("/docs/", "/handbook/");
    let b = "former.base-path";
    assert_eq!(
        pairs(&model, "[former]\nbase-path = [\"/docs/\"]\n"),
        [
            pair("/docs", "/handbook", b),
            pair("/docs/go/setup", "/handbook/go/setup", b),
            pair("/docs/guides/setup", "/handbook/guides/setup", b),
            pair("/docs/guides/setup.md", "/handbook/guides/setup.md", b),
            pair("/docs/index.md", "/handbook/index.md", b),
            pair("/docs/llms.txt", "/handbook/llms.txt", b),
        ]
    );
}

/// A second change composes with the first: an address under either former
/// base path forwards straight to today's, and a `[moved]` key's under
/// each.
#[test]
fn a_second_base_path_composes_with_the_first() {
    let model = MODEL.replace("/docs/", "/manual/");
    let found = pairs(
        &model,
        "[moved]\n\"guides/my-setup.md\" = \"guides/setup.md\"\n\n\
         [former]\nbase-path = [\"/docs/\", \"/handbook/\"]\n",
    );
    for old in ["/docs", "/handbook"] {
        for (from, to) in [
            ("", ""),
            ("/guides/setup", "/guides/setup"),
            ("/go/setup", "/go/setup"),
            ("/llms.txt", "/llms.txt"),
            ("/guides/my-setup", "/guides/setup"),
            ("/guides/my-setup.md", "/guides/setup.md"),
        ] {
            let from = format!("{old}{from}");
            let to = format!("/manual{to}");
            assert!(
                found.iter().any(|(f, t, _)| *f == from && *t == to),
                "{from} -> {to} in {found:?}"
            );
        }
    }
    // Every pair leads to an address today, never to another former one.
    assert!(found.iter().all(|(_, to, _)| to.starts_with("/manual")));
    assert!(found.contains(&pair(
        "/docs/guides/my-setup",
        "/manual/guides/setup",
        "guides/my-setup.md"
    )));
}

#[test]
fn a_former_permalink_path_forwards_each_name() {
    let model = MODEL.replace(
        "trailing-slash = \"never\"",
        "trailing-slash = \"never\"\npermalink-path = \"/go/\"",
    );
    assert_eq!(
        pairs(&model, "[former]\npermalink-path = [\"/docs/go/\"]\n"),
        [pair("/docs/go/setup", "/go/setup", "former.permalink-path")]
    );
    // Set apart, the permalinks' path doesn't move with the base path.
    let moved = model.replace("/docs/", "/handbook/");
    assert!(
        pairs(&moved, "[former]\nbase-path = [\"/docs/\"]\n")
            .iter()
            .all(|(from, _, _)| !from.contains("/go/"))
    );
}

/// `agents` turned off: each Markdown version forwards to its page, and
/// `llms.txt` to the page at the base path.
#[test]
fn agents_turned_off_forwards_each_markdown_version_to_its_page() {
    let model = MODEL.replace("agents = true\n", "agents = false\n");
    let a = "former.agents";
    assert_eq!(
        pairs(&model, "[former]\nagents = [true]\n"),
        [
            pair("/docs/guides/setup.md", "/docs/guides/setup", a),
            pair("/docs/index.md", "/docs", a),
            pair("/docs/llms.txt", "/docs", a),
        ]
    );
    // Without the record, nothing forwards.
    assert!(pairs(&model, "").is_empty());
}

/// With Netlify as the host, each permalink's address leads to its section
/// with a forced 301, after a rule for each dimension's query parameter
/// that serves the forwarding page, which takes the reader's choice from
/// it; an alias and a name moved to a page lead where they end; a retired
/// name's page is served with a 410. `redirects.json` doesn't list them.
#[test]
fn with_netlify_each_permalink_leads_to_its_section() {
    let model = MODEL
        .replace("agents = true\n", "agents = true\nhost = \"netlify\"\n")
        .replace("\"never\"", "\"always\"")
        + "[dimensions.pm]\nvalues = [\"npm\", \"pnpm\"]\n";
    let model = ascribe_model::load_str(&model, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let fs = MemoryFs::new(&layout)
        .with_source(
            "guides/setup.md",
            "---\ntype: page\ntitle: Setup\npermalink: setup\n---\n\n\
             ## Install\n@id {permalink=true}: install\n\nText.\n",
        )
        .with_source("index.md", PAGE)
        .with_file(
            "ascribe.addresses",
            "version = 1\n\n[moved]\n\"go:start\" = \"go:setup\"\n\
             \"go:keys\" = \"guides/setup.md#install\"\n\n\
             [retired]\n\"go:legacy\" = \"Removed in 4.0.\"\n",
        );
    let project = Project::load(Arc::new(model), layout, &fs);
    let output = output(&project);
    assert_eq!(
        output.get(HOST_REDIRECTS_PATH).map(String::as_str),
        Some(
            "/docs/go/install/ pm=:pm /docs/go/install/ 200!\n\
             /docs/go/install/ /docs/guides/setup/#install 301!\n\
             /docs/go/keys/ pm=:pm /docs/go/keys/ 200!\n\
             /docs/go/keys/ /docs/guides/setup/#install 301!\n\
             /docs/go/legacy/ /docs/go/legacy/ 410!\n\
             /docs/go/setup/ pm=:pm /docs/go/setup/ 200!\n\
             /docs/go/setup/ /docs/guides/setup/ 301!\n\
             /docs/go/start/ pm=:pm /docs/go/start/ 200!\n\
             /docs/go/start/ /docs/guides/setup/ 301!\n"
        )
    );
    assert_eq!(redirects(&project)["redirects"], json!([]));
}
