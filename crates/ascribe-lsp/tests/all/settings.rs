//! Project Settings' round trip: `ascribe/settings` answers from the
//! registry and `ascribe.toml` as it is now, and `ascribe/edit`'s
//! `setSetting` changes one value in place, keeping the file's comments: it
//! sets a value the file writes, adds one it leaves out to its table, and
//! resets one by taking its line out with the comment that touches it. The
//! checks are listed with their levels and limits, and set the same way.
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
    set_in(client, fixture, "consumer", key, value, shown)
}

/// Sets `[table] key` to `value`, or resets it with `null`, as shown from the
/// answer for `shown`.
fn set_in(
    client: &mut Client,
    fixture: &Fixture,
    table: &str,
    key: &str,
    value: Value,
    shown: Value,
) -> Value {
    let config = fixture.path("ascribe.toml");
    client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": uri(&config).as_str() },
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                "action": "setSetting",
                "args": { "table": table, "key": key, "value": value, "shown": shown },
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
fn agents_defaults_on_with_a_site_and_off_without_one() {
    let fixture = fixture();
    let mut client = Client::start(&fixture.root());
    client.settle();
    let answer = settings(&mut client, &fixture);
    let agents = setting(&answer, "consumer", "agents");
    assert_eq!(agents["value"], Value::Null);
    assert_eq!(agents["default"], true, "the file sets a site");

    let without = Fixture::new(
        &MODEL.replace("site = \"https://docs.example.com\"\n", ""),
        &[("docs/index.md", "# Home\n\nHello.\n")],
    );
    let mut client = Client::start(&without.root());
    client.settle();
    let answer = settings(&mut client, &without);
    let agents = setting(&answer, "consumer", "agents");
    assert_eq!(agents["value"], Value::Null);
    assert_eq!(agents["default"], false, "the file sets no site");
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

#[test]
fn the_parts_with_structure_are_listed_with_their_keys_and_lines() {
    let model = r#"spec = "0.1"
dimensions.edition = { values = ["cloud", "server"] }

[project]
content-root = "docs"

[builds.cloud]
variants = { edition = "cloud" }

[types.guide]
purpose = "Teaches a task."
frontmatter = { title = "string" }

[releases]
planned-label = "Coming soon"

[releases.cloud]
live = ["2.0"]
"#;
    let fixture = Fixture::new(model, &[("docs/index.md", "# Home\n\nHello.\n")]);
    let mut client = Client::start(&fixture.root());
    client.settle();
    let answer = settings(&mut client, &fixture);
    let parts: Vec<(String, String, Vec<String>, Value)> = answer["parts"]
        .as_array()
        .expect("parts")
        .iter()
        .map(|p| {
            (
                p["table"].as_str().unwrap().to_owned(),
                p["name"].as_str().unwrap().to_owned(),
                p["keys"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|k| k.as_str().unwrap().to_owned())
                    .collect(),
                p["set"]["line"].clone(),
            )
        })
        .collect();
    // In the reference's order, not the file's; `planned-label`, a key of
    // `[releases]` itself, isn't a part.
    assert_eq!(
        parts,
        vec![
            (
                "types".to_owned(),
                "guide".to_owned(),
                vec!["purpose".to_owned(), "frontmatter".to_owned()],
                json!(10)
            ),
            (
                "dimensions".to_owned(),
                "edition".to_owned(),
                vec!["values".to_owned()],
                json!(2)
            ),
            (
                "releases".to_owned(),
                "cloud".to_owned(),
                vec!["live".to_owned()],
                json!(17)
            ),
            (
                "builds".to_owned(),
                "cloud".to_owned(),
                vec!["variants".to_owned()],
                json!(7)
            ),
        ]
    );
}

/// The edit `setSetting` answers for `[table] key`, applied to `text`, the
/// buffer at version 1.
fn edited(text: &str, table: &str, key: &str, value: Value) -> String {
    let fixture = Fixture::new(text, &[("docs/index.md", "# Home\n\nHello.\n")]);
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.open(&config, 1, text);
    client.settle();
    let answer = set_in(&mut client, &fixture, table, key, value, json!(1));
    assert!(answer["error"].is_null(), "{answer}");
    applied(text, &answer)
}

#[test]
fn a_value_the_file_leaves_out_is_added_to_its_table() {
    assert_eq!(
        edited(MODEL, "consumer", "slugger", json!("github")),
        MODEL.replace(
            "base-path = \"/\"\n",
            "base-path = \"/\"\nslugger = \"github\"\n"
        )
    );
    assert_eq!(
        edited(MODEL, "consumer", "html", json!(false)),
        MODEL.replace("base-path = \"/\"\n", "base-path = \"/\"\nhtml = false\n")
    );
    // A table the file doesn't have is added at its end.
    assert_eq!(
        edited(MODEL, "editor", "build", json!("site")),
        format!("{MODEL}\n[editor]\nbuild = \"site\"\n")
    );
    assert_eq!(
        edited(MODEL, "checks.links", "reuse-days", json!(7)),
        format!("{MODEL}\n[checks.links]\nreuse-days = 7\n")
    );
}

#[test]
fn reset_to_default_takes_the_line_and_the_comment_that_touches_it() {
    // The comment directly above goes with the line, and so does the one
    // after its value.
    assert_eq!(
        edited(MODEL, "consumer", "trailing-slash", Value::Null),
        MODEL.replace(
            "# Pages end in a slash.\ntrailing-slash = \"always\" # as Astro's\n",
            ""
        )
    );
    // The first key of a table: the header stays, with its comment.
    assert_eq!(
        edited(MODEL, "consumer", "site", Value::Null),
        MODEL.replace("site = \"https://docs.example.com\"\n", "")
    );
    // A comment with a blank line before the key stays: it may be about the
    // lines around it too.
    let apart = "spec = \"0.1\"\n\n[consumer]\nsite = \"https://docs.example.com\"\n\n# Routing.\n\nbase-path = \"/docs\"\nhtml = true\n";
    assert_eq!(
        edited(apart, "consumer", "base-path", Value::Null),
        "spec = \"0.1\"\n\n[consumer]\nsite = \"https://docs.example.com\"\n\n# Routing.\n\nhtml = true\n"
    );
    // A comment of several lines goes whole.
    let long = "spec = \"0.1\"\n\n[consumer]\nhtml = true\n# Two lines\n# about the slash.\ntrailing-slash = \"never\"\n";
    assert_eq!(
        edited(long, "consumer", "trailing-slash", Value::Null),
        "spec = \"0.1\"\n\n[consumer]\nhtml = true\n"
    );
    // A value the file leaves out has nothing to reset.
    assert_eq!(edited(MODEL, "consumer", "slugger", Value::Null), MODEL);
}

fn check<'a>(answer: &'a Value, slug: &str) -> &'a Value {
    answer["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["slug"] == slug)
        .unwrap_or_else(|| panic!("no check {slug}"))
}

#[test]
fn the_checks_list_every_configurable_check_the_ones_the_file_sets_first() {
    let model = format!(
        "{MODEL}\n[checks]\npage-orphan = \"off\"\n\n[checks.image-large]\nlimit = \"200 KB\"\n"
    );
    let fixture = Fixture::new(&model, &[("docs/index.md", "# Home\n\nHello.\n")]);
    let mut client = Client::start(&fixture.root());
    client.settle();
    let answer = settings(&mut client, &fixture);
    let checks = answer["checks"].as_array().unwrap();
    let slugs: Vec<&str> = checks.iter().map(|c| c["slug"].as_str().unwrap()).collect();
    let configurable: std::collections::BTreeSet<&str> = ascribe_core::diagnostics::CONFIGURABLE
        .iter()
        .map(|s| s.as_str())
        .collect();
    assert_eq!(checks.len(), configurable.len());
    assert_eq!(
        slugs
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        configurable
    );
    assert_eq!(
        &slugs[..2],
        &["page-orphan", "image-large"],
        "the file's, in the reference's order"
    );
    for c in checks {
        assert!(
            c["when"].as_array().is_some_and(|w| !w.is_empty()),
            "{} has no `when`",
            c["slug"]
        );
        assert_eq!(c["choices"], json!(["off", "advice", "warning", "error"]));
    }

    let orphan = check(&answer, "page-orphan");
    assert_eq!(orphan["named"], true);
    assert_eq!(orphan["level"], "off");
    assert_eq!(
        orphan["set"]["line"],
        model
            .lines()
            .position(|l| l.starts_with("page-orphan"))
            .unwrap()
            + 1
    );
    assert_eq!(orphan["limit"], Value::Null);

    let image = check(&answer, "image-large");
    assert_eq!(image["named"], true);
    assert_eq!(image["level"], Value::Null);
    assert_eq!(image["limit"]["kind"], "size");
    assert_eq!(image["limit"]["value"], "200 KB");
    assert_eq!(image["limit"]["default"], 500_000);
    assert_eq!(image["limit"]["defaultText"], "500 KB");

    let size = check(&answer, "page-size");
    assert_eq!(size["named"], false);
    assert_eq!(size["name"], "Page size");
    assert_eq!(size["code"], "ASC149");
    assert_eq!(size["area"], "Pages");
    assert_eq!(size["default"], "advice");
    assert_eq!(size["limit"]["kind"], "number");
    assert_eq!(size["limit"]["defaultText"], "50,000 characters");
    let when: String = size["when"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["text"].as_str().unwrap())
        .collect();
    assert!(
        when.starts_with("A page's Markdown, as a build writes it"),
        "{when}"
    );
    assert!(when.ends_with("longest."), "{when}");
}

#[test]
fn a_checks_level_and_limit_are_set_added_and_reset_in_the_form_the_file_writes() {
    let with = |checks: &str| format!("{MODEL}\n[checks]\n{checks}");
    // A check the file doesn't name: its level goes in `[checks]`, and a
    // limit in a table of its own.
    assert_eq!(
        edited(
            &with("page-orphan = \"off\"\n"),
            "checks.page-size",
            "level",
            json!("warning")
        ),
        with("page-orphan = \"off\"\npage-size = \"warning\"\n")
    );
    assert_eq!(
        edited(
            &with("page-orphan = \"off\"\n"),
            "checks.page-size",
            "limit",
            json!(40_000)
        ),
        format!(
            "{}\n[checks.page-size]\nlimit = 40000\n",
            with("page-orphan = \"off\"\n")
        )
    );
    // With no `[checks]` to add to, a level gets a table of its own too.
    assert_eq!(
        edited(MODEL, "checks.page-orphan", "level", json!("off")),
        format!("{MODEL}\n[checks.page-orphan]\nlevel = \"off\"\n")
    );
    // A level alone is set in place, takes a limit by becoming an inline
    // table, and is reset with its line.
    let alone = with("# Quieter.\npage-size = \"warning\"\n");
    assert_eq!(
        edited(&alone, "checks.page-size", "level", json!("error")),
        with("# Quieter.\npage-size = \"error\"\n")
    );
    assert_eq!(
        edited(&alone, "checks.page-size", "limit", json!(40_000)),
        with("# Quieter.\npage-size = { level = \"warning\", limit = 40000 }\n")
    );
    assert_eq!(
        edited(&alone, "checks.page-size", "level", Value::Null),
        with("")
    );
    // An inline table keeps its other key, and goes when it has none left.
    let inline = with("image-large = { level = \"warning\", limit = \"1 MB\" }\n");
    assert_eq!(
        edited(&inline, "checks.image-large", "limit", json!("2 MB")),
        with("image-large = { level = \"warning\", limit = \"2 MB\" }\n")
    );
    assert_eq!(
        edited(&inline, "checks.image-large", "level", Value::Null),
        with("image-large = { limit = \"1 MB\" }\n")
    );
    assert_eq!(
        edited(
            &with("image-large = { limit = \"1 MB\" }\n"),
            "checks.image-large",
            "limit",
            Value::Null
        ),
        with("")
    );
    // A table's lines are set, added, and reset as a setting's are.
    let table = format!("{MODEL}\n[checks.memo-long]\n# Longer memos.\nlimit = 3000\n");
    assert_eq!(
        edited(&table, "checks.memo-long", "level", json!("off")),
        format!("{table}level = \"off\"\n")
    );
    assert_eq!(
        edited(&table, "checks.memo-long", "limit", Value::Null),
        format!("{MODEL}\n[checks.memo-long]\n")
    );
}

#[test]
fn a_check_is_set_only_as_its_settings_take() {
    let fixture = fixture();
    let config = fixture.path("ascribe.toml");
    let mut client = Client::start(&fixture.root());
    client.open(&config, 1, MODEL);
    client.settle();
    let mut error = |table: &str, key: &str, value: Value| {
        set_in(&mut client, &fixture, table, key, value, json!(1))["error"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    };
    assert_eq!(
        error("checks.page-orphan", "limit", json!(3)),
        "`[checks.page-orphan] limit` isn't a setting Project Settings changes."
    );
    assert_eq!(
        error("checks.page-size", "limit", json!("50 KB")),
        "`[checks.page-size] limit` takes a whole number, or null."
    );
    assert_eq!(
        error("checks.page-size", "level", json!(2)),
        "`[checks.page-size] level` takes a string, or null."
    );
    // A check that isn't configurable isn't a check Project Settings sets.
    assert_eq!(
        error("checks.link-target-missing", "level", json!("off")),
        "`[checks.link-target-missing] level` isn't a setting Project Settings changes."
    );
}
