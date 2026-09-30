//! Scripted requests for the navigation features (completion, hover, go to
//! definition, document links, CodeLens, inlay hints), against a copy of
//! `examples/quill` with a features registry added.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::path::Path;

use lsp_types::{PositionEncodingKind, Uri};
use serde_json::{Value, json};
use support::{Client, Fixture, MODEL, Setup, uri};

const FEATURES: &str = r#"
[features.streaming-sync]
name = "Streaming sync"
available = "cloud, self-managed preview 3.4"
"#;

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir").flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy");
        }
    }
}

/// A copy of `examples/quill` with a feature registered.
fn quill() -> Fixture {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let f = Fixture::new(MODEL, &[]);
    copy_dir(&source, &f.root());
    let model = std::fs::read_to_string(f.path("ascribe.toml")).expect("model");
    f.write("ascribe.toml", &format!("{model}{FEATURES}"));
    f
}

fn read(f: &Fixture, rel: &str) -> String {
    std::fs::read_to_string(f.path(rel)).expect("a file")
}

/// The LSP position (UTF-16) of a byte offset.
fn position(text: &str, offset: usize) -> Value {
    let before = &text[..offset];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    let character: usize = text[start..offset].chars().map(char::len_utf16).sum();
    json!({ "line": line, "character": character })
}

/// The position of the start of the first `needle`, plus `extra` bytes.
fn at(text: &str, needle: &str, extra: usize) -> Value {
    let i = text.find(needle).unwrap_or_else(|| panic!("no {needle:?}"));
    position(text, i + extra)
}

fn text_doc(path: &Path) -> Value {
    json!({ "uri": uri(path).as_str() })
}

fn hover(client: &mut Client, path: &Path, pos: Value) -> Option<Value> {
    let r = client
        .request(
            "textDocument/hover",
            json!({ "textDocument": text_doc(path), "position": pos }),
        )
        .response_result
        .expect("a result");
    (!r.is_null()).then_some(r)
}

fn hover_text(client: &mut Client, path: &Path, pos: Value) -> String {
    hover(client, path, pos).expect("a hover")["contents"]["value"]
        .as_str()
        .expect("markup")
        .to_owned()
}

fn definition(client: &mut Client, path: &Path, pos: Value) -> Option<Value> {
    let r = client
        .request(
            "textDocument/definition",
            json!({ "textDocument": text_doc(path), "position": pos }),
        )
        .response_result
        .expect("a result");
    (!r.is_null()).then_some(r)
}

/// Opens `marked` (with `$0` where the cursor is) in place of the file and
/// completes there.
fn complete(client: &mut Client, path: &Path, marked: &str) -> Value {
    let i = marked.find("$0").expect("a cursor");
    let text = marked.replacen("$0", "", 1);
    client.open(path, 100, &text);
    client
        .request(
            "textDocument/completion",
            json!({ "textDocument": text_doc(path), "position": position(&text, i) }),
        )
        .response_result
        .expect("a result")
}

fn items(list: &Value) -> Vec<Value> {
    list["items"].as_array().cloned().unwrap_or_default()
}

fn labels(list: &Value) -> Vec<String> {
    items(list)
        .iter()
        .map(|i| i["label"].as_str().unwrap().to_owned())
        .collect()
}

fn item(list: &Value, label: &str) -> Value {
    items(list)
        .into_iter()
        .find(|i| i["label"] == label)
        .unwrap_or_else(|| panic!("no item {label:?} in {:?}", labels(list)))
}

fn new_text(item: &Value) -> &str {
    item["textEdit"]["newText"].as_str().expect("an edit")
}

// -- Capabilities -------------------------------------------------------------

#[test]
fn the_capabilities_are_advertised() {
    let f = quill();
    let client = Client::start(&f.root());
    let caps = &client.initialize_result["capabilities"];
    assert!(
        caps["completionProvider"]["triggerCharacters"]
            .as_array()
            .unwrap()
            .contains(&json!("@"))
    );
    assert_eq!(caps["hoverProvider"], true);
    assert_eq!(caps["definitionProvider"], true);
    assert!(caps["documentLinkProvider"].is_object());
    assert!(caps["codeLensProvider"].is_object());
    assert_eq!(caps["inlayHintProvider"], true);
    assert_eq!(
        caps["executeCommandProvider"]["commands"],
        json!(["ascribe.openFile"])
    );
}

// -- Completion -----------------------------------------------------------------

#[test]
fn directive_names_come_from_the_model() {
    let f = quill();
    let page = f.path("docs/quickstart.md");
    let mut client = Client::start(&f.root());
    let list = complete(&mut client, &page, "# T\n\n@no$0\n");
    let names = labels(&list);
    for want in [
        "id",
        "include",
        "variant",
        "available",
        "note",
        "steps",
        "details",
        "end",
    ] {
        assert!(names.contains(&want.to_owned()), "{names:?}");
    }
    let note = item(&list, "note");
    assert_eq!(note["detail"], "Callout.");
    assert_eq!(new_text(&note), "note");
    // The edit replaces what was typed after the `@`.
    assert_eq!(
        note["textEdit"]["range"],
        json!({ "start": { "line": 2, "character": 1 }, "end": { "line": 2, "character": 3 } })
    );
    // Also inside a list item and a block quote.
    assert!(labels(&complete(&mut client, &page, "1. @no$0\n")).contains(&"note".to_owned()));
    assert!(labels(&complete(&mut client, &page, "> @in$0\n")).contains(&"include".to_owned()));
    // Not in the middle of a sentence.
    assert!(items(&complete(&mut client, &page, "mail me@no$0\n")).is_empty());
}

#[test]
fn attribute_keys_and_values_come_from_the_schema() {
    let f = quill();
    let page = f.path("docs/quickstart.md");
    let mut client = Client::start(&f.root());

    let keys = complete(&mut client, &page, "@note {$0\n");
    assert_eq!(labels(&keys), ["type"]);
    assert_eq!(new_text(&item(&keys, "type")), "type=");
    // A key already written isn't offered again.
    let none = complete(&mut client, &page, "@note {type=tip, $0\n");
    assert!(items(&none).is_empty());

    let values = complete(&mut client, &page, "@note {type=$0\n");
    for want in ["tip", "warning"] {
        assert!(labels(&values).contains(&want.to_owned()));
    }
    let partial = complete(&mut client, &page, "@note {type=ti$0}\n");
    assert_eq!(
        item(&partial, "tip")["textEdit"]["range"],
        json!({ "start": { "line": 0, "character": 12 }, "end": { "line": 0, "character": 14 } })
    );

    // `@variant` takes the model's dimensions, and the values show their labels.
    let dims = complete(&mut client, &page, "@variant {$0\n");
    assert_eq!(labels(&dims), ["pm", "deployment"]);
    assert_eq!(item(&dims, "deployment")["detail"], "Deployment");
    let dvalues = complete(&mut client, &page, "@variant {deployment=$0\n");
    assert_eq!(labels(&dvalues), ["cloud", "self-managed"]);
    assert_eq!(item(&dvalues, "cloud")["detail"], "Quill Cloud");
    let pm = complete(&mut client, &page, "@variant {pm=$0\n");
    assert_eq!(item(&pm, "yarn")["detail"], "Yarn");
    // A value set doesn't repeat a member.
    let set = complete(&mut client, &page, "@variant {pm=npm|$0\n");
    assert_eq!(labels(&set), ["pnpm", "yarn"]);
    // A boolean.
    let boolean = complete(&mut client, &page, "@include {heading=$0\n");
    assert_eq!(labels(&boolean), ["true", "false"]);
    // An image's attributes come from the model too.
    let image = complete(&mut client, &page, "![a](p.png){$0\n");
    assert_eq!(labels(&image), ["width", "height"]);
}

#[test]
fn availability_completes_targets_states_and_features() {
    let f = quill();
    let page = f.path("docs/quickstart.md");
    let mut client = Client::start(&f.root());

    let targets = complete(&mut client, &page, "@available: $0\n");
    let names = labels(&targets);
    for want in [
        "cloud",
        "self-managed",
        "deployment",
        "pm",
        "streaming-sync",
    ] {
        assert!(names.contains(&want.to_owned()), "{names:?}");
    }
    assert_eq!(item(&targets, "cloud")["detail"], "Quill Cloud");
    assert_eq!(
        item(&targets, "streaming-sync")["documentation"]["value"],
        "Available: Quill Cloud (GA); self-managed (preview, 3.4+)"
    );
    // A feature key is a whole spec, so it isn't offered after a comma.
    let second = complete(&mut client, &page, "@available: cloud, $0\n");
    assert!(!labels(&second).contains(&"streaming-sync".to_owned()));
    assert!(labels(&second).contains(&"self-managed".to_owned()));
    // After a target: lifecycle states.
    let states = complete(&mut client, &page, "@available: cloud, self-managed $0\n");
    let names = labels(&states);
    for want in ["ga", "preview", "beta", "deprecated"] {
        assert!(names.contains(&want.to_owned()), "{names:?}");
    }
    assert!(items(&complete(&mut client, &page, "@available: cloud ga 3$0\n")).is_empty());
    // In a history.
    let history = complete(
        &mut client,
        &page,
        "@available: self-managed (preview 3.3, $0\n",
    );
    assert!(labels(&history).contains(&"ga".to_owned()));

    // The same in the frontmatter.
    let front = complete(&mut client, &page, "---\ntitle: T\navailable: cl$0\n---\n");
    assert!(labels(&front).contains(&"cloud".to_owned()));
    assert!(items(&complete(&mut client, &page, "---\ntitle: T$0\n---\n")).is_empty());
}

#[test]
fn phrases_show_their_values() {
    let f = quill();
    let page = f.path("docs/quickstart.md");
    let mut client = Client::start(&f.root());
    let list = complete(&mut client, &page, "Use {pro$0 here.\n");
    assert_eq!(labels(&list), ["product", "cloud", "version", "api"]);
    let product = item(&list, "product");
    assert_eq!(product["detail"], "Quill");
    assert_eq!(new_text(&product), "product}");
    // An existing closing brace stays.
    let closed = complete(&mut client, &page, "Use {pro$0} here.\n");
    assert_eq!(new_text(&item(&closed, "product")), "product");
    // Not in a code fence, unless it opted in to phrases.
    assert!(items(&complete(&mut client, &page, "```sh\necho {pro$0\n```\n")).is_empty());
    let opted = complete(&mut client, &page, "```yaml phrases=true\nv: {ver$0\n```\n");
    assert!(labels(&opted).contains(&"version".to_owned()));
    // Not after the braces are closed, and not in inline code.
    assert!(items(&complete(&mut client, &page, "{product} x$0\n")).is_empty());
    assert!(items(&complete(&mut client, &page, "`{pro$0\n")).is_empty());
    assert!(items(&complete(&mut client, &page, "Use ``{pro$0}`` here.\n")).is_empty());
    assert!(items(&complete(&mut client, &page, "Use ``{pro$0\n")).is_empty());
}

#[test]
fn include_paths_and_ids() {
    let f = quill();
    let page = f.path("docs/quickstart.md");
    let mut client = Client::start(&f.root());
    let files = complete(&mut client, &page, "@include: $0\n");
    let names = labels(&files);
    assert!(names.contains(&"_fragments/prerequisites.md".to_owned()));
    assert!(names.contains(&"keys.md".to_owned()));
    // Not the file itself.
    assert!(!names.contains(&"quickstart.md".to_owned()));
    let prerequisites = item(&files, "_fragments/prerequisites.md");
    assert_eq!(new_text(&prerequisites), "_fragments/prerequisites.md");
    assert_eq!(prerequisites["detail"], "Fragment");

    // Paths are relative to the file they're written in.
    let fragment = f.path("docs/_fragments/prerequisites.md");
    let from_fragment = complete(&mut client, &fragment, "@include: ../ke$0\n");
    assert_eq!(new_text(&item(&from_fragment, "../keys.md")), "../keys.md");
    let rooted = complete(&mut client, &fragment, "@include: /ke$0\n");
    assert_eq!(new_text(&item(&rooted, "/keys.md")), "/keys.md");

    // After `#`: the target's heading ids.
    let ids = complete(&mut client, &page, "@include: keys.md#$0\n");
    assert_eq!(labels(&ids), ["Create a key", "Rotate keys"]);
    assert_eq!(new_text(&item(&ids, "Rotate keys")), "rotate-keys");
    let filtered = complete(&mut client, &page, "@include: keys.md#rot$0\n");
    assert_eq!(labels(&filtered), ["Rotate keys"]);
}

#[test]
fn link_completion_finds_a_heading_by_title() {
    let f = quill();
    let page = f.path("docs/quickstart.md");
    let mut client = Client::start(&f.root());

    let list = complete(&mut client, &page, "See [x](rot$0\n");
    let rotate = item(&list, "Rotate keys");
    // The file path (and the id) from the current file; the path is the detail.
    assert_eq!(new_text(&rotate), "keys.md#rotate-keys");
    assert_eq!(rotate["detail"], "keys.md#rotate-keys");
    assert_eq!(rotate["labelDetails"]["description"], "API keys");
    assert_eq!(
        rotate["textEdit"]["range"],
        json!({ "start": { "line": 0, "character": 8 }, "end": { "line": 0, "character": 11 } })
    );

    // From a fragment in a subdirectory, the path climbs out of it.
    let fragment = f.path("docs/_fragments/prerequisites.md");
    let from_fragment = complete(&mut client, &fragment, "See [x](rot$0\n");
    assert_eq!(
        new_text(&item(&from_fragment, "Rotate keys")),
        "../keys.md#rotate-keys"
    );

    // Pages are found by their title, and a fragment is never offered.
    let pages = complete(&mut client, &page, "See [x](install$0\n");
    let install = item(&pages, "Install the Quill agent");
    assert_eq!(new_text(&install), "install-agent.md");
    assert_eq!(install["detail"], "install-agent.md");
    assert!(
        !labels(&complete(&mut client, &page, "[x](prereq$0\n"))
            .iter()
            .any(|l| l.contains("prerequisites.md"))
    );

    // Headings of the current file are `#id` links.
    let own = complete(&mut client, &page, "## Local heading\n\nSee [x](local$0\n");
    assert_eq!(new_text(&item(&own, "Local heading")), "#local-heading");

    // After `#`: the ids of the file already named.
    let ids = complete(&mut client, &page, "See [x](keys.md#$0\n");
    assert_eq!(new_text(&item(&ids, "Create a key")), "create-key");
    // A fragment's ids aren't linkable.
    let fragment_ids = complete(
        &mut client,
        &page,
        "See [x](_fragments/prerequisites.md#$0\n",
    );
    assert!(items(&fragment_ids).is_empty());
    // An image's source isn't a page.
    assert!(items(&complete(&mut client, &page, "![x](rot$0\n")).is_empty());
    // A reference definition is a destination.
    let definition = complete(&mut client, &page, "[ref]: rot$0\n");
    assert_eq!(
        new_text(&item(&definition, "Rotate keys")),
        "keys.md#rotate-keys"
    );
}

#[test]
fn heading_completion_reports_truncation_for_links_and_includes() {
    let headings = (0..120)
        .map(|number| format!("## Heading {number}\n"))
        .collect::<String>();
    let f = Fixture::new(
        MODEL,
        &[("docs/page.md", ""), ("docs/target.md", &headings)],
    );
    let page = f.path("docs/page.md");
    let mut client = Client::start(&f.root());

    for marked in ["[x](target.md#$0\n", "@include: target.md#$0\n"] {
        let result = complete(&mut client, &page, marked);
        assert_eq!(result["items"].as_array().unwrap().len(), 100);
        assert_eq!(result["isIncomplete"], true);
    }
}

#[test]
fn identical_titles_are_told_apart_by_path() {
    let page = |t: &str| format!("---\ntitle: {t}\n---\n\nBody.\n");
    let f = Fixture::new(
        MODEL,
        &[
            ("docs/a/setup.md", &page("Setup")),
            ("docs/b/setup.md", &page("Setup")),
            ("docs/index.md", &page("Home")),
        ],
    );
    let mut client = Client::start(&f.root());
    let list = complete(&mut client, &f.path("docs/index.md"), "[x](Setup$0\n");
    let both: Vec<Value> = items(&list)
        .into_iter()
        .filter(|i| i["label"] == "Setup")
        .collect();
    assert_eq!(both.len(), 2);
    assert_eq!(both[0]["detail"], "a/setup.md");
    assert_eq!(new_text(&both[0]), "a/setup.md");
    assert_eq!(both[1]["detail"], "b/setup.md");
    assert_eq!(new_text(&both[1]), "b/setup.md");
}

#[test]
fn a_long_list_is_searched_and_cut() {
    let mut files: Vec<(String, String)> = (0..250)
        .map(|i| {
            (
                format!("docs/p{i:03}.md"),
                format!("---\ntitle: Page {i}\n---\n\n## Heading {i}\n"),
            )
        })
        .collect();
    files.push((
        "docs/index.md".to_owned(),
        "---\ntitle: Home\n---\n".to_owned(),
    ));
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let f = Fixture::new(MODEL, &refs);
    let mut client = Client::start(&f.root());
    let index = f.path("docs/index.md");
    let all = complete(&mut client, &index, "[x]($0\n");
    assert_eq!(all["isIncomplete"], true);
    assert_eq!(items(&all).len(), 100);
    let narrowed = complete(&mut client, &index, "[x](Heading 24$0\n");
    assert_eq!(narrowed["isIncomplete"], false);
    assert_eq!(items(&narrowed).len(), 11);
}

#[test]
fn completion_columns_follow_the_negotiated_encoding() {
    let f = quill();
    let page = f.path("docs/quickstart.md");
    let text = "😀 See [x](rot";
    for (encodings, start) in [(None, 11), (Some(vec![PositionEncodingKind::UTF8]), 13)] {
        let mut client = Client::start_with(
            &f.root(),
            Setup {
                encodings,
                ..Setup::default()
            },
        );
        client.open(&page, 1, text);
        let end = start + 3;
        let list = client
            .request(
                "textDocument/completion",
                json!({ "textDocument": text_doc(&page), "position": { "line": 0, "character": end } }),
            )
            .response_result
            .expect("a result");
        assert_eq!(
            item(&list, "Rotate keys")["textEdit"]["range"],
            json!({ "start": { "line": 0, "character": start }, "end": { "line": 0, "character": end } })
        );
    }
}

// -- Hover ----------------------------------------------------------------------

#[test]
fn hovering_a_link_shows_its_path_and_a_preview() {
    let f = quill();
    let quickstart = f.path("docs/quickstart.md");
    let text = read(&f, "docs/quickstart.md");
    let mut client = Client::start(&f.root());
    let value = hover_text(
        &mut client,
        &quickstart,
        at(&text, "install the agent](", 3),
    );
    assert!(value.contains("**Install the Quill agent**"), "{value}");
    assert!(value.contains("`docs/install-agent.md`"), "{value}");
    // The preview is the first paragraph, with declared phrases replaced.
    assert!(
        value.contains("The Quill agent watches your docs repository and syncs changes to Quill."),
        "{value}"
    );
    let range = hover(
        &mut client,
        &quickstart,
        at(&text, "install the agent](", 3),
    )
    .unwrap()["range"]
        .clone();
    assert_eq!(range["start"]["line"], range["end"]["line"]);

    // A link to a heading names the heading and its section.
    let keys = read(&f, "docs/keys.md");
    let value = hover_text(
        &mut client,
        &f.path("docs/keys.md"),
        at(&keys, "install-agent.md#install-agent", 3),
    );
    assert!(value.contains("**Install the agent**"), "{value}");
    assert!(
        value.contains("`docs/install-agent.md#install-agent`"),
        "{value}"
    );
    assert!(value.contains("Install the agent package:"), "{value}");

    // An image is a file.
    let value = hover_text(&mut client, &quickstart, at(&text, "playground.png", 2));
    assert_eq!(value, "File `docs/playground.png`");
    // Whitespace has no hover.
    assert!(
        hover(
            &mut client,
            &quickstart,
            json!({ "line": 3, "character": 0 })
        )
        .is_none()
    );
}

#[test]
fn hovering_a_missing_target_says_so() {
    let f = quill();
    let page = f.path("docs/quickstart.md");
    let mut client = Client::start(&f.root());
    client.open(&page, 1, "See [x](nope.md) and [y](keys.md#no-such-id).\n");
    let value = hover_text(&mut client, &page, json!({ "line": 0, "character": 6 }));
    assert_eq!(value, "**Not found:** `nope.md`");
    let value = hover_text(&mut client, &page, json!({ "line": 0, "character": 30 }));
    assert!(
        value.contains("No heading with the id `no-such-id`"),
        "{value}"
    );
}

#[test]
fn hovering_an_include_shows_the_fragment() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let text = read(&f, "docs/install-agent.md");
    let mut client = Client::start(&f.root());
    let value = hover_text(
        &mut client,
        &page,
        at(&text, "_fragments/prerequisites.md", 4),
    );
    assert!(
        value.contains("`docs/_fragments/prerequisites.md`"),
        "{value}"
    );
    assert!(value.contains("a fragment"), "{value}");
    assert!(
        value.contains("Before you install the agent, make sure you have:"),
        "{value}"
    );
}

#[test]
fn hovering_a_phrase_shows_its_value() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let text = read(&f, "docs/install-agent.md");
    let mut client = Client::start(&f.root());
    let value = hover_text(&mut client, &page, at(&text, "{version}", 2));
    assert_eq!(value, "**{version}**\n\n3.4.1");
    // An undeclared one says it's literal.
    client.open(&page, 2, "Use {nope} here.\n");
    let value = hover_text(&mut client, &page, json!({ "line": 0, "character": 6 }));
    assert!(value.contains("isn't a declared phrase"), "{value}");
}

#[test]
fn hovering_availability_shows_it_in_words() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let text = read(&f, "docs/install-agent.md");
    let mut client = Client::start(&f.root());
    let value = hover_text(
        &mut client,
        &page,
        at(&text, "@available: cloud, self-managed preview 3.4", 20),
    );
    assert_eq!(
        value,
        "Available: Quill Cloud (GA); self-managed (preview, 3.4+)"
    );
    // The frontmatter's `available`.
    let value = hover_text(
        &mut client,
        &page,
        at(&text, "available: cloud, self-managed preview 3.3", 14),
    );
    assert_eq!(
        value,
        "Available: Quill Cloud (GA); self-managed (preview, 3.3+)"
    );
    // A feature key shows the feature's name and its spec.
    client.open(&page, 2, "## S\n@available: streaming-sync\n");
    let value = hover_text(&mut client, &page, json!({ "line": 1, "character": 16 }));
    assert_eq!(
        value,
        "**Streaming sync** (feature `streaming-sync`)\n\nAvailable: Quill Cloud (GA); self-managed (preview, 3.4+)"
    );
}

#[test]
fn hovering_a_directive_describes_it_from_its_schema() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let mut client = Client::start(&f.root());
    client.open(&page, 1, "@include {heading=false}: keys.md#create-key\n");
    let value = hover_text(&mut client, &page, json!({ "line": 0, "character": 3 }));
    assert!(
        value.starts_with("**@include**, a built-in directive"),
        "{value}"
    );
    assert!(value.contains("Transclude a file or a region."), "{value}");
    assert!(value.contains("Primary: an identifier"), "{value}");
    let value = hover_text(&mut client, &page, json!({ "line": 0, "character": 12 }));
    assert!(value.contains("`heading` (boolean"), "{value}");
    assert!(value.contains("default `true`"), "{value}");
}

// -- Go to definition -------------------------------------------------------------

fn location(v: &Value) -> (String, u64, u64) {
    (
        v["uri"].as_str().expect("a uri").to_owned(),
        v["range"]["start"]["line"].as_u64().unwrap(),
        v["range"]["start"]["character"].as_u64().unwrap(),
    )
}

#[test]
fn links_and_includes_go_to_their_targets() {
    let f = quill();
    let install = f.path("docs/install-agent.md");
    let text = read(&f, "docs/install-agent.md");
    let mut client = Client::start(&f.root());

    // A link to a heading goes to the heading.
    let keys = read(&f, "docs/keys.md");
    let heading_line = keys.lines().position(|l| l == "## Rotate keys").unwrap() as u64;
    let got = definition(&mut client, &install, at(&text, "keys.md#rotate-keys", 2)).unwrap();
    assert_eq!(
        location(&got),
        (
            uri(&f.path("docs/keys.md")).as_str().to_owned(),
            heading_line,
            0
        )
    );

    // A link to a page goes to the top of the file.
    let quickstart = f.path("docs/quickstart.md");
    let qs = read(&f, "docs/quickstart.md");
    let got = definition(&mut client, &quickstart, at(&qs, "install-agent.md)", 2)).unwrap();
    assert_eq!(location(&got), (uri(&install).as_str().to_owned(), 0, 0));

    // An include goes to the fragment.
    let got = definition(
        &mut client,
        &install,
        at(&text, "_fragments/prerequisites.md", 3),
    )
    .unwrap();
    assert_eq!(
        location(&got),
        (
            uri(&f.path("docs/_fragments/prerequisites.md"))
                .as_str()
                .to_owned(),
            0,
            0
        )
    );

    // An include of a section goes to its heading; an image to its file.
    client.open(
        &install,
        2,
        "@include: keys.md#create-key\n\n![p](playground.png)\n",
    );
    let create = keys.lines().position(|l| l == "## Create a key").unwrap() as u64;
    let got = definition(&mut client, &install, json!({ "line": 0, "character": 14 })).unwrap();
    assert_eq!(location(&got).1, create);
    let got = definition(&mut client, &install, json!({ "line": 2, "character": 6 })).unwrap();
    assert_eq!(
        location(&got).0,
        uri(&f.path("docs/playground.png")).as_str()
    );

    // A missing target has no definition; nor does plain text.
    client.open(&install, 3, "See [x](nope.md).\n\nText.\n");
    assert!(definition(&mut client, &install, json!({ "line": 0, "character": 6 })).is_none());
    assert!(definition(&mut client, &install, json!({ "line": 2, "character": 1 })).is_none());
}

#[test]
fn an_id_goes_to_its_heading() {
    let f = quill();
    let keys = f.path("docs/keys.md");
    let text = read(&f, "docs/keys.md");
    let mut client = Client::start(&f.root());
    // On the `@id` primary, the heading it names.
    let got = definition(&mut client, &keys, at(&text, "@id: rotate-keys", 8)).unwrap();
    let line = text.lines().position(|l| l == "## Rotate keys").unwrap() as u64;
    assert_eq!(location(&got), (uri(&keys).as_str().to_owned(), line, 0));
    // A `#id` link in the file goes to its own heading.
    client.open(&keys, 2, "## Rotate\n@id: r\n\nSee [](#r).\n");
    let got = definition(&mut client, &keys, json!({ "line": 3, "character": 6 })).unwrap();
    assert_eq!(location(&got), (uri(&keys).as_str().to_owned(), 0, 0));
}

#[test]
fn phrases_and_features_go_to_ascribe_toml() {
    let f = quill();
    let install = f.path("docs/install-agent.md");
    let text = read(&f, "docs/install-agent.md");
    let model = read(&f, "ascribe.toml");
    let mut client = Client::start(&f.root());
    let toml = uri(&f.path("ascribe.toml")).as_str().to_owned();

    let got = definition(&mut client, &install, at(&text, "{version}", 2)).unwrap();
    let line = model
        .lines()
        .position(|l| l.starts_with("version = "))
        .unwrap() as u64;
    assert_eq!(location(&got), (toml.clone(), line, 0));
    // `{cloud}` is a phrase named like a dimension value.
    let got = definition(&mut client, &install, at(&text, "{cloud}", 2)).unwrap();
    let line = model
        .lines()
        .position(|l| l.starts_with("cloud = "))
        .unwrap() as u64;
    assert_eq!(location(&got), (toml.clone(), line, 0));

    client.open(&install, 2, "## S\n@available: streaming-sync\n");
    let got = definition(&mut client, &install, json!({ "line": 1, "character": 16 })).unwrap();
    let line = model
        .lines()
        .position(|l| l == "[features.streaming-sync]")
        .unwrap() as u64;
    assert_eq!(location(&got), (toml, line, 0));
    // A dimension value isn't in the registry.
    client.open(&install, 3, "## S\n@available: cloud\n");
    assert!(definition(&mut client, &install, json!({ "line": 1, "character": 15 })).is_none());
}

// -- Document links and CodeLens -----------------------------------------------------

#[test]
fn every_link_and_include_is_a_document_link() {
    let f = quill();
    let install = f.path("docs/install-agent.md");
    let mut client = Client::start(&f.root());
    client.open(
        &install,
        1,
        "See [k](keys.md#rotate-keys), [w](https://example.com/x), ![p](playground.png), [n](nope.md).\n\n@include: _fragments/prerequisites.md\n",
    );
    let links = client
        .request(
            "textDocument/documentLink",
            json!({ "textDocument": text_doc(&install) }),
        )
        .response_result
        .expect("a result");
    let links = links.as_array().unwrap();
    let keys_line = read(&f, "docs/keys.md")
        .lines()
        .position(|l| l == "## Rotate keys")
        .unwrap()
        + 1;
    let targets: Vec<(String, String)> = links
        .iter()
        .map(|l| {
            (
                l["range"].to_string(),
                l["target"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(links.len(), 4, "{targets:?}");
    let keys_uri = uri(&f.path("docs/keys.md"));
    assert_eq!(
        links[0]["target"],
        format!("{}#L{keys_line}", keys_uri.as_str())
    );
    assert_eq!(
        links[0]["range"],
        json!({ "start": { "line": 0, "character": 8 }, "end": { "line": 0, "character": 27 } })
    );
    assert_eq!(links[1]["target"], "https://example.com/x");
    assert_eq!(
        links[2]["target"],
        uri(&f.path("docs/playground.png")).as_str()
    );
    // The include's path, not the directive.
    assert_eq!(
        links[3]["target"],
        uri(&f.path("docs/_fragments/prerequisites.md")).as_str()
    );
    assert_eq!(
        links[3]["range"],
        json!({ "start": { "line": 2, "character": 10 }, "end": { "line": 2, "character": 37 } })
    );
}

#[test]
fn external_document_links_expand_destination_phrases() {
    let model = format!("{MODEL}\n[phrases]\nsite = \"https://example.com\"\n");
    let f = Fixture::new(&model, &[("docs/page.md", "See [site]({site}/docs).\n")]);
    let page = f.path("docs/page.md");
    let mut client = Client::start(&f.root());
    client.open(&page, 1, "See [site]({site}/docs).\n");

    let links = client
        .request(
            "textDocument/documentLink",
            json!({ "textDocument": text_doc(&page) }),
        )
        .response_result
        .expect("a result");
    assert_eq!(links[0]["target"], "https://example.com/docs");
}

#[test]
fn a_code_lens_names_the_include_and_opens_it() {
    let f = quill();
    let install = f.path("docs/install-agent.md");
    let text = read(&f, "docs/install-agent.md");
    let mut client = Client::start(&f.root());
    client.open(
        &install,
        1,
        "@include: _fragments/prerequisites.md\n\n@include: keys.md#create-key\n\n@include: gone.md\n",
    );
    let lenses = client
        .request(
            "textDocument/codeLens",
            json!({ "textDocument": text_doc(&install) }),
        )
        .response_result
        .expect("a result");
    let lenses = lenses.as_array().unwrap();
    // A missing target has no lens (the diagnostic says so).
    assert_eq!(lenses.len(), 2, "{lenses:?}");
    assert_eq!(
        lenses[0]["command"]["title"],
        "Includes _fragments/prerequisites.md"
    );
    assert_eq!(lenses[0]["command"]["command"], "ascribe.openFile");
    assert_eq!(
        lenses[0]["range"],
        json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 37 } })
    );
    assert_eq!(
        lenses[1]["command"]["title"],
        "Includes keys.md › Create a key"
    );
    let _ = text;

    // Running the command asks the client to show the file.
    let args = lenses[1]["command"]["arguments"].clone();
    let before = client.requests_from_server.len();
    let response = client.request(
        "workspace/executeCommand",
        json!({ "command": "ascribe.openFile", "arguments": args }),
    );
    assert!(response.response_result.is_ok());
    let shown: Vec<_> = client.requests_from_server[before..]
        .iter()
        .filter(|r| r.method == "window/showDocument")
        .collect();
    assert_eq!(shown.len(), 1);
    assert_eq!(
        shown[0].params["uri"],
        uri(&f.path("docs/keys.md")).as_str()
    );
    assert_eq!(shown[0].params["takeFocus"], true);
    assert_eq!(shown[0].params["selection"]["start"]["line"], 5);
    // Anything else is refused.
    let refused = client.request(
        "workspace/executeCommand",
        json!({ "command": "ascribe.openFile", "arguments": ["https://example.com"] }),
    );
    assert!(refused.response_result.is_err());
    let unknown = client.request("workspace/executeCommand", json!({ "command": "nope" }));
    assert!(unknown.response_result.is_err());
}

// -- Inlay hints ---------------------------------------------------------------------

fn hints(client: &mut Client, path: &Path, range: Value) -> Vec<Value> {
    client
        .request(
            "textDocument/inlayHint",
            json!({ "textDocument": text_doc(path), "range": range }),
        )
        .response_result
        .expect("a result")
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn whole() -> Value {
    json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 1000, "character": 0 } })
}

#[test]
fn empty_links_show_their_resolved_title() {
    let f = quill();
    let keys = f.path("docs/keys.md");
    let mut client = Client::start(&f.root());
    let text = read(&f, "docs/keys.md");
    let got = hints(&mut client, &keys, whole());
    assert_eq!(got.len(), 1, "{got:?}");
    // The text of a link to a heading is the heading's.
    assert_eq!(got[0]["label"], "Install the agent");
    let at_bracket = at(&text, "[](install-agent.md", 1);
    assert_eq!(got[0]["position"], at_bracket);

    // A link to a page shows its title, with declared phrases replaced; a
    // link with text has no hint; a range limits them.
    client.open(
        &keys,
        2,
        "A [](install-agent.md), [text](quickstart.md), [](quickstart.md).\n\n[](quickstart.md#try-in-browser)\n",
    );
    let got = hints(&mut client, &keys, whole());
    let labels: Vec<&str> = got.iter().map(|h| h["label"].as_str().unwrap()).collect();
    assert_eq!(
        labels,
        [
            "Install the Quill agent",
            "Try Quill in the browser",
            "Try in the browser"
        ]
    );
    let first_line =
        json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 70 } });
    assert_eq!(hints(&mut client, &keys, first_line).len(), 2);
    // A target that isn't there has none.
    client.open(&keys, 3, "[](nope.md) [](keys.md#nope)\n");
    assert!(hints(&mut client, &keys, whole()).is_empty());
}

// -- Freshness -----------------------------------------------------------------------

#[test]
fn answers_come_from_the_current_project() {
    let f = quill();
    let install = f.path("docs/install-agent.md");
    let keys = f.path("docs/keys.md");
    let mut client = Client::start(&f.root());
    let text = read(&f, "docs/keys.md");
    client.open(&keys, 1, &text);
    let before = hints(&mut client, &keys, whole());
    assert_eq!(before[0]["label"], "Install the agent");

    // Rename the heading the link points at, in the other file, and ask at
    // once: the hint, hover, and completion see the new title.
    let install_text = read(&f, "docs/install-agent.md");
    client.open(&install, 1, &install_text);
    let line = install_text
        .lines()
        .position(|l| l == "## Install the agent")
        .unwrap() as u32;
    client.change(
        &install,
        2,
        vec![support::edit((line, 3), (line, 20), "Install everything")],
    );
    let after = hints(&mut client, &keys, whole());
    assert_eq!(after[0]["label"], "Install everything");
    let list = complete(&mut client, &keys, "See [x](Install ev$0\n");
    assert!(labels(&list).contains(&"Install everything".to_owned()));
    client.open(&keys, 200, &text);
    let value = hover_text(
        &mut client,
        &keys,
        at(&text, "install-agent.md#install-agent", 3),
    );
    assert!(value.contains("**Install everything**"), "{value}");

    // A file deleted from disk is gone from the answers.
    f.remove("docs/keys.md");
    client.watched(&[(&keys, lsp_types::FileChangeType::DELETED)]);
    client.close(&keys);
    client.settle();
    let none = client
        .request(
            "textDocument/hover",
            json!({ "textDocument": text_doc(&keys), "position": { "line": 0, "character": 0 } }),
        )
        .response_result
        .expect("a result");
    assert!(none.is_null());
}

#[test]
fn requests_for_files_outside_the_project_are_null() {
    let f = quill();
    let mut client = Client::start(&f.root());
    let outside: Uri = "untitled:Untitled-1".parse().expect("a uri");
    for (method, params) in [
        (
            "textDocument/hover",
            json!({ "position": { "line": 0, "character": 0 } }),
        ),
        (
            "textDocument/definition",
            json!({ "position": { "line": 0, "character": 0 } }),
        ),
        (
            "textDocument/completion",
            json!({ "position": { "line": 0, "character": 0 } }),
        ),
        ("textDocument/documentLink", json!({})),
        ("textDocument/codeLens", json!({})),
    ] {
        let mut params = params;
        params["textDocument"] = json!({ "uri": outside.as_str() });
        let r = client
            .request(method, params)
            .response_result
            .expect("a result");
        assert!(r.is_null(), "{method}");
    }
    let mut params = json!({ "range": whole() });
    params["textDocument"] = json!({ "uri": outside.as_str() });
    let r = client
        .request("textDocument/inlayHint", params)
        .response_result
        .expect("a result");
    assert!(r.is_null());
}

// -- Robustness ----------------------------------------------------------------------

#[test]
fn no_request_fails_at_any_position() {
    let f = quill();
    let mut client = Client::start(&f.root());
    let tricky = "---\ntitle: 😀 T\navailable: cloud, \"self-managed\n---\n\n## Ünï {product} @id\n@id: ü\n\n> - @variant {pm=npm|}: [😀](k\u{301}.md#) ![a](b.png){wid\n@include: 😀/../#\n```yaml phrases=true\n{ver\n";
    let mut docs: Vec<(std::path::PathBuf, String)> =
        ["quickstart.md", "install-agent.md", "keys.md"]
            .iter()
            .map(|n| (f.path(&format!("docs/{n}")), read(&f, &format!("docs/{n}"))))
            .collect();
    docs.push((f.path("docs/tricky.md"), tricky.to_owned()));
    for (path, text) in docs {
        client.open(&path, 1, &text);
        for (line, l) in text.lines().enumerate() {
            let mut col = 0;
            for c in l.chars().chain(std::iter::once(' ')) {
                let pos = json!({ "line": line, "character": col });
                col += c.len_utf16();
                for method in ["hover", "definition", "completion"] {
                    let r = client.request(
                        &format!("textDocument/{method}"),
                        json!({ "textDocument": text_doc(&path), "position": pos }),
                    );
                    assert!(r.response_result.is_ok(), "{method} at {pos} of {path:?}");
                }
            }
        }
        for method in ["documentLink", "codeLens"] {
            let r = client.request(
                &format!("textDocument/{method}"),
                json!({ "textDocument": text_doc(&path) }),
            );
            assert!(r.response_result.is_ok());
        }
        assert!(
            client
                .request(
                    "textDocument/inlayHint",
                    json!({ "textDocument": text_doc(&path), "range": whole() })
                )
                .response_result
                .is_ok()
        );
    }
}
