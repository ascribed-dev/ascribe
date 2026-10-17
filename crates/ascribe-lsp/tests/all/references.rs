//! Find All References (`textDocument/references`) and the project's
//! inventory (`ascribe/inventory`), over an in-memory connection.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::support;

use std::path::Path;

use crate::support::{Client, Fixture};
use serde_json::{Value, json};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[phrases]
product = "Quill"
unused = "Nothing"

[features.sso]
name = "Single sign-on"
available = "cloud"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[glossary]
pages = ["glossary.md"]

[notes.security]
label = "Security"

[builds.site]
variants = "switch"
availability = "badge"
"#;

const INDEX: &str = "---\ntitle: Home\n---\n\n# Home\n\nSee [setup](guide.md#setup) and [the guide](guide.md). Use {product}.\n\nCreate an API key.\n";
const GUIDE: &str =
    "---\ntitle: Guide\n---\n\n# Guide\n\n@include: _setup.md\n\nJump to [install](#install).\n";
const SETUP: &str = "## Setup\n\n### Install\n\nRun it with {product}.\n";
const GLOSSARY: &str = "---\ntitle: Glossary\n---\n\n## API key\n@term\n\nA secret.\n";
const OTHER: &str = "---\ntitle: Other\n---\n\n@include: _setup.md#install\n\n@available: sso\nSSO only.\n\n@note {type=security}: Careful.\n";

fn project() -> Fixture {
    Fixture::new(
        MODEL,
        &[
            ("docs/index.md", INDEX),
            ("docs/guide.md", GUIDE),
            ("docs/_setup.md", SETUP),
            ("docs/other.md", OTHER),
            ("docs/glossary.md", GLOSSARY),
        ],
    )
}

fn started(f: &Fixture) -> Client {
    let mut client = Client::start(&f.root());
    client.settle();
    client
}

/// The position of the first `needle` in `text`, plus `delta` bytes.
fn at(text: &str, needle: &str, delta: usize) -> Value {
    let i = text.find(needle).unwrap_or_else(|| panic!("no {needle:?}")) + delta;
    let before = &text[..i];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    json!({ "line": line, "character": text[start..i].encode_utf16().count() })
}

/// The references at `position`, each as its file (relative to `docs/`) and
/// the text its range covers; `None` for a `null` answer.
fn references(
    client: &mut Client,
    f: &Fixture,
    file: &str,
    position: Value,
    declaration: bool,
) -> Option<Vec<(String, String)>> {
    references_in(client, f, &format!("docs/{file}"), position, declaration)
}

/// The references at `position` of `file`, a path from the project's folder,
/// as [`references`] has them.
fn references_in(
    client: &mut Client,
    f: &Fixture,
    file: &str,
    position: Value,
    declaration: bool,
) -> Option<Vec<(String, String)>> {
    let result = client
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": support::uri(&f.path(file)).as_str() },
                "position": position,
                "context": { "includeDeclaration": declaration },
            }),
        )
        .response_result
        .expect("the request succeeds");
    if result.is_null() {
        return None;
    }
    Some(
        result
            .as_array()
            .expect("a list")
            .iter()
            .map(|location| {
                let uri = location["uri"].as_str().expect("a URI");
                let file = FILES
                    .iter()
                    .find(|file| support::uri(&f.path(file)).as_str() == uri)
                    .unwrap_or_else(|| panic!("an unknown file: {uri}"));
                let text = std::fs::read_to_string(f.path(file)).expect("a file");
                let range: lsp_types::Range =
                    serde_json::from_value(location["range"].clone()).expect("a range");
                let name = file
                    .strip_prefix("docs/")
                    .map_or(format!("../{file}"), str::to_owned);
                (name, slice(&text, range))
            })
            .collect(),
    )
}

/// The files a reference can be in.
const FILES: [&str; 5] = [
    "docs/index.md",
    "docs/guide.md",
    "docs/_setup.md",
    "docs/other.md",
    "ascribe.toml",
];

/// The text of a single-line range (UTF-16 columns, ASCII text).
fn slice(text: &str, range: lsp_types::Range) -> String {
    let line = text.lines().nth(range.start.line as usize).unwrap_or("");
    if range.start.line != range.end.line {
        return line.to_owned();
    }
    line.get(range.start.character as usize..range.end.character as usize)
        .unwrap_or_default()
        .to_owned()
}

fn pairs(list: &[(&str, &str)]) -> Option<Vec<(String, String)>> {
    Some(
        list.iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect(),
    )
}

#[test]
fn a_heading_in_a_fragment_is_linked_through_the_page_that_includes_it() {
    let f = project();
    let mut client = started(&f);
    assert_eq!(
        references(&mut client, &f, "_setup.md", at(SETUP, "Setup", 2), false),
        pairs(&[("index.md", "[setup](guide.md#setup)")])
    );
    assert_eq!(
        references(&mut client, &f, "_setup.md", at(SETUP, "Install", 0), true),
        pairs(&[
            ("_setup.md", "### Install"),
            ("guide.md", "[install](#install)"),
            ("other.md", "@include: _setup.md#install"),
        ])
    );
}

#[test]
fn a_link_or_an_include_asks_about_what_it_names() {
    let f = project();
    let mut client = started(&f);
    // A link to a page: the page's references.
    assert_eq!(
        references(
            &mut client,
            &f,
            "index.md",
            at(INDEX, "the guide", 1),
            false
        ),
        pairs(&[
            ("index.md", "[setup](guide.md#setup)"),
            ("index.md", "[the guide](guide.md)"),
        ])
    );
    // A link to a heading: the heading's.
    assert_eq!(
        references(&mut client, &f, "index.md", at(INDEX, "#setup", 1), false),
        pairs(&[("index.md", "[setup](guide.md#setup)")])
    );
    // An include: the fragment's.
    assert_eq!(
        references(&mut client, &f, "guide.md", at(GUIDE, "_setup", 1), false),
        pairs(&[
            ("guide.md", "@include: _setup.md"),
            ("other.md", "@include: _setup.md#install"),
        ])
    );
}

#[test]
fn the_start_of_a_file_or_its_frontmatter_asks_about_the_file() {
    let f = project();
    let mut client = started(&f);
    let fragment = pairs(&[
        ("guide.md", "@include: _setup.md"),
        ("other.md", "@include: _setup.md#install"),
    ]);
    assert_eq!(
        references(
            &mut client,
            &f,
            "_setup.md",
            json!({ "line": 0, "character": 0 }),
            false
        ),
        fragment
    );
    assert_eq!(
        references(&mut client, &f, "guide.md", at(GUIDE, "title", 2), true),
        pairs(&[
            ("guide.md", ""),
            ("index.md", "[setup](guide.md#setup)"),
            ("index.md", "[the guide](guide.md)"),
        ])
    );
    // Prose that uses nothing has no references.
    assert_eq!(
        references(&mut client, &f, "guide.md", at(GUIDE, "Jump", 1), false),
        None
    );
}

#[test]
fn model_entries_are_found_where_pages_use_them() {
    let f = project();
    let mut client = started(&f);
    assert_eq!(
        references(&mut client, &f, "index.md", at(INDEX, "{product}", 2), true),
        pairs(&[
            ("../ascribe.toml", "product"),
            ("_setup.md", "{product}"),
            ("index.md", "{product}"),
        ])
    );
    assert_eq!(
        references(&mut client, &f, "other.md", at(OTHER, "sso", 0), false),
        pairs(&[("other.md", "sso")])
    );
    assert_eq!(
        references(&mut client, &f, "index.md", at(INDEX, "API key", 4), false),
        pairs(&[("index.md", "API key")])
    );
    assert_eq!(
        references(&mut client, &f, "other.md", at(OTHER, "@note", 1), false),
        pairs(&[("other.md", "security")])
    );
}

#[test]
fn an_entry_is_found_from_its_declaration_in_the_model() {
    let f = project();
    let mut client = started(&f);
    // A phrase's key, with its declaration first.
    assert_eq!(
        references_in(
            &mut client,
            &f,
            "ascribe.toml",
            at(MODEL, "product", 2),
            true
        ),
        pairs(&[
            ("../ascribe.toml", "product"),
            ("_setup.md", "{product}"),
            ("index.md", "{product}"),
        ])
    );
    // A table's header: a feature, a note type.
    assert_eq!(
        references_in(
            &mut client,
            &f,
            "ascribe.toml",
            at(MODEL, "[features.sso]", 0),
            false
        ),
        pairs(&[("other.md", "sso")])
    );
    assert_eq!(
        references_in(
            &mut client,
            &f,
            "ascribe.toml",
            at(MODEL, "[notes.security]", 1),
            false
        ),
        pairs(&[("other.md", "security")])
    );
    // A phrase nothing uses has none; a build, which pages don't name, and a
    // line that declares nothing, aren't asked about.
    assert_eq!(
        references_in(
            &mut client,
            &f,
            "ascribe.toml",
            at(MODEL, "unused", 0),
            false
        ),
        Some(Vec::new())
    );
    assert_eq!(
        references_in(
            &mut client,
            &f,
            "ascribe.toml",
            at(MODEL, "[builds.site]", 1),
            false
        ),
        None
    );
    assert_eq!(
        references_in(&mut client, &f, "ascribe.toml", at(MODEL, "spec", 0), false),
        None
    );
}

fn inventory(client: &mut Client, path: &Path) -> Value {
    client
        .request(
            "ascribe/inventory",
            json!({ "textDocument": { "uri": support::uri(path).as_str() } }),
        )
        .response_result
        .expect("the request succeeds")
}

#[test]
fn the_inventory_lists_pages_fragments_orphans_and_the_model() {
    let f = project();
    let mut client = started(&f);
    let r = inventory(&mut client, &f.path("docs/index.md"));
    assert_eq!(
        r["pages"],
        json!([
            { "path": "glossary.md", "title": "Glossary", "label": "Glossary", "navLabel": null, "type": "page", "incoming": 0, "builds": ["site"] },
            { "path": "guide.md", "title": "Guide", "label": "Guide", "navLabel": null, "type": "page", "incoming": 2, "builds": ["site"] },
            { "path": "index.md", "title": "Home", "label": "Home", "navLabel": null, "type": "page", "incoming": 0, "builds": ["site"] },
            { "path": "other.md", "title": "Other", "label": "Other", "navLabel": null, "type": "page", "incoming": 0, "builds": ["site"] },
        ])
    );
    assert_eq!(
        r["fragments"],
        json!([{ "path": "_setup.md", "includedBy": ["guide.md", "other.md"] }])
    );
    assert!(
        r["contentUri"]
            .as_str()
            .is_some_and(|u| u.ends_with("/docs")),
        "{}",
        r["contentUri"]
    );
    // An index page is never an orphan.
    assert_eq!(r["orphans"], json!(["glossary.md", "other.md"]));
    let uses = |kind: &str, key: &str| {
        r["model"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == kind && e["key"] == key)
            .unwrap_or_else(|| panic!("no {kind} {key}"))["uses"]
            .clone()
    };
    assert_eq!(uses("phrase", "product"), 2);
    assert_eq!(uses("phrase", "unused"), 0);
    assert_eq!(uses("feature", "sso"), 1);
    assert_eq!(uses("term", "api-key"), 1);
    assert_eq!(uses("note", "security"), 1);
    assert_eq!(uses("note", "tip"), 0);
    assert_eq!(uses("dimension", "deployment"), 0);
    assert_eq!(uses("build", "site"), Value::Null);
    let product = &r["model"][0];
    assert_eq!(
        product["declaration"],
        json!({ "start": { "line": 6, "character": 0 }, "end": { "line": 6, "character": 7 } })
    );
    // A built-in note type isn't declared in `ascribe.toml`.
    let note = r["model"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "note" && e["key"] == "note")
        .unwrap();
    assert_eq!(note["declaration"], Value::Null);
    // A dimension lists its values, each where `values` has it.
    let deployment = r["model"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "dimension" && e["key"] == "deployment")
        .unwrap();
    assert_eq!(
        deployment["values"],
        json!([
            { "value": "cloud", "label": null, "declaration": { "start": { "line": 14, "character": 11 }, "end": { "line": 14, "character": 16 } } },
            { "value": "self-managed", "label": null, "declaration": { "start": { "line": 14, "character": 20 }, "end": { "line": 14, "character": 32 } } },
        ])
    );
    assert!(product.get("values").is_none(), "{product}");
    // A content type counts its pages; the implicit one isn't declared.
    let page_type = r["model"].as_array().unwrap().last().unwrap();
    assert_eq!(
        page_type,
        &json!({ "kind": "type", "key": "page", "label": null, "uses": 4, "declaration": null, "page": null })
    );
}

#[test]
fn a_declared_content_type_counts_its_pages_and_opens_its_table() {
    let model = format!(
        "{MODEL}\n[types.guide]\npurpose = \"Teach a task\"\n\n[types.guide.frontmatter]\ntitle = \"string\"\n\n[types.reference]\n\n[types.reference.frontmatter]\ntitle = \"string\"\n"
    );
    let f = Fixture::new(
        &model,
        &[
            ("docs/a.md", "---\ntitle: A\ntype: guide\n---\n\n# A\n"),
            ("docs/b.md", "---\ntitle: B\ntype: guide\n---\n\n# B\n"),
            (
                "docs/glossary.md",
                "---\ntitle: Glossary\ntype: reference\n---\n\n## API key\n@term\n\nA secret.\n",
            ),
        ],
    );
    let mut client = started(&f);
    let r = inventory(&mut client, &f.path("docs/a.md"));
    let types: Vec<&Value> = r["model"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "type")
        .collect();
    assert_eq!(types.len(), 2, "{types:?}");
    assert_eq!(types[0]["key"], "guide");
    assert_eq!(types[0]["label"], "Teach a task");
    assert_eq!(types[0]["uses"], 2);
    assert_eq!(types[1]["key"], "reference");
    assert_eq!(types[1]["uses"], 1);
    let line = model.lines().position(|l| l == "[types.guide]").unwrap();
    assert_eq!(
        types[0]["declaration"],
        json!({ "start": { "line": line, "character": 0 }, "end": { "line": line, "character": 13 } })
    );
}

#[test]
fn counts_include_unsaved_edits() {
    let f = project();
    let mut client = started(&f);
    let other = f.path("docs/other.md");
    client.open(
        &other,
        1,
        &format!("{OTHER}\nBack to [the guide](guide.md) and {{product}}.\n"),
    );
    client.settle();
    let r = inventory(&mut client, &other);
    assert_eq!(r["pages"][1]["path"], "guide.md");
    assert_eq!(r["pages"][1]["incoming"], 3);
    assert_eq!(r["model"][0]["uses"], 3);
    assert_eq!(
        references(
            &mut client,
            &f,
            "guide.md",
            json!({ "line": 0, "character": 0 }),
            false
        )
        .map(|list| list.len()),
        Some(3)
    );
}

#[test]
fn any_file_of_the_project_asks_for_the_inventory_and_the_targets() {
    let f = project();
    f.write("README.md", "Not a page.\n");
    f.write("docs/nested/ascribe.toml", support::MODEL);
    f.write("docs/nested/docs/n.md", "# Nested\n");
    let mut client = started(&f);
    for file in ["ascribe.toml", "README.md"] {
        let r = inventory(&mut client, &f.path(file));
        assert_eq!(r["pages"].as_array().map(Vec::len), Some(4), "{file}");
        let targets = client
            .request(
                "ascribe/targets",
                json!({
                    "textDocument": { "uri": support::uri(&f.path(file)).as_str() },
                    "kinds": ["builds", "headings"],
                }),
            )
            .response_result
            .expect("the request succeeds");
        assert_eq!(
            targets["builds"],
            json!([{ "name": "site", "editor": true }])
        );
        // Paths are written from the content root.
        assert!(targets["headings"].as_array().unwrap().contains(&json!({
            "page": "guide.md", "text": "Setup", "id": "setup", "level": 2,
            "link": "guide.md#setup", "rootLink": "/guide.md#setup",
        })));
    }
    // A nested project's file, and a file outside the folder, are another
    // project's.
    for file in ["docs/nested/docs/n.md", "docs/nested/ascribe.toml"] {
        assert_eq!(
            inventory(&mut client, &f.path(file)),
            json!({ "pages": [], "fragments": [], "orphans": [], "navigation": null, "model": [] }),
            "{file}"
        );
    }
}

/// For every example project, each page's count of incoming uses and each
/// phrase's count of uses are the links, includes, and `{key}`s the resolver
/// recorded for it, counted from its edges and its index.
#[test]
fn counts_agree_with_the_resolver_for_every_example() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    for example in [
        "astro-site",
        "docs-repository",
        "getting-started",
        "monorepo/docs",
        "monorepo/handbook",
        "monorepo/handbook/pages/security",
        "quill",
    ] {
        let root = support::real_path(&examples.join(example));
        let config = root.join("ascribe.toml");
        let project = ascribe_check::Project::load(&config)
            .unwrap_or_else(|e| panic!("{example}: {e}"))
            .index();
        let mut client = Client::start(&root);
        client.settle();
        let r = inventory(&mut client, &config);
        let pages = r["pages"].as_array().expect("pages");
        assert!(!pages.is_empty(), "{example}");
        assert_eq!(pages.len(), project.pages().count(), "{example}");
        for page in pages {
            let path = ascribe_core::RelPath::parse(page["path"].as_str().unwrap()).unwrap();
            let links = project
                .links_to(&path)
                .iter()
                .filter(|site| site.file != path)
                .count();
            let includes = project.includers(&path).len();
            assert_eq!(page["incoming"], links + includes, "{example}: {path}");
        }
        let phrases = r["model"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "phrase");
        for phrase in phrases {
            let key = phrase["key"].as_str().unwrap();
            let recorded: usize = project
                .files()
                .map(|file| {
                    file.phrases
                        .iter()
                        .filter(|p| p.declared && p.phrase.key == key)
                        .count()
                })
                .sum();
            assert_eq!(phrase["uses"], recorded, "{example}: {{{key}}}");
        }
        assert_eq!(client.shutdown(), ascribe_lsp::Exit::Clean);
    }
}

/// What Find All References answers at `position`, as `(file, line,
/// column)`: the file from the project root, the line and column from 1.
fn places(
    client: &mut Client,
    f: &Fixture,
    file: &str,
    position: Value,
) -> Vec<(String, u32, u32)> {
    let result = client
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": support::uri(&f.path(&format!("docs/{file}"))).as_str() },
                "position": position,
                "context": { "includeDeclaration": false },
            }),
        )
        .response_result
        .expect("the request succeeds");
    result
        .as_array()
        .expect("a list")
        .iter()
        .map(|location| {
            let uri = location["uri"].as_str().expect("a URI");
            let file = FILES
                .iter()
                .find(|file| support::uri(&f.path(file)).as_str() == uri)
                .unwrap_or_else(|| panic!("an unknown file: {uri}"));
            let range: lsp_types::Range =
                serde_json::from_value(location["range"].clone()).expect("a range");
            // The text is ASCII, so a UTF-16 column is a character column.
            (
                (*file).to_owned(),
                range.start.line + 1,
                range.start.character + 1,
            )
        })
        .collect()
}

#[test]
fn ascribe_refs_agrees_with_find_all_references() {
    let f = project();
    let mut client = started(&f);
    let project = ascribe_check::Project::load(&f.root().join("ascribe.toml"))
        .expect("the project loads")
        .index();
    let cases: &[(&str, Value, &str)] = &[
        // A heading in a fragment, named through the page that includes it.
        ("_setup.md", at(SETUP, "Setup", 2), "guide.md#setup"),
        ("_setup.md", at(SETUP, "Install", 0), "_setup.md#install"),
        ("index.md", at(INDEX, "the guide", 1), "guide.md"),
        ("guide.md", at(GUIDE, "_setup", 1), "_setup.md"),
        ("index.md", at(INDEX, "{product}", 1), "phrase:product"),
        ("other.md", at(OTHER, "sso", 0), "feature:sso"),
        ("index.md", at(INDEX, "API key", 0), "term:api-key"),
        ("other.md", at(OTHER, "security", 0), "note:security"),
    ];
    for (file, position, target) in cases {
        let editor = places(&mut client, &f, file, position.clone());
        let asked = ascribe_query::refs::Asked::entry(target)
            .unwrap_or_else(|| ascribe_query::refs::Asked::path(target))
            .expect("a target");
        let refs = ascribe_query::refs(&project, &asked, target, usize::MAX, &[]);
        let command: Vec<(String, u32, u32)> = refs
            .places
            .iter()
            .map(|p| (p.file.clone(), p.line, p.column))
            .collect();
        assert!(refs.exists, "{target}");
        assert!(!command.is_empty(), "{target}: nothing uses it");
        assert_eq!(command, editor, "{target}");
    }
}
