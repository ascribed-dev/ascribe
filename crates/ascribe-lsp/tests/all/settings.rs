//! Project Settings' round trip: `ascribe/settings` answers from the
//! registry and `ascribe.toml` as it is now, and `ascribe/edit`'s
//! `setSetting` changes one value in place, keeping the file's comments.
//! Each answer names the version of the buffer it read, an edit for another
//! version is refused, and a value the model refuses is set anyway, its
//! problem shown under it.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use ascribe_core::{LineIndex, WideEncoding, WideLineCol};
use serde_json::{Value, json};

use crate::support::{Client, Fixture, uri};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

# Routing, which the site's astro.config agrees with.
[consumer]
site = "https://docs.example.com"
# Pages end in a slash.
trailing-slash = "always" # as Astro's
base-path = "/"
"#;

fn fixture() -> Fixture {
    Fixture::new(MODEL, &[("docs/index.md", "# Home\n\nHello.\n")])
}

fn settings(client: &mut Client, fixture: &Fixture) -> Value {
    let config = fixture.path("ascribe.toml");
    client
        .request(
            "ascribe/settings",
            json!({ "textDocument": { "uri": uri(&config).as_str() } }),
        )
        .response_result
        .expect("an answer")
}

fn setting<'a>(answer: &'a Value, table: &str, key: &str) -> &'a Value {
    answer["settings"]
        .as_array()
        .expect("settings")
        .iter()
        .find(|s| s["table"] == table && s["key"] == key)
        .unwrap_or_else(|| panic!("no [{table}] {key}"))
}

/// Sets `[consumer] key` to `value`, as shown from the answer for `shown`:
/// a buffer's version, or `null` for the disk.
fn set(client: &mut Client, fixture: &Fixture, key: &str, value: Value, shown: Value) -> Value {
    let config = fixture.path("ascribe.toml");
    client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": uri(&config).as_str() },
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                "action": "setSetting",
                "args": { "table": "consumer", "key": key, "value": value, "shown": shown },
            }),
        )
        .response_result
        .expect("an answer")
}

/// `text` with the edit's text edits to its one file applied.
fn applied(text: &str, answer: &Value) -> String {
    let changes = answer["edit"]["changes"].as_object().expect("changes");
    assert_eq!(changes.len(), 1, "{answer}");
    let index = LineIndex::new(text);
    let offset = |p: &Value| {
        index
            .wide_offset(
                WideEncoding::Utf16,
                WideLineCol {
                    line: p["line"].as_u64().unwrap() as u32,
                    col: p["character"].as_u64().unwrap() as u32,
                },
            )
            .expect("an offset")
    };
    let edits: Vec<ascribe_core::TextEdit> = changes
        .values()
        .next()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            ascribe_core::TextEdit::replace(
                ascribe_core::Span::new(offset(&e["range"]["start"]), offset(&e["range"]["end"])),
                e["newText"].as_str().unwrap(),
            )
        })
        .collect();
    ascribe_core::apply_edits(text, &edits).expect("edits that apply")
}

/// The lines of `before` that `after` doesn't have.
fn changed_lines(before: &str, after: &str) -> Vec<String> {
    before
        .lines()
        .zip(after.lines())
        .filter(|(a, b)| a != b)
        .map(|(_, b)| b.to_owned())
        .collect()
}

#[test]
fn a_setting_shows_its_value_its_default_its_description_and_its_line() {
    let fixture = fixture();
    let mut client = Client::start(&fixture.root());
    client.settle();
    let answer = settings(&mut client, &fixture);
    assert_eq!(answer["version"], Value::Null, "read from the disk");
    assert_eq!(answer["unreadable"], Value::Null);
    let slash = setting(&answer, "consumer", "trailing-slash");
    assert_eq!(slash["name"], "Trailing slash");
    assert_eq!(slash["kind"], "choice");
    assert_eq!(slash["choices"], json!(["always", "never"]));
    assert_eq!(slash["value"], "always");
    assert_eq!(slash["default"], "always");
    assert_eq!(slash["set"]["line"], 10);
    assert_eq!(
        slash["set"]["range"]["start"],
        json!({ "line": 9, "character": 0 })
    );
    assert_eq!(slash["problems"], json!([]));
    let words: String = slash["description"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["text"].as_str().unwrap())
        .collect();
    assert!(
        words.starts_with("Routing. Whether page URLs end in /"),
        "{words}"
    );
    let base = setting(&answer, "consumer", "base-path");
    assert_eq!(base["value"], "/");
    assert_eq!(base["set"]["line"], 11);
    // A setting the file leaves out has its default and no place.
    let slugger = setting(&answer, "consumer", "slugger");
    assert_eq!(slugger["value"], Value::Null);
    assert_eq!(slugger["set"], Value::Null);
    assert_eq!(slugger["default"], "github");
}

#[test]
fn setting_a_value_changes_one_line_and_the_answer_follows_the_buffer() {
    let fixture = fixture();
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.open(&config, 1, MODEL);
    client.settle();
    assert_eq!(settings(&mut client, &fixture)["version"], 1);

    let answer = set(
        &mut client,
        &fixture,
        "trailing-slash",
        json!("never"),
        json!(1),
    );
    let after = applied(MODEL, &answer);
    assert_eq!(
        changed_lines(MODEL, &after),
        vec![r#"trailing-slash = "never" # as Astro's"#.to_owned()]
    );
    assert_eq!(after.lines().count(), MODEL.lines().count());

    client.replace(&config, 2, &after);
    client.settle();
    let now = settings(&mut client, &fixture);
    assert_eq!(now["version"], 2);
    assert_eq!(
        setting(&now, "consumer", "trailing-slash")["value"],
        "never"
    );

    // Undo in the file: the answer follows the buffer.
    client.replace(&config, 3, MODEL);
    client.settle();
    let undone = settings(&mut client, &fixture);
    assert_eq!(undone["version"], 3);
    assert_eq!(
        setting(&undone, "consumer", "trailing-slash")["value"],
        "always"
    );
}

#[test]
fn an_edit_for_an_older_version_is_refused() {
    let fixture = fixture();
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.open(&config, 4, MODEL);
    client.settle();
    let answer = set(
        &mut client,
        &fixture,
        "trailing-slash",
        json!("never"),
        json!(3),
    );
    assert_eq!(
        answer["error"],
        "`ascribe.toml` changed after the setting was shown. Choose the value again."
    );
}

#[test]
fn an_edit_shown_from_the_disk_is_refused_once_the_file_is_open() {
    let fixture = fixture();
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.settle();
    assert_eq!(settings(&mut client, &fixture)["version"], Value::Null);
    // Opened and typed in after the tab drew the disk's answer.
    client.open(
        &config,
        1,
        &MODEL.replace(r#"base-path = "/""#, r#"base-path = "/doc""#),
    );
    client.settle();
    let answer = set(
        &mut client,
        &fixture,
        "trailing-slash",
        json!("never"),
        Value::Null,
    );
    assert_eq!(
        answer["error"],
        "`ascribe.toml` changed after the setting was shown. Choose the value again."
    );
}

#[test]
fn an_edit_shown_from_a_buffer_is_refused_once_the_file_is_closed() {
    let fixture = fixture();
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.open(
        &config,
        2,
        &MODEL.replace(r#"base-path = "/""#, r#"base-path = "/doc""#),
    );
    client.settle();
    assert_eq!(settings(&mut client, &fixture)["version"], 2);
    // Closed without saving: the disk is what the file is now.
    client.close(&config);
    client.settle();
    let answer = set(
        &mut client,
        &fixture,
        "trailing-slash",
        json!("never"),
        json!(2),
    );
    assert_eq!(
        answer["error"],
        "`ascribe.toml` changed after the setting was shown. Choose the value again."
    );
    // From the disk's answer, it's set.
    let answer = set(
        &mut client,
        &fixture,
        "trailing-slash",
        json!("never"),
        Value::Null,
    );
    assert_eq!(
        changed_lines(MODEL, &applied(MODEL, &answer)),
        vec![r#"trailing-slash = "never" # as Astro's"#.to_owned()]
    );
}

#[test]
fn an_edit_must_say_what_it_was_shown_from() {
    let fixture = fixture();
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.open(&config, 1, MODEL);
    client.settle();
    let answer = client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": uri(&config).as_str() },
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                "action": "setSetting",
                "args": { "table": "consumer", "key": "trailing-slash", "value": "never" },
                "version": 1,
            }),
        )
        .response_result
        .expect("an answer");
    assert_eq!(
        answer["error"],
        "`shown` must be the version the setting was shown from, or null."
    );
}

#[test]
fn a_value_the_model_refuses_is_set_and_its_problem_shows_under_it() {
    let fixture = fixture();
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.open(&config, 1, MODEL);
    client.settle();
    let answer = set(&mut client, &fixture, "base-path", json!("docs"), json!(1));
    let after = applied(MODEL, &answer);
    assert_eq!(
        changed_lines(MODEL, &after),
        vec![r#"base-path = "docs""#.to_owned()]
    );
    client.replace(&config, 2, &after);
    client.settle();
    let now = settings(&mut client, &fixture);
    let base = setting(&now, "consumer", "base-path");
    assert_eq!(base["value"], "docs");
    assert_eq!(
        base["problems"],
        json!([{ "message": "base-path must start with \"/\", such as \"/docs/\"", "severity": "error" }])
    );
    assert_eq!(
        setting(&now, "consumer", "trailing-slash")["problems"],
        json!([])
    );

    // And it's set back from the file as it is, though the model doesn't load.
    let back = set(&mut client, &fixture, "base-path", json!("/"), json!(2));
    assert_eq!(applied(&after, &back), MODEL);
}

#[test]
fn a_setting_is_checked_against_the_registry_and_its_kind() {
    let fixture = fixture();
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.open(&config, 1, MODEL);
    client.settle();
    assert_eq!(
        set(
            &mut client,
            &fixture,
            "trailing-slash",
            json!(true),
            json!(1)
        )["error"],
        "`[consumer] trailing-slash` takes a string."
    );
    assert_eq!(
        set(&mut client, &fixture, "no-such-key", json!("x"), json!(1))["error"],
        "`[consumer] no-such-key` isn't a setting Project Settings changes."
    );
    assert_eq!(
        set(&mut client, &fixture, "slugger", json!("github"), json!(1))["error"],
        "`ascribe.toml` doesn't set `[consumer] slugger`. Add it to the file by hand."
    );
    // The value it has already: nothing to change.
    let same = set(
        &mut client,
        &fixture,
        "trailing-slash",
        json!("always"),
        json!(1),
    );
    assert_eq!(applied(MODEL, &same), MODEL);
}

#[test]
fn a_file_that_isnt_toml_has_settings_with_no_values() {
    let fixture = fixture();
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.settle();
    client.open(&config, 1, "spec = \n[consumer\n");
    client.settle();
    let answer = settings(&mut client, &fixture);
    assert_eq!(answer["version"], 1);
    assert!(
        answer["unreadable"]
            .as_str()
            .unwrap()
            .starts_with("`ascribe.toml` isn't valid TOML")
    );
    assert_eq!(
        setting(&answer, "consumer", "trailing-slash")["value"],
        Value::Null
    );
}
