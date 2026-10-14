//! Tested examples in the editor, against a copy of `examples/tested`: the
//! hover on a `@snippet`'s address, its advice following the observations,
//! the command to copy, the preview's tested line, and the source's
//! `ascribe.examples.toml` as it's edited.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use crate::support::{self, Client, Fixture, uri};
use lsp_types::FileChangeType;
use serde_json::{Value, json};

const PAGE: &str = "docs/index.md";
const CONFIG: &str = "code/ascribe.examples.toml";
const DEFAULT: &str = "code/examples.observations/default.toml";
const NODE_22: &str = "code/examples.observations/node-22.toml";

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

/// A copy of `examples/tested`.
fn tested() -> Fixture {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tested");
    let f = Fixture::new("", &[]);
    copy_dir(&source, &f.root());
    f
}

fn read(f: &Fixture, rel: &str) -> String {
    std::fs::read_to_string(f.path(rel)).expect("a file")
}

/// The LSP position of the first `needle` in `text`, plus `extra` bytes.
fn at(text: &str, needle: &str, extra: usize) -> Value {
    let i = text.find(needle).unwrap_or_else(|| panic!("no {needle:?}")) + extra;
    let before = &text[..i];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |n| n + 1);
    json!({ "line": line, "character": text[start..i].chars().map(char::len_utf16).sum::<usize>() })
}

fn hover(client: &mut Client, path: &Path, pos: Value) -> String {
    let r = client
        .request(
            "textDocument/hover",
            json!({ "textDocument": { "uri": uri(path).as_str() }, "position": pos }),
        )
        .response_result
        .expect("a result");
    r["contents"]["value"]
        .as_str()
        .unwrap_or_else(|| panic!("a hover: {r}"))
        .to_owned()
}

/// The hover on the page's first `@snippet`, the TypeScript file.
fn snippet_hover(f: &Fixture, client: &mut Client) -> String {
    let page = read(f, PAGE);
    hover(
        client,
        &f.path(PAGE),
        at(&page, "code:examples/greet.ts", 5),
    )
}

/// The observation with `greet.ts` hashed as something it isn't.
fn edited_hash(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.starts_with("\"examples/greet.ts\"") {
                format!("\"examples/greet.ts\" = \"sha256-lf:{}\"", "0".repeat(64))
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_hover_shows_the_file_and_what_it_was_tested_with() {
    let f = tested();
    let mut client = Client::start(&f.root());
    client.settle();
    let text = snippet_hover(&f, &mut client);
    // The file, as a link that opens it at the region.
    let file = uri(&f.path("code/examples/greet.ts"));
    assert!(
        text.starts_with(&format!(
            "[code\\/examples\\/greet\\.ts](<{}#L",
            file.as_str()
        )),
        "{text}"
    );
    assert!(text.contains(", region `greet`, lines 2 to 6"), "{text}");
    assert!(
        text.contains("**Tested with** Node 22\\.22\\.0 and 24\\.21\\.0 on Linux"),
        "{text}"
    );
    assert!(
        text.contains("Observations current: `default`, `node-22`"),
        "{text}"
    );
    // A current example has nothing to run.
    assert!(!text.contains("ascribe observe"), "{text}");
}

#[test]
fn the_hover_says_where_to_run_a_stale_example() {
    let f = tested();
    f.write(DEFAULT, &edited_hash(&read(&f, DEFAULT)));
    let mut client = Client::start(&f.root());
    client.settle();
    let text = snippet_hover(&f, &mut client);
    // `node-22` still has it as it is.
    assert!(
        text.contains("Tested with** Node 22\\.22\\.0 on Linux"),
        "{text}"
    );
    f.write(NODE_22, &edited_hash(&read(&f, NODE_22)));
    let text = snippet_hover(&f, &mut client);
    assert!(
        text.contains("**Changed after its last test run**"),
        "{text}"
    );
    assert!(
        text.contains("Observations that list it: `default`, `node-22`"),
        "{text}"
    );
    assert!(
        text.contains("Run `ascribe observe` in `code` to test it as it is now."),
        "{text}"
    );
}

#[test]
fn the_hover_says_an_example_no_observation_lists_is_not_tested() {
    let f = tested();
    f.remove(DEFAULT);
    f.remove(NODE_22);
    let mut client = Client::start(&f.root());
    client.settle();
    let text = snippet_hover(&f, &mut client);
    assert!(
        text.contains("**Not tested:** no observation lists it."),
        "{text}"
    );
    assert!(text.contains("Run `ascribe observe` in `code`"), "{text}");
}

#[test]
fn a_hostile_label_is_shown_as_text() {
    let f = tested();
    let config = read(&f, CONFIG).replace(
        "label = \"Node\"",
        "label = \"<img src=x onerror=alert(1)>[x](javascript:alert(1))\"",
    );
    f.write(CONFIG, &config);
    let mut client = Client::start(&f.root());
    client.settle();
    let text = snippet_hover(&f, &mut client);
    assert!(
        text.contains("\\<img src\\=x onerror\\=alert\\(1\\)\\>\\[x\\]\\(javascript\\:alert"),
        "{text}"
    );
    // Nothing in the line can open a tag or a link.
    let line = text
        .lines()
        .find(|l| l.starts_with("**Tested with**"))
        .expect("the tested line");
    for (i, c) in line.char_indices() {
        if "<>[]()".contains(c) {
            assert_eq!(&line[i - 1..i], "\\", "{line}");
        }
    }
}

#[test]
fn a_changed_observation_makes_the_line_stale_without_an_edit() {
    let f = tested();
    let page = f.path(PAGE);
    let mut client = Client::start(&f.root());
    client.settle();
    assert!(!client.codes(&page).contains(&"example-stale".to_owned()));
    f.write(DEFAULT, &edited_hash(&read(&f, DEFAULT)));
    f.write(NODE_22, &edited_hash(&read(&f, NODE_22)));
    client.watched(&[
        (&f.path(DEFAULT), FileChangeType::CHANGED),
        (&f.path(NODE_22), FileChangeType::CHANGED),
    ]);
    client.settle();
    let stale: Vec<_> = client
        .diagnostics(&page)
        .into_iter()
        .filter(|d| support::slug(d) == "example-stale")
        .collect();
    assert_eq!(stale.len(), 1, "{stale:?}");
    // At the `@snippet` line of the TypeScript file.
    let text = read(&f, PAGE);
    let line = text[..text.find("code:examples/greet.ts").unwrap()]
        .matches('\n')
        .count();
    assert_eq!(stale[0].range.start.line as usize, line);

    // Deleting the observations' folder leaves it unobserved.
    f.remove(DEFAULT);
    f.remove(NODE_22);
    std::fs::remove_dir(f.path("code/examples.observations")).expect("rmdir");
    client.watched(&[(
        &f.path("code/examples.observations"),
        FileChangeType::DELETED,
    )]);
    client.settle();
    let codes = client.codes(&page);
    assert!(
        codes.contains(&"example-unobserved".to_owned()),
        "{codes:?}"
    );
    assert!(!codes.contains(&"example-stale".to_owned()), "{codes:?}");
}

#[test]
fn the_advice_offers_to_copy_the_command_and_nothing_that_runs_it() {
    let f = tested();
    f.remove(DEFAULT);
    f.remove(NODE_22);
    let page = f.path(PAGE);
    let mut client = Client::start(&f.root());
    client.settle();
    let unobserved = client
        .diagnostics(&page)
        .into_iter()
        .find(|d| support::slug(d) == "example-unobserved")
        .expect("example-unobserved");
    let actions = client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(&page).as_str() },
                "range": unobserved.range,
                "context": { "diagnostics": [unobserved] }
            }),
        )
        .response_result
        .expect("actions");
    let actions = actions.as_array().expect("a list");
    assert_eq!(actions.len(), 1, "{actions:?}");
    let copy = &actions[0];
    assert_eq!(copy["title"], "Copy the command");
    assert_eq!(copy["edit"], Value::Null);
    assert_eq!(copy["command"]["command"], "ascribe.copyCommand");
    let folder = f.path("code");
    assert_eq!(
        copy["command"]["arguments"],
        json!([if cfg!(windows) {
            format!(
                "Set-Location -LiteralPath '{}'; ascribe observe",
                folder.display()
            )
        } else {
            format!("cd '{}' && ascribe observe", folder.display())
        }])
    );
}

#[test]
fn the_preview_has_the_tested_line() {
    let f = tested();
    let page = f.path(PAGE);
    let mut client = Client::start(&f.root());
    client.settle();
    let result = client
        .request(
            "ascribe/preview",
            json!({ "textDocument": { "uri": uri(&page).as_str() } }),
        )
        .response_result
        .expect("the preview");
    let html = result["page"]["html"].as_str().expect("html");
    assert!(
        html.contains("<div class=\"ascribe-code-tested\">Node 22.22.0 and 24.21.0 on Linux</div>"),
        "{html}"
    );
}

#[test]
fn an_open_config_reports_its_problems_where_they_are() {
    let f = tested();
    let config = f.path(CONFIG);
    let mut client = Client::start(&f.root());
    client.settle();
    let text = "include = [\"examples/**\"]\ntests = \"node --test\"\n";
    client.open(&config, 1, text);
    client.settle();
    let found = client.diagnostics(&config);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(support::slug(&found[0]), "examples-config-invalid");
    assert_eq!(found[0].range.start.line, 1);
    assert!(
        found[0].message.contains("it has no key `tests`"),
        "{}",
        found[0].message
    );
    // Fixed as it's typed.
    client.replace(
        &config,
        2,
        "include = [\"examples/**\"]\ntest = \"node --test\"\n",
    );
    client.settle();
    assert!(client.diagnostics(&config).is_empty());
    // A config no source names is just a TOML file.
    let other = f.path("code/examples/ascribe.examples.toml");
    client.open(&other, 1, "nope = 1\n");
    client.settle();
    assert!(client.publications(&other).is_empty());
}

#[test]
fn an_open_config_completes_its_keys() {
    let f = tested();
    let config = f.path(CONFIG);
    let mut client = Client::start(&f.root());
    client.settle();
    let text = "include = [\"examples/**\"]\n\n\n[environment]\nnode = { label = \"Node\", }\n";
    client.open(&config, 1, text);
    let labels = |client: &mut Client, pos: Value| -> Vec<String> {
        let r = client
            .request(
                "textDocument/completion",
                json!({ "textDocument": { "uri": uri(&config).as_str() }, "position": pos }),
            )
            .response_result
            .expect("a result");
        r.as_array()
            .expect("items")
            .iter()
            .map(|i| i["label"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(
        labels(&mut client, at(text, "\n\n\n", 1)),
        ["ignore", "test"]
    );
    assert_eq!(labels(&mut client, at(text, ", }", 2)), ["command"]);
}
